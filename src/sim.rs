//! Event-driven combat engine. Knows nothing about any class: everything
//! comes from the compiled spec.

use crate::expr::{self, Env, Expr, Resolver, Var};
use crate::spec::*;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

const INF: f64 = f64::INFINITY;
const MAX_TRIGGER_DEPTH: u32 = 8;

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

const GOLDEN: u64 = 0x9E3779B97F4A7C15;

/// Random numbers as independent streams, one per purpose: fight length, the
/// crit roll and damage range of each source, every chance roll site and every
/// RPPM proc. Each stream is its own splitmix64 sequence keyed by
/// (iteration seed, stream name), so the n-th crit roll of Final Verdict is the
/// same number in two runs with the same seed no matter what else happened in
/// between. That keeps paired comparisons (same seed, different stats or gear)
/// sharing their luck after the runs diverge. Keys come from names, not
/// positions, so a gear set that adds a trinket proc leaves every other
/// stream untouched.
struct Streams(Vec<u64>);
impl Streams {
    fn new(seed: u64, keys: &[u64]) -> Self {
        let s = mix(seed.wrapping_add(GOLDEN));
        Streams(keys.iter().map(|&k| mix(s ^ k)).collect())
    }
    /// Uniform in [0, 1), next number of stream `k`.
    fn f64(&mut self, k: usize) -> f64 {
        let st = &mut self.0[k];
        *st = st.wrapping_add(GOLDEN);
        (mix(*st) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
    fn range(&mut self, k: usize, lo: f64, hi: f64) -> f64 { lo + (hi - lo) * self.f64(k) }
}

/// Stable 64-bit key for a stream name (FNV-1a, then mixed).
fn stream_key(name: &str) -> u64 {
    let mut h: u64 = 0xCBF29CE484222325;
    for b in name.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100000001B3);
    }
    mix(h)
}

// ---------- compiled spec ----------

#[derive(Debug, Clone)]
enum CEffect {
    // `rid`: this effect's own random stream for its chance roll
    Buff { buff: usize, stacks: u32, chance: f64, rid: usize },
    Consume { buff: usize, all: bool },
    Reset { ab: usize, chance: f64, rid: usize },
    Reduce { ab: usize, secs: f64, chance: f64, rid: usize },
    Gain { res: usize, amt: f64, chance: f64, rid: usize },
    Trigger { ab: usize, chance: f64, delay: f64, mastery_chance: Option<f64>, rid: usize },
    ExtendDot { dot: usize, secs: f64 },
    ExtendBuff { buff: usize, secs: f64 },
    If { cond: Expr, then: Vec<CEffect>, otherwise: Vec<CEffect> },
}

/// How a buff's presence is decided.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BuffKind {
    /// One timer shared by all stacks.
    Shared,
    /// Each application has its own timer.
    Overlap,
    /// Up while the target's health is within (lo, hi].
    Health(f64, f64),
}

#[derive(Debug, Clone)]
struct CAbility {
    def: AbilityDef,
    cost: Vec<(usize, f64)>,
    gain: Vec<(usize, f64)>,
    free_with: Option<usize>,
    requires_buff: Option<usize>,
    on_start: Vec<CEffect>,
    on_cast: Vec<CEffect>,
    on_crit: Vec<CEffect>,
    on_noncrit: Vec<CEffect>,
    /// Result row this ability's casts and damage are reported under.
    src: usize,
    /// (banked dot, pct, per_mastery) shares of each hit fed into DoTs.
    feeds: Vec<(usize, f64, f64)>,
    /// (buff, fraction per stack) crit chance that applies to this ability only.
    crit_mods: Vec<(usize, f64)>,
    dot: Option<usize>,
    stored_from: Option<usize>,
    /// (buff, fraction) recharge speed-ups for this ability.
    cd_rate: Vec<(usize, f64)>,
    /// (buff, fraction per stack) bonuses that apply to this ability's direct damage.
    buff_mods: Vec<(usize, f64)>,
    physical: bool,
}

#[derive(Debug, Clone)]
struct CDot {
    def: DotDef,
    on_tick: Vec<CEffect>,
    buff_mods: Vec<(usize, f64)>,
    crit_mods: Vec<(usize, f64)>,
    physical: bool,
    src: usize,
}

#[derive(Debug, Clone)]
pub struct Compiled {
    pub spec: Spec,
    res_names: Vec<String>,
    res_max: Vec<f64>,
    res_start: Vec<f64>,
    abilities: Vec<CAbility>,
    buffs: Vec<BuffDef>,
    dots: Vec<CDot>,
    rotation: Vec<(usize, Option<Expr>)>,
    precombat: Vec<usize>,
    procs: Vec<(usize, f64, bool)>,
    auto_mods: Vec<(usize, f64)>,
    auto_on_hit: Vec<CEffect>,
    auto_mult: f64,
    auto_replace: Option<usize>,
    haste_buffs: Vec<(usize, f64)>,
    gcd_buffs: Vec<(usize, f64)>,
    crit_buffs: Vec<(usize, f64)>,
    store_buffs: Vec<(usize, f64)>,
    crit_dmg_buffs: Vec<(usize, f64)>,
    mastery_buffs: Vec<(usize, f64)>,
    power_buffs: Vec<(usize, f64)>,
    buff_kind: Vec<BuffKind>,
    on_expire: Vec<Vec<CEffect>>,
    /// Per buff: (ability, fraction per stack) recharge speed-ups it gives.
    rate_users: Vec<Vec<(usize, f64)>>,
    names: Vec<String>,
    /// Key of every random stream, in the layout of the `*_stream` methods.
    stream_keys: Vec<u64>,
}

struct Names<'a> {
    res: &'a HashMap<String, usize>,
    ab: &'a HashMap<String, usize>,
    buff: &'a HashMap<String, usize>,
    dot: &'a HashMap<String, usize>,
}

fn get(m: &HashMap<String, usize>, kind: &str, n: &str) -> Result<usize, String> {
    m.get(n).copied().ok_or_else(|| format!("unknown {kind} '{n}'"))
}

impl<'a> Resolver for Names<'a> {
    fn resolve(&self, name: &str) -> Result<Var, String> {
        if let Some(&i) = self.res.get(name) {
            return Ok(Var::Resource(i));
        }
        let parts: Vec<&str> = name.split('.').collect();
        Ok(match parts.as_slice() {
            ["gcd"] => Var::Gcd,
            ["time"] => Var::Time,
            ["fight_remains"] | ["target", "time_to_die"] => Var::FightRemains,
            ["target", "health", "pct"] => Var::HealthPct,
            ["active_enemies"] => Var::Const(1.0),
            ["cooldown", a, f] => {
                let i = get(self.ab, "ability", a)?;
                match *f {
                    "ready" | "up" => Var::CdReady(i),
                    "remains" => Var::CdRemains(i),
                    "charges" => Var::CdCharges(i),
                    "charges_fractional" => Var::CdChargesFrac(i),
                    _ => return Err(format!("unknown cooldown field '{f}'")),
                }
            }
            ["buff", bn, f] => {
                let i = get(self.buff, "buff", bn)?;
                match *f {
                    "up" => Var::BuffUp(i),
                    "remains" => Var::BuffRemains(i),
                    "stack" | "stacks" => Var::BuffStack(i),
                    _ => return Err(format!("unknown buff field '{f}'")),
                }
            }
            ["dot", dn, f] => {
                let i = get(self.dot, "dot", dn)?;
                match *f {
                    "ticking" | "up" => Var::DotTicking(i),
                    "remains" => Var::DotRemains(i),
                    _ => return Err(format!("unknown dot field '{f}'")),
                }
            }
            _ => return Err(format!("unknown identifier '{name}'")),
        })
    }
}

/// Per-ability entry for `name`, plus the shared `source` row's entry if it differs.
fn keyed(m: &HashMap<String, f64>, name: &str, source: &str) -> f64 {
    m.get(name).copied().unwrap_or(0.0) + if source != name { m.get(source).copied().unwrap_or(0.0) } else { 0.0 }
}

fn crit_mods(buffs: &[BuffDef], ability: &str, source: &str) -> Vec<(usize, f64)> {
    buffs
        .iter()
        .enumerate()
        .map(|(i, b)| (i, keyed(&b.ability_crit_pct, ability, source) / 100.0))
        .filter(|&(_, p)| p != 0.0)
        .collect()
}

/// (buff, value / 100) for every buff whose `field` is non-zero.
fn per_buff(buffs: &[BuffDef], field: impl Fn(&BuffDef) -> f64) -> Vec<(usize, f64)> {
    buffs.iter().enumerate().filter(|(_, b)| field(b) != 0.0).map(|(i, b)| (i, field(b) / 100.0)).collect()
}

/// Compile one effect list. `site` names where it sits in the spec; every
/// chance-roll site gets its own random stream, registered in `rolls`.
fn compile_effects(v: &[Effect], n: &Names, site: &str, rolls: &mut Vec<String>) -> Result<Vec<CEffect>, String> {
    v.iter().enumerate().map(|(k, e)| compile_effect(e, n, &format!("{site}/{k}"), rolls)).collect()
}

fn compile_effect(e: &Effect, n: &Names, site: &str, rolls: &mut Vec<String>) -> Result<CEffect, String> {
    let mut rid = || {
        rolls.push(site.to_string());
        rolls.len() - 1
    };
    Ok(match e {
        Effect::Buff { buff: b, stacks, chance } => {
            CEffect::Buff { buff: get(n.buff, "buff", b)?, stacks: *stacks, chance: *chance, rid: rid() }
        }
        Effect::Consume { buff: b, all } => CEffect::Consume { buff: get(n.buff, "buff", b)?, all: *all },
        Effect::ResetCooldown { ability, chance } => {
            CEffect::Reset { ab: get(n.ab, "ability", ability)?, chance: *chance, rid: rid() }
        }
        Effect::ReduceCooldown { ability, seconds, chance } => {
            CEffect::Reduce { ab: get(n.ab, "ability", ability)?, secs: *seconds, chance: *chance, rid: rid() }
        }
        Effect::Gain { resource, amount, chance } => {
            CEffect::Gain { res: get(n.res, "resource", resource)?, amt: *amount, chance: *chance, rid: rid() }
        }
        Effect::ExtendDot { dot: d, seconds } => CEffect::ExtendDot { dot: get(n.dot, "dot", d)?, secs: *seconds },
        Effect::ExtendBuff { buff: b, seconds } => {
            CEffect::ExtendBuff { buff: get(n.buff, "buff", b)?, secs: *seconds }
        }
        Effect::If { cond, then, otherwise } => CEffect::If {
            cond: expr::parse(cond, n).map_err(|e| format!("if '{cond}': {e}"))?,
            then: compile_effects(then, n, &format!("{site}/then"), rolls)?,
            otherwise: compile_effects(otherwise, n, &format!("{site}/else"), rolls)?,
        },
        Effect::Trigger { ability, chance, delay, mastery_chance } => CEffect::Trigger {
            ab: get(n.ab, "ability", ability)?,
            chance: *chance,
            delay: *delay,
            mastery_chance: *mastery_chance,
            rid: rid(),
        },
    })
}

fn buff_mods(buffs: &[BuffDef], ability: &str, source: &str, school: &str) -> Vec<(usize, f64)> {
    let mut v = Vec::new();
    for (i, b) in buffs.iter().enumerate() {
        let pct = b.damage_pct
            + b.school_pct.get(school).copied().unwrap_or(0.0)
            + keyed(&b.ability_pct, ability, source);
        if pct != 0.0 {
            v.push((i, pct / 100.0));
        }
    }
    v
}

impl Compiled {
    pub fn new(spec: Spec) -> Result<Self, String> {
        let mut res_names: Vec<String> = spec.resources.keys().cloned().collect();
        res_names.sort();
        let res: HashMap<String, usize> =
            res_names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        let ab: HashMap<String, usize> =
            spec.abilities.iter().enumerate().map(|(i, a)| (a.name.clone(), i)).collect();
        if ab.len() != spec.abilities.len() {
            return Err("duplicate ability names".into());
        }
        let buff: HashMap<String, usize> =
            spec.buffs.iter().enumerate().map(|(i, b)| (b.name.clone(), i)).collect();
        let mut dot = HashMap::new();
        for a in &spec.abilities {
            if let Some(d) = &a.dot {
                if !["pandemic", "extend", "reset"].contains(&d.refresh.as_str()) {
                    return Err(format!("dot '{}': refresh must be pandemic, extend or reset", d.name));
                }
                let n = dot.len();
                dot.insert(d.name.clone(), n);
            }
        }
        let names = Names { res: &res, ab: &ab, buff: &buff, dot: &dot };

        let rolls = std::cell::RefCell::new(Vec::<String>::new());
        let effects = |v: &[Effect], at: &str| -> Result<Vec<CEffect>, String> {
            compile_effects(v, &names, at, &mut rolls.borrow_mut())
        };
        let rmap = |m: &HashMap<String, f64>| -> Result<Vec<(usize, f64)>, String> {
            m.iter().map(|(k, v)| Ok((get(&res, "resource", k)?, *v))).collect()
        };

        // Result rows: abilities (variants sharing a `source` share a row),
        // then dots, then auto attack.
        let mut src_names: Vec<String> = Vec::new();
        let src_of = |n: &str, v: &mut Vec<String>| -> usize {
            v.iter().position(|x| x == n).unwrap_or_else(|| {
                v.push(n.to_string());
                v.len() - 1
            })
        };
        let ab_src: Vec<usize> = spec
            .abilities
            .iter()
            .map(|a| src_of(a.source.as_deref().unwrap_or(&a.name), &mut src_names))
            .collect();
        for a in &spec.abilities {
            if let Some(d) = &a.dot {
                src_of(&d.name, &mut src_names);
            }
        }
        src_names.push("auto_attack".into());

        let mut abilities = Vec::new();
        let mut dots = Vec::new();
        for (ai, a) in spec.abilities.iter().enumerate() {
            let school = a.damage.as_ref().map(|d| d.school.as_str()).unwrap_or("physical");
            let ctx = |e: String| format!("ability '{}': {e}", a.name);
            let dot_idx = match &a.dot {
                Some(d) => {
                    dots.push(CDot {
                        on_tick: effects(&d.on_tick, &format!("{}/on_tick", d.name)).map_err(ctx)?,
                        // banked damage already carries the hit's buffs: only buffs naming the DoT apply
                        buff_mods: if d.banked {
                            per_buff(&spec.buffs, |b| b.ability_pct.get(&d.name).copied().unwrap_or(0.0))
                        } else {
                            buff_mods(&spec.buffs, &d.name, &d.name, &d.school)
                        },
                        crit_mods: crit_mods(&spec.buffs, &d.name, &d.name),
                        physical: d.school == "physical",
                        src: src_names.iter().position(|x| *x == d.name).unwrap(),
                        def: d.clone(),
                    });
                    Some(dots.len() - 1)
                }
                None => None,
            };
            abilities.push(CAbility {
                cost: rmap(&a.cost).map_err(ctx)?,
                gain: rmap(&a.gain).map_err(ctx)?,
                free_with: a.free_with.as_ref().map(|b| get(&buff, "buff", b)).transpose().map_err(ctx)?,
                requires_buff: a
                    .requires_buff
                    .as_ref()
                    .map(|b| get(&buff, "buff", b))
                    .transpose()
                    .map_err(ctx)?,
                on_start: effects(&a.on_start, &format!("{}/on_start", a.name)).map_err(ctx)?,
                on_cast: effects(&a.on_cast, &format!("{}/on_cast", a.name)).map_err(ctx)?,
                on_crit: effects(&a.on_crit, &format!("{}/on_crit", a.name)).map_err(ctx)?,
                on_noncrit: effects(&a.on_noncrit, &format!("{}/on_noncrit", a.name)).map_err(ctx)?,
                src: ab_src[ai],
                feeds: a
                    .damage
                    .iter()
                    .flat_map(|d| &d.feed)
                    .map(|f| Ok((get(&dot, "dot", &f.dot)?, f.pct / 100.0, f.per_mastery)))
                    .collect::<Result<_, String>>()
                    .map_err(ctx)?,
                crit_mods: crit_mods(&spec.buffs, &a.name, a.source.as_deref().unwrap_or(&a.name)),
                dot: dot_idx,
                stored_from: a
                    .damage
                    .as_ref()
                    .and_then(|d| d.stored_from.as_ref())
                    .map(|b| get(&buff, "buff", b))
                    .transpose()
                    .map_err(ctx)?,
                buff_mods: buff_mods(&spec.buffs, &a.name, a.source.as_deref().unwrap_or(&a.name), school),
                cd_rate: spec
                    .buffs
                    .iter()
                    .enumerate()
                    .filter_map(|(bi, b)| b.cooldown_rate_pct.get(&a.name).map(|p| (bi, p / 100.0)))
                    .collect(),
                physical: school == "physical",
                def: a.clone(),
            });
        }

        let mut rotation = Vec::new();
        for r in &spec.rotation {
            let i = get(&ab, "ability", &r.action).map_err(|e| format!("rotation: {e}"))?;
            let c = match &r.condition {
                Some(s) => {
                    Some(expr::parse(s, &names).map_err(|e| format!("rotation '{}': {e}", r.action))?)
                }
                None => None,
            };
            rotation.push((i, c));
        }
        let precombat =
            spec.precombat.iter().map(|n| get(&ab, "ability", n)).collect::<Result<_, _>>()?;
        let procs = spec
            .procs
            .iter()
            .map(|p| Ok((get(&ab, "ability", &p.ability)?, p.per_minute, p.hasted)))
            .collect::<Result<_, String>>()?;

        let (auto_on_hit, auto_mult, auto_replace) = match &spec.auto_attack {
            Some(a) => (
                effects(&a.on_hit, "auto_attack/on_hit")?,
                a.mult,
                a.replace_with.as_ref().map(|n| get(&ab, "ability", n)).transpose()?,
            ),
            None => (vec![], 1.0, None),
        };
        let haste_buffs = spec
            .buffs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.haste_pct != 0.0)
            .map(|(i, b)| (i, b.haste_pct / 100.0))
            .collect();
        let gcd_buffs = spec
            .buffs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.gcd_pct != 0.0)
            .map(|(i, b)| (i, b.gcd_pct / 100.0))
            .collect();
        let store_buffs = spec
            .buffs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.store_damage_pct != 0.0)
            .map(|(i, b)| (i, b.store_damage_pct / 100.0))
            .collect();
        let crit_buffs = spec
            .buffs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.crit_pct != 0.0)
            .map(|(i, b)| (i, b.crit_pct / 100.0))
            .collect();
        for d in &dots {
            if d.def.banked && d.def.tick_on_apply {
                return Err(format!("dot '{}': banked dots can't tick on apply", d.def.name));
            }
        }
        let buff_kind = spec
            .buffs
            .iter()
            .map(|b| match (b.below_health, b.above_health, b.overlap) {
                (None, None, false) => BuffKind::Shared,
                (None, None, true) => BuffKind::Overlap,
                (below, above, _) => BuffKind::Health(above.unwrap_or(-1.0), below.unwrap_or(1e9)),
            })
            .collect();
        let on_expire = spec.buffs.iter().map(|b| effects(&b.on_expire, &format!("{}/on_expire", b.name))).collect::<Result<_, _>>()?;
        let rate_users = (0..spec.buffs.len())
            .map(|b| {
                abilities
                    .iter()
                    .enumerate()
                    .flat_map(|(i, a)| a.cd_rate.iter().filter(|r| r.0 == b).map(move |r| (i, r.1)))
                    .collect()
            })
            .collect();

        let roll_names = rolls.into_inner();
        let mut stream_names = vec!["fight_length".to_string()];
        stream_names.extend(src_names.iter().map(|n| format!("crit/{n}")));
        stream_names.extend(src_names.iter().map(|n| format!("range/{n}")));
        stream_names.extend(spec.procs.iter().map(|p| format!("rppm/{}", p.ability)));
        stream_names.extend(roll_names.iter().map(|n| format!("roll/{n}")));
        // the same proc listed twice still needs two streams
        let mut seen = HashMap::new();
        let stream_keys = stream_names
            .iter()
            .map(|n| {
                let k = seen.entry(n.clone()).or_insert(0u32);
                *k += 1;
                stream_key(&if *k == 1 { n.clone() } else { format!("{n}#{k}") })
            })
            .collect();

        Ok(Compiled {
            res_max: res_names.iter().map(|n| spec.resources[n].max).collect(),
            res_start: res_names.iter().map(|n| spec.resources[n].start).collect(),
            res_names,
            auto_mods: buff_mods(&spec.buffs, "auto_attack", "auto_attack", "physical"),
            buffs: spec.buffs.clone(),
            abilities,
            dots,
            rotation,
            precombat,
            procs,
            auto_on_hit,
            auto_mult,
            auto_replace,
            haste_buffs,
            gcd_buffs,
            crit_buffs,
            store_buffs,
            crit_dmg_buffs: per_buff(&spec.buffs, |b| b.crit_damage_pct),
            mastery_buffs: per_buff(&spec.buffs, |b| b.mastery_pct),
            power_buffs: per_buff(&spec.buffs, |b| b.power_pct),
            buff_kind,
            on_expire,
            rate_users,
            names: src_names,
            stream_keys,
            spec,
        })
    }

    /// Damage sources in result order: abilities, then dots, then auto attack.
    pub fn source_names(&self) -> &[String] { &self.names }

    // Stream layout: fight length, crit per source, damage range per source,
    // RPPM per proc, then one per chance-roll site.
    fn crit_stream(&self, src: usize) -> usize { 1 + src }
    fn range_stream(&self, src: usize) -> usize { 1 + self.names.len() + src }
    fn proc_stream(&self, k: usize) -> usize { 1 + 2 * self.names.len() + k }
    fn roll_stream(&self, rid: usize) -> usize { 1 + 2 * self.names.len() + self.procs.len() + rid }
}

const FIGHT_STREAM: usize = 0;

// ---------- per-iteration state ----------

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Ev {
    Ready,
    Auto,
    DotTick(u32, u32),
    Trigger(u32),
    Proc(u32),
    CastEnd(u32),
    BuffExpire(u32),
}

#[derive(Clone)]
pub struct IterResult {
    pub length: f64,
    pub damage: Vec<f64>,
    pub casts: Vec<u64>,
    pub hits: Vec<u64>,
    pub crits: Vec<u64>,
    pub noncrit_damage: Vec<f64>,
    /// Resource gained per source (all resources summed), and the part lost to the cap.
    pub res_gain: Vec<f64>,
    pub res_waste: Vec<f64>,
}

struct State<'a> {
    c: &'a Compiled,
    rng: Streams,
    now: f64,
    end: f64,
    res: Vec<f64>,
    charges: Vec<u32>,
    recharge_at: Vec<f64>,
    cd_len: Vec<f64>,
    buff_exp: Vec<f64>,
    buff_stk: Vec<u32>,
    /// Per-stack expiry times for `overlap` buffs.
    ov_exp: Vec<Vec<f64>>,
    stored: Vec<f64>,
    dot_exp: Vec<f64>,
    dot_gen: Vec<u32>,
    dot_last: Vec<f64>,
    /// Damage waiting to be dealt by banked DoTs.
    bank: Vec<f64>,
    gcd_ready: f64,
    /// The player is casting until this time.
    cast_end: f64,
    q: BinaryHeap<Reverse<(u64, u64, Ev)>>,
    seq: u64,
    depth: u32,
    /// Stack of result rows currently executing (for resource attribution).
    cur: Vec<usize>,
    out: IterResult,
    log: bool,
}

impl<'a> Env for State<'a> {
    fn var(&self, v: &Var) -> f64 {
        let b = |x: bool| if x { 1.0 } else { 0.0 };
        match *v {
            Var::Resource(i) => self.res[i],
            Var::Gcd => self.gcd_len(1.5, true),
            Var::Time => self.now,
            Var::FightRemains => (self.end - self.now).max(0.0),
            Var::HealthPct => self.health_pct(),
            Var::CdReady(i) => b(self.charges_now(i) > 0),
            Var::CdRemains(i) => self.cd_remains(i),
            Var::CdCharges(i) => self.charges_now(i) as f64,
            Var::CdChargesFrac(i) => {
                let n = self.charges_now(i);
                let max = self.c.abilities[i].def.charges;
                if n >= max || self.recharge_at[i] == INF {
                    n as f64
                } else {
                    n as f64 + 1.0 - ((self.recharge_at[i] - self.now) / self.cd_len[i]).clamp(0.0, 1.0)
                }
            }
            Var::BuffUp(i) => b(self.buff_up(i)),
            Var::BuffRemains(i) => if self.buff_up(i) { (self.buff_exp[i] - self.now).max(0.0) } else { 0.0 },
            Var::BuffStack(i) => self.stacks(i),
            Var::DotTicking(i) => b(self.dot_exp[i] > self.now),
            Var::DotRemains(i) => (self.dot_exp[i] - self.now).max(0.0),
            Var::Const(x) => x,
        }
    }
}

fn us(t: f64) -> u64 { (t.max(0.0) * 1e6).round() as u64 }

impl<'a> State<'a> {
    fn new(c: &'a Compiled, seed: u64, log: bool) -> Self {
        let mut rng = Streams::new(seed, &c.stream_keys);
        let f = &c.spec.fight;
        let end = f.duration * (1.0 + rng.range(FIGHT_STREAM, -f.variance, f.variance));
        let n_src = c.names.len();
        let na = c.abilities.len();
        State {
            c,
            rng,
            now: 0.0,
            end,
            res: c.res_start.clone(),
            charges: c.abilities.iter().map(|a| a.def.charges).collect(),
            recharge_at: vec![INF; na],
            cd_len: vec![0.0; na],
            buff_exp: vec![-1.0; c.buffs.len()],
            buff_stk: vec![0; c.buffs.len()],
            ov_exp: vec![Vec::new(); c.buffs.len()],
            stored: vec![0.0; c.buffs.len()],
            dot_exp: vec![-1.0; c.dots.len()],
            dot_gen: vec![0; c.dots.len()],
            dot_last: vec![0.0; c.dots.len()],
            bank: vec![0.0; c.dots.len()],
            gcd_ready: 0.0,
            cast_end: 0.0,
            q: BinaryHeap::new(),
            seq: 0,
            depth: 0,
            out: IterResult {
                length: end,
                damage: vec![0.0; n_src],
                casts: vec![0; n_src],
                hits: vec![0; n_src],
                crits: vec![0; n_src],
                noncrit_damage: vec![0.0; n_src],
                res_gain: vec![0.0; n_src],
                res_waste: vec![0.0; n_src],
            },
            cur: Vec::new(),
            log,
        }
    }

    fn push(&mut self, t: f64, e: Ev) {
        self.seq += 1;
        self.q.push(Reverse((us(t), self.seq, e)));
    }

    /// Target health in percent, falling linearly to 0 at the end of the fight.
    fn health_pct(&self) -> f64 { 100.0 * (1.0 - self.now / self.end).max(0.0) }

    fn buff_up(&self, i: usize) -> bool {
        match self.c.buff_kind[i] {
            BuffKind::Shared => self.buff_exp[i] > self.now,
            BuffKind::Overlap => self.ov_exp[i].iter().any(|&t| t > self.now),
            BuffKind::Health(lo, hi) => {
                let h = self.health_pct();
                h > lo && h <= hi
            }
        }
    }

    fn stacks(&self, i: usize) -> f64 {
        match self.c.buff_kind[i] {
            BuffKind::Shared => if self.buff_exp[i] > self.now { self.buff_stk[i] as f64 } else { 0.0 },
            BuffKind::Overlap => self.ov_exp[i].iter().filter(|&&t| t > self.now).count() as f64,
            BuffKind::Health(..) => if self.buff_up(i) { 1.0 } else { 0.0 },
        }
    }

    /// Sum of `pct x stacks` over a list of (buff, pct) modifiers.
    fn buff_sum(&self, mods: &[(usize, f64)]) -> f64 {
        mods.iter().map(|&(b, p)| p * self.stacks(b)).sum()
    }

    fn mastery(&self) -> f64 { self.c.spec.player.mastery + self.buff_sum(&self.c.mastery_buffs) }

    fn haste_mult(&self) -> f64 {
        let mut h = 1.0 + self.c.spec.player.haste;
        for &(b, p) in &self.c.haste_buffs {
            h *= 1.0 + p * self.stacks(b);
        }
        h
    }

    fn gcd_len(&self, base: f64, hasted: bool) -> f64 {
        if !hasted {
            return base;
        }
        let mut g = self.haste_mult();
        for &(b, p) in &self.c.gcd_buffs {
            g *= 1.0 + p * self.stacks(b);
        }
        (base / g).max(0.75)
    }

    fn cast_time(&self, i: usize) -> f64 {
        let d = &self.c.abilities[i].def;
        if d.cast_hasted { d.cast_time / self.haste_mult() } else { d.cast_time }
    }

    fn charges_now(&self, i: usize) -> u32 {
        let max = self.c.abilities[i].def.charges;
        let mut n = self.charges[i];
        let mut t = self.recharge_at[i];
        while n < max && t <= self.now + 1e-9 {
            n += 1;
            t += self.cd_len[i];
        }
        n
    }

    fn cd_length(&self, i: usize) -> f64 {
        let d = &self.c.abilities[i].def;
        let mut len = if d.cooldown_hasted { d.cooldown / self.haste_mult() } else { d.cooldown };
        for &(b, p) in &self.c.abilities[i].cd_rate {
            len /= 1.0 + p * self.stacks(b);
        }
        len
    }

    fn sync_cd(&mut self, i: usize) {
        let max = self.c.abilities[i].def.charges;
        while self.charges[i] < max && self.recharge_at[i] <= self.now + 1e-9 {
            self.charges[i] += 1;
            if self.charges[i] < max {
                // next charge recharges at the rate in effect now
                self.cd_len[i] = self.cd_length(i);
                self.recharge_at[i] += self.cd_len[i];
            } else {
                self.recharge_at[i] = INF;
            }
        }
    }

    fn cd_remains(&self, i: usize) -> f64 {
        if self.charges_now(i) > 0 { 0.0 } else { (self.recharge_at[i] - self.now).max(0.0) }
    }

    fn start_cd(&mut self, i: usize) {
        let d = &self.c.abilities[i].def;
        if d.cooldown <= 0.0 {
            return;
        }
        self.sync_cd(i);
        let len = self.cd_length(i);
        let _ = d;
        self.charges[i] = self.charges[i].saturating_sub(1);
        if self.recharge_at[i] == INF {
            self.cd_len[i] = len;
            self.recharge_at[i] = self.now + len;
        }
    }

    fn usable(&self, i: usize) -> bool {
        let a = &self.c.abilities[i];
        if a.def.triggered_only {
            return false;
        }
        if a.def.cooldown > 0.0 && self.charges_now(i) == 0 {
            return false;
        }
        if let Some(b) = a.requires_buff
            && !self.buff_up(b) {
                return false;
            }
        let free = a.free_with.is_some_and(|b| self.buff_up(b));
        if !free {
            for &(r, amt) in &a.cost {
                if self.res[r] + 1e-9 < amt {
                    return false;
                }
            }
        }
        true
    }

    fn gain(&mut self, r: usize, amt: f64) {
        let before = self.res[r];
        self.res[r] = (before + amt).min(self.c.res_max[r]);
        if let Some(&src) = self.cur.last() {
            self.out.res_gain[src] += amt;
            self.out.res_waste[src] += amt - (self.res[r] - before);
        }
    }

    fn roll(&mut self, rid: usize, chance: f64) -> bool {
        chance >= 1.0 || self.rng.f64(self.c.roll_stream(rid)) < chance
    }

    /// Schedule the natural-expiry check for a buff with `on_expire` effects.
    fn watch_expiry(&mut self, buff: usize) {
        if !self.c.on_expire[buff].is_empty() || !self.c.rate_users[buff].is_empty() {
            self.push(self.buff_exp[buff], Ev::BuffExpire(buff as u32));
        }
    }

    /// A recharge-rate buff went from `k0` to `k1` stacks: recharges in
    /// progress speed up or slow down from now on.
    fn rescale_recharges(&mut self, buff: usize, k0: f64, k1: f64) {
        if k0 == k1 {
            return;
        }
        for &(i, p) in &self.c.rate_users[buff] {
            if self.recharge_at[i] != INF {
                let f = (1.0 + p * k0) / (1.0 + p * k1);
                self.recharge_at[i] = self.now + (self.recharge_at[i] - self.now) * f;
                self.cd_len[i] *= f;
                self.sync_cd(i);
            }
        }
    }

    fn apply(&mut self, e: &CEffect) {
        match *e {
            CEffect::Buff { buff, stacks, chance, rid } => {
                if !self.roll(rid, chance) {
                    return;
                }
                let k0 = self.stacks(buff);
                let def = &self.c.buffs[buff];
                if self.c.buff_kind[buff] == BuffKind::Overlap {
                    let now = self.now;
                    let v = &mut self.ov_exp[buff];
                    v.retain(|&t| t > now);
                    for _ in 0..stacks {
                        if v.len() as u32 >= def.max_stacks {
                            // replace the stack closest to running out
                            let k = (0..v.len()).min_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
                            v.swap_remove(k);
                        }
                        v.push(now + def.duration);
                    }
                    self.buff_exp[buff] = now + def.duration;
                } else {
                    let cur = if self.buff_up(buff) { self.buff_stk[buff] } else { 0 };
                    if cur == 0 {
                        self.stored[buff] = 0.0;
                    }
                    self.buff_stk[buff] = (cur + stacks).min(def.max_stacks);
                    self.buff_exp[buff] = self.now + def.duration;
                    self.watch_expiry(buff);
                }
                if self.log {
                    println!("{:8.3} buff {} -> {}", self.now, def.name, self.stacks(buff));
                }
                self.rescale_recharges(buff, k0, self.stacks(buff));
            }
            CEffect::Consume { buff, all } => {
                if !self.buff_up(buff) {
                    return;
                }
                let k0 = self.stacks(buff);
                if self.c.buff_kind[buff] == BuffKind::Overlap {
                    let now = self.now;
                    let v = &mut self.ov_exp[buff];
                    v.retain(|&t| t > now);
                    if all {
                        v.clear();
                    } else if let Some(k) = (0..v.len()).min_by(|&a, &b| v[a].total_cmp(&v[b])) {
                        v.swap_remove(k);
                    }
                } else if all || self.buff_stk[buff] <= 1 {
                    self.buff_stk[buff] = 0;
                    self.buff_exp[buff] = self.now;
                } else {
                    self.buff_stk[buff] -= 1;
                }
                self.rescale_recharges(buff, k0, self.stacks(buff));
            }
            CEffect::Reset { ab, chance, rid } => {
                if !self.roll(rid, chance) {
                    return;
                }
                self.sync_cd(ab);
                let max = self.c.abilities[ab].def.charges;
                if self.charges[ab] < max {
                    self.charges[ab] += 1;
                    self.recharge_at[ab] =
                        if self.charges[ab] < max { self.now + self.cd_len[ab] } else { INF };
                }
                if self.log {
                    println!("{:8.3} reset {}", self.now, self.c.abilities[ab].def.name);
                }
            }
            CEffect::Reduce { ab, secs, chance, rid } => {
                if !self.roll(rid, chance) {
                    return;
                }
                if self.recharge_at[ab] != INF {
                    self.recharge_at[ab] -= secs;
                    self.sync_cd(ab);
                }
            }
            CEffect::Gain { res, amt, chance, rid } => {
                if !self.roll(rid, chance) {
                    return;
                }
                self.gain(res, amt);
            }
            CEffect::ExtendDot { dot, secs } => {
                if self.dot_exp[dot] > self.now {
                    self.dot_exp[dot] += secs;
                }
            }
            CEffect::ExtendBuff { buff, secs } => {
                if !self.buff_up(buff) {
                    return;
                }
                match self.c.buff_kind[buff] {
                    BuffKind::Overlap => self.ov_exp[buff].iter_mut().for_each(|t| *t += secs),
                    BuffKind::Shared => {
                        self.buff_exp[buff] += secs;
                        self.watch_expiry(buff);
                    }
                    BuffKind::Health(..) => {}
                }
            }
            CEffect::If { ref cond, ref then, ref otherwise } => {
                let list = if cond.eval(self) != 0.0 { then } else { otherwise };
                for e in list {
                    self.apply(e);
                }
            }
            CEffect::Trigger { ab, chance, delay, mastery_chance, rid } => {
                let chance = match mastery_chance {
                    Some(f) => self.mastery() * f,
                    None => chance,
                };
                if !self.roll(rid, chance) {
                    return;
                }
                if delay > 0.0 {
                    self.push(self.now + delay, Ev::Trigger(ab as u32));
                } else {
                    self.execute(ab, true);
                }
            }
        }
    }

    fn crit_chance(&self, bonus: f64, mods: &[(usize, f64)]) -> f64 {
        let mut c = self.c.spec.player.crit + bonus;
        for &(b, p) in &self.c.crit_buffs {
            c += p * self.stacks(b);
        }
        c + self.buff_sum(mods)
    }

    /// Book one hit of final damage `dmg` against result row `src`.
    fn record(&mut self, src: usize, dmg: f64, crit: bool) {
        self.out.damage[src] += dmg;
        self.out.hits[src] += 1;
        for &(b, pct) in &self.c.store_buffs {
            if self.buff_exp[b] > self.now - 1e-9 {
                self.stored[b] += dmg * pct;
            }
        }
        if crit {
            self.out.crits[src] += 1;
        } else {
            self.out.noncrit_damage[src] += dmg;
        }
        if self.log {
            println!("{:8.3}   {} hits for {:.0}{}", self.now, self.c.names[src], dmg, if crit { " (crit)" } else { "" });
        }
    }

    /// Shared damage roll for every source. Returns whether it crit, and the damage.
    /// `raw`: base is already final damage (e.g. a stored-damage burst) —
    /// skip versatility, armor and buff multipliers.
    #[allow(clippy::too_many_arguments)]
    fn hit(&mut self, src: usize, base: f64, mult: f64, mods: &[(usize, f64)], crit_mods: &[(usize, f64)], physical: bool, crit_bonus: f64, crit_dmg: f64, raw: bool) -> (bool, f64) {
        let p = &self.c.spec.player;
        let mut m = mult;
        if !raw {
            m *= p.global_mult * (1.0 + p.versatility);
            if physical {
                m *= self.c.spec.fight.physical_mult;
            }
            for &(b, pct) in mods {
                m *= 1.0 + pct * self.stacks(b);
            }
        }
        let crit = self.rng.f64(self.c.crit_stream(src)) < self.crit_chance(crit_bonus, crit_mods);
        let cm = p.crit_multiplier + crit_dmg + self.buff_sum(&self.c.crit_dmg_buffs);
        let dmg = base * m * if crit { cm } else { 1.0 };
        self.record(src, dmg, crit);
        (crit, dmg)
    }

    fn power_mult(&self) -> f64 { 1.0 + self.buff_sum(&self.c.power_buffs) }

    fn ap(&self) -> f64 {
        let p = &self.c.spec.player;
        (p.attack_power + p.weapon_attack_power) * self.power_mult()
    }

    fn sp(&self) -> f64 { self.c.spec.player.spell_power * self.power_mult() }

    fn weapon_roll(&mut self, src: usize) -> f64 {
        match &self.c.spec.player.weapon {
            Some(w) => if w.max > w.min { self.rng.range(self.c.range_stream(src), w.min, w.max) } else { w.min },
            None => 0.0,
        }
    }

    fn tick_len(&self, def: &DotDef) -> f64 {
        if def.hasted { def.tick_time / self.haste_mult() } else { def.tick_time }
    }

    fn start_ticking(&mut self, di: usize) {
        self.dot_gen[di] += 1;
        self.dot_last[di] = self.now;
        let g = self.dot_gen[di];
        let t = self.tick_len(&self.c.dots[di].def);
        self.push(self.now + t, Ev::DotTick(di as u32, g));
    }

    fn apply_dot(&mut self, di: usize) {
        let c = self.c;
        let def = &c.dots[di].def;
        let remaining = (self.dot_exp[di] - self.now).max(0.0);
        let was = remaining > 0.0;
        self.dot_exp[di] = self.now
            + def.duration
            + match def.refresh.as_str() {
                "extend" => remaining,
                "pandemic" => remaining.min(0.3 * def.duration),
                _ => 0.0,
            };
        if !was {
            if def.tick_on_apply {
                self.dot_tick(di, 1.0);
            }
            self.start_ticking(di);
        }
    }

    /// Add damage to a banked DoT and (re)start its full duration.
    fn feed(&mut self, di: usize, amount: f64) {
        let was = self.dot_exp[di] > self.now;
        self.bank[di] += amount;
        self.dot_exp[di] = self.now + self.c.dots[di].def.duration;
        if !was {
            self.start_ticking(di);
        }
    }

    /// One DoT tick; `frac` < 1 for the shortened final tick.
    fn dot_tick(&mut self, di: usize, frac: f64) {
        let c = self.c;
        let d = &c.dots[di];
        if d.def.banked {
            // spread what's banked evenly over the time left
            let share = ((self.now - self.dot_last[di]) / (self.dot_exp[di] - self.dot_last[di])).clamp(0.0, 1.0);
            let part = self.bank[di] * share;
            self.bank[di] -= part;
            let mut m = d.def.mult;
            for &(b, pct) in &d.buff_mods {
                m *= 1.0 + pct * self.stacks(b);
            }
            self.record(d.src, part * m, false);
        } else if d.def.ap_coeff != 0.0 || d.def.sp_coeff != 0.0 {
            let mastery = if d.def.mastery { 1.0 + self.mastery() } else { 1.0 };
            let base = (d.def.ap_coeff * self.ap() + d.def.sp_coeff * self.sp()) * frac;
            let (crit, _) = self.hit(d.src, base, d.def.mult * mastery, &d.buff_mods, &d.crit_mods, d.physical, d.def.crit_bonus, d.def.crit_damage_bonus, false);
            if crit && d.def.extend_on_crit > 0.0 && self.dot_exp[di] > self.now {
                self.dot_exp[di] += d.def.extend_on_crit;
            }
        }
        for e in &d.on_tick {
            self.apply(e);
        }
    }

    /// Execute an ability. `triggered`: fired by another effect — no cost,
    /// no GCD, and silently skipped if its buff requirement or cooldown isn't met.
    fn execute(&mut self, i: usize, triggered: bool) {
        let c = self.c;
        let a = &c.abilities[i];
        if triggered {
            if self.depth >= MAX_TRIGGER_DEPTH {
                return;
            }
            if let Some(b) = a.requires_buff
                && !self.buff_up(b) {
                    return;
                }
            if a.def.cooldown > 0.0 && self.charges_now(i) == 0 {
                return;
            }
        }
        if self.log {
            let res: Vec<String> = c.res_names.iter().zip(&self.res).map(|(n, v)| format!("{n}={v}")).collect();
            println!("{:8.3} {} {} [{}]", self.now, if triggered { "trigger" } else { "cast" }, a.def.name, res.join(" "));
        }
        self.depth += 1;
        self.cur.push(a.src);
        if !triggered {
            let free = a.free_with.is_some_and(|b| self.buff_up(b));
            if free {
                self.apply(&CEffect::Consume { buff: a.free_with.unwrap(), all: false });
            } else {
                for &(r, amt) in &a.cost {
                    self.res[r] -= amt;
                }
            }
        }
        for &(r, amt) in &a.gain {
            self.gain(r, amt);
        }
        self.start_cd(i);
        for e in &a.on_start {
            self.apply(e);
        }
        if let Some(d) = &a.def.damage {
            let (ap, sp) = (self.ap(), self.sp());
            for _ in 0..d.hits {
                let w = if d.weapon_mult > 0.0 { self.weapon_roll(a.src) * d.weapon_mult } else { 0.0 };
                let base = match a.stored_from {
                    Some(b) => std::mem::take(&mut self.stored[b]),
                    None => d.ap_coeff * ap + d.sp_coeff * sp + w,
                };
                let raw = a.stored_from.is_some();
                let mastery = if d.mastery && !raw { 1.0 + self.mastery() } else { 1.0 };
                let (crit, dmg) = self.hit(a.src, base, d.mult * mastery, &a.buff_mods, &a.crit_mods, a.physical, d.crit_bonus, d.crit_damage_bonus, raw);
                for &(di, pct, per_mastery) in &a.feeds {
                    let share = pct + per_mastery * self.mastery();
                    self.feed(di, dmg * share);
                }
                for e in if crit { &a.on_crit } else { &a.on_noncrit } {
                    self.apply(e);
                }
            }
        }
        if let Some(di) = a.dot {
            self.apply_dot(di);
        }
        for e in &a.on_cast {
            self.apply(e);
        }
        self.out.casts[a.src] += 1;
        self.cur.pop();
        self.depth -= 1;
    }

    fn choose(&self) -> Option<usize> {
        for (i, cond) in &self.c.rotation {
            if !self.usable(*i) {
                continue;
            }
            if let Some(e) = cond
                && e.eval(self) == 0.0 {
                    continue;
                }
            return Some(*i);
        }
        None
    }

    fn schedule_proc(&mut self, k: usize) {
        let (_, ppm, hasted) = self.c.procs[k];
        let rate = ppm / 60.0 * if hasted { self.haste_mult() } else { 1.0 };
        let dt = -(1.0 - self.rng.f64(self.c.proc_stream(k))).ln() / rate;
        self.push(self.now + dt, Ev::Proc(k as u32));
    }

    fn run(mut self) -> IterResult {
        let c = self.c;
        for &i in &c.precombat {
            self.execute(i, false);
        }
        self.push(0.0, Ev::Ready);
        if c.spec.player.weapon.is_some() {
            self.push(0.0, Ev::Auto);
        }
        for k in 0..c.procs.len() {
            self.schedule_proc(k);
        }
        let auto_src = c.names.len() - 1;

        while let Some(Reverse((t, _, ev))) = self.q.pop() {
            self.now = t as f64 / 1e6;
            if self.now >= self.end {
                break;
            }
            match ev {
                Ev::Auto => {
                    match c.auto_replace {
                        Some(i) => self.execute(i, true),
                        None => {
                            let base = self.weapon_roll(auto_src);
                            self.hit(auto_src, base, c.auto_mult, &c.auto_mods, &[], true, 0.0, 0.0, false);
                            self.out.casts[auto_src] += 1;
                        }
                    }
                    for e in &c.auto_on_hit {
                        self.apply(e);
                    }
                    let speed = c.spec.player.weapon.as_ref().unwrap().speed / self.haste_mult();
                    self.push(self.now + speed, Ev::Auto);
                    if self.gcd_ready <= self.now {
                        self.push(self.now, Ev::Ready); // a proc may have made something castable
                    }
                }
                Ev::DotTick(di, g) => {
                    let di = di as usize;
                    if g != self.dot_gen[di] || self.dot_exp[di] < self.now - 1e-6 {
                        continue;
                    }
                    let d = &c.dots[di].def;
                    let tick = self.tick_len(d);
                    let frac = ((self.now - self.dot_last[di]) / tick).clamp(0.0, 1.0);
                    self.dot_tick(di, frac);
                    self.dot_last[di] = self.now;
                    if self.now + tick <= self.dot_exp[di] + 1e-6 {
                        self.push(self.now + tick, Ev::DotTick(di as u32, g));
                    } else if self.dot_exp[di] > self.now + 1e-6 {
                        // shortened final tick at expiry, like the real game
                        self.push(self.dot_exp[di], Ev::DotTick(di as u32, g));
                    }
                }
                Ev::Trigger(i) => self.execute(i as usize, true),
                Ev::Proc(k) => {
                    let k = k as usize;
                    self.execute(c.procs[k].0, true);
                    self.schedule_proc(k);
                }
                Ev::CastEnd(i) => {
                    self.execute(i as usize, false);
                    self.push(self.gcd_ready.max(self.now), Ev::Ready);
                }
                Ev::BuffExpire(b) => {
                    let b = b as usize;
                    // stale if the buff was refreshed, extended or consumed since
                    if (self.buff_exp[b] - self.now).abs() < 1e-6 && self.buff_stk[b] > 0 {
                        let k0 = self.buff_stk[b] as f64;
                        self.buff_stk[b] = 0;
                        self.rescale_recharges(b, k0, 0.0);
                        if self.log {
                            println!("{:8.3} buff {} expires", self.now, c.buffs[b].name);
                        }
                        for e in &c.on_expire[b] {
                            self.apply(e);
                        }
                    }
                }
                Ev::Ready => {
                    if self.now + 2e-6 < self.gcd_ready || self.now + 2e-6 < self.cast_end {
                        continue;
                    }
                    match self.choose() {
                        Some(i) => {
                            let d = &c.abilities[i].def;
                            let ct = self.cast_time(i);
                            if ct > 0.0 {
                                // the GCD runs from the start of the cast; the spell resolves at its end
                                if d.gcd > 0.0 {
                                    self.gcd_ready = self.now + self.gcd_len(d.gcd, d.gcd_hasted);
                                }
                                if self.log {
                                    println!("{:8.3} begin cast {} ({:.3}s)", self.now, d.name, ct);
                                }
                                self.cast_end = self.now + ct;
                                self.push(self.cast_end, Ev::CastEnd(i as u32));
                            } else {
                                self.execute(i, false);
                                if d.gcd > 0.0 {
                                    self.gcd_ready = self.now + self.gcd_len(d.gcd, d.gcd_hasted);
                                }
                                self.push(self.gcd_ready.max(self.now), Ev::Ready);
                            }
                        }
                        None => {
                            // idle until the next cooldown comes up (or re-check shortly)
                            let mut next = self.now + 0.5;
                            for (i, _) in &c.rotation {
                                let r = self.cd_remains(*i);
                                if r > 0.0 {
                                    next = next.min(self.now + r);
                                }
                            }
                            self.push(next.max(self.now + 0.01), Ev::Ready);
                        }
                    }
                }
            }
        }
        self.out
    }
}

pub fn run_iteration(c: &Compiled, seed: u64, log: bool) -> IterResult {
    State::new(c, seed, log).run()
}
