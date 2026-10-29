//! Event-driven combat engine. Knows nothing about any class: everything
//! comes from the compiled spec.

use crate::expr::{self, Env, Expr, Resolver, Var};
use crate::spec::*;
use std::cmp::Ordering;
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
    Buff { buff: usize, stacks: u32, chance: f64, dur: Option<f64>, set: Option<Expr>, rid: usize },
    Consume { buff: usize, all: bool },
    Reset { ab: usize, chance: f64, rid: usize },
    Reduce { ab: usize, secs: f64, chance: f64, rid: usize },
    Gain { res: usize, amt: f64, chance: f64, rid: usize },
    Trigger { ab: usize, chance: f64, delay: f64, mastery_chance: Option<f64>, rid: usize },
    ExtendDot { dot: usize, secs: f64 },
    DotStack { dot: usize, add: i32, set: Option<Expr>, min: u32 },
    CancelDot { dot: usize },
    ExtendBuff { buff: usize, secs: f64 },
    If { cond: Expr, then: Vec<CEffect>, otherwise: Vec<CEffect> },
    Summon { pet: usize, count: u32, dur: Option<f64> },
    Dismiss { pet: usize, count: u32, each: Vec<CEffect> },
    ExtendPet { pet: usize, secs: f64 },
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
    on_free: Vec<CEffect>,
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
    /// (buff, fraction per stack) cast time reductions.
    cast_mods: Vec<(usize, f64)>,
    physical: bool,
    /// Pet that casts this ability, and its index among the pet's abilities.
    pet: Option<usize>,
    pet_slot: usize,
    /// Pet energy spent per cast.
    pet_cost: f64,
    /// Damage multiplier expression (`damage.scale`).
    scale: Option<Expr>,
}

#[derive(Debug, Clone)]
struct CPet {
    duration: f64,
    max_active: u32,
    coeff: OwnerCoeff,
    inherit_haste: bool,
    inherit_crit: bool,
    crit: f64,
    energy: f64,
    /// Cheapest action's energy cost: an instance with less leaves (0: never).
    min_cost: f64,
    delay: f64,
    /// (ability, swing time, hasted)
    melee: Option<(usize, f64, bool)>,
    /// Priority list: (ability, condition).
    actions: Vec<(usize, Option<Expr>)>,
    n_abilities: usize,
    on_summon: Vec<CEffect>,
    on_expire: Vec<CEffect>,
}

#[derive(Debug, Clone)]
struct CDot {
    def: DotDef,
    on_tick: Vec<CEffect>,
    on_crit: Vec<CEffect>,
    buff_mods: Vec<(usize, f64)>,
    crit_mods: Vec<(usize, f64)>,
    physical: bool,
    src: usize,
    pet: Option<usize>,
}

/// A proc rolled on the player's direct hits (see `ProcDef::on_hit`).
#[derive(Debug, Clone)]
struct HitProc {
    ab: usize,
    rppm: f64,
    hasted: bool,
    chance: f64,
    icd: f64,
}

/// SimC's real PPM: the time counted per attempt is capped at this.
const RPPM_MAX_INTERVAL: f64 = 3.5;
/// Bad-luck protection looks back at most this long.
const RPPM_MAX_BLP: f64 = 1000.0;

#[derive(Debug, Clone)]
pub struct Compiled {
    pub spec: Spec,
    res_names: Vec<String>,
    res_max: Vec<f64>,
    res_start: Vec<f64>,
    abilities: Vec<CAbility>,
    buffs: Vec<BuffDef>,
    dots: Vec<CDot>,
    pets: Vec<CPet>,
    pet_names: Vec<String>,
    rotation: Vec<(usize, Option<Expr>)>,
    precombat: Vec<usize>,
    procs: Vec<(usize, f64, bool)>,
    hit_procs: Vec<HitProc>,
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
    vers_buffs: Vec<(usize, f64)>,
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
    pet: &'a HashMap<String, usize>,
    /// Inside a pet: its name, so its own abilities and DoTs resolve by their
    /// local names first.
    scope: &'a str,
}

impl Names<'_> {
    fn scoped(&self, m: &HashMap<String, usize>, kind: &str, n: &str) -> Result<usize, String> {
        if !self.scope.is_empty()
            && let Some(&i) = m.get(&format!("{}_{n}", self.scope)) {
                return Ok(i);
            }
        get(m, kind, n)
    }
    fn ab(&self, n: &str) -> Result<usize, String> { self.scoped(self.ab, "ability", n) }
    fn dot(&self, n: &str) -> Result<usize, String> { self.scoped(self.dot, "dot", n) }
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
                let i = self.ab(a)?;
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
                let i = self.dot(dn)?;
                match *f {
                    "ticking" | "up" => Var::DotTicking(i),
                    "remains" => Var::DotRemains(i),
                    "stack" | "stacks" => Var::DotStack(i),
                    _ => return Err(format!("unknown dot field '{f}'")),
                }
            }
            ["pet", pn, f] => {
                let i = get(self.pet, "pet", pn)?;
                match *f {
                    "active" | "count" | "up" => Var::PetActive(i),
                    "remains" => Var::PetRemains(i),
                    _ => return Err(format!("unknown pet field '{f}'")),
                }
            }
            ["self", "energy"] => Var::SelfEnergy,
            ["self", "remains"] => Var::SelfRemains,
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
        Effect::Buff { buff: b, stacks, chance, duration, set } => CEffect::Buff {
            buff: get(n.buff, "buff", b)?,
            stacks: *stacks,
            chance: *chance,
            dur: *duration,
            set: set.as_ref().map(|e| expr::parse(e, n).map_err(|err| format!("buff set '{e}': {err}"))).transpose()?,
            rid: rid(),
        },
        Effect::Consume { buff: b, all } => CEffect::Consume { buff: get(n.buff, "buff", b)?, all: *all },
        Effect::ResetCooldown { ability, chance } => {
            CEffect::Reset { ab: n.ab(ability)?, chance: *chance, rid: rid() }
        }
        Effect::ReduceCooldown { ability, seconds, chance } => {
            CEffect::Reduce { ab: n.ab(ability)?, secs: *seconds, chance: *chance, rid: rid() }
        }
        Effect::Gain { resource, amount, chance } => {
            CEffect::Gain { res: get(n.res, "resource", resource)?, amt: *amount, chance: *chance, rid: rid() }
        }
        Effect::ExtendDot { dot: d, seconds } => CEffect::ExtendDot { dot: n.dot(d)?, secs: *seconds },
        Effect::DotStack { dot: d, stacks, set, min } => CEffect::DotStack {
            dot: n.dot(d)?,
            add: *stacks,
            set: set.as_ref().map(|e| expr::parse(e, n).map_err(|err| format!("dot_stack set '{e}': {err}"))).transpose()?,
            min: *min,
        },
        Effect::CancelDot { dot: d } => CEffect::CancelDot { dot: n.dot(d)? },
        Effect::ExtendBuff { buff: b, seconds } => {
            CEffect::ExtendBuff { buff: get(n.buff, "buff", b)?, secs: *seconds }
        }
        Effect::If { cond, then, otherwise } => CEffect::If {
            cond: expr::parse(cond, n).map_err(|e| format!("if '{cond}': {e}"))?,
            then: compile_effects(then, n, &format!("{site}/then"), rolls)?,
            otherwise: compile_effects(otherwise, n, &format!("{site}/else"), rolls)?,
        },
        Effect::Summon { pet, count, duration } => {
            CEffect::Summon { pet: get(n.pet, "pet", pet)?, count: *count, dur: *duration }
        }
        Effect::Dismiss { pet, count, each } => CEffect::Dismiss {
            pet: get(n.pet, "pet", pet)?,
            count: *count,
            each: compile_effects(each, n, &format!("{site}/each"), rolls)?,
        },
        Effect::ExtendPet { pet, seconds } => CEffect::ExtendPet { pet: get(n.pet, "pet", pet)?, secs: *seconds },
        Effect::Trigger { ability, chance, delay, mastery_chance } => CEffect::Trigger {
            ab: n.ab(ability)?,
            chance: *chance,
            delay: *delay,
            mastery_chance: *mastery_chance,
            rid: rid(),
        },
    })
}

/// `pet`: the pet dealing the damage. Owner-wide bonuses reach pets only
/// from buffs flagged `pets`; `pet_pct` adds per-pet ones.
fn buff_mods(buffs: &[BuffDef], ability: &str, source: &str, school: &str, pet: Option<&str>) -> Vec<(usize, f64)> {
    let mut v = Vec::new();
    for (i, b) in buffs.iter().enumerate() {
        let owner = if pet.is_none() || b.pets {
            b.damage_pct + b.school_pct.get(school).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut pct = owner + keyed(&b.ability_pct, ability, source);
        if let Some(p) = pet {
            pct += b.pet_pct.get(p).copied().unwrap_or(0.0);
        }
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
        // Owner abilities, then every pet's (renamed `<pet>_<ability>`).
        let mut all: Vec<(AbilityDef, Option<usize>)> = spec.abilities.iter().map(|a| (a.clone(), None)).collect();
        for (pi, p) in spec.pets.iter().enumerate() {
            all.extend(p.abilities.iter().map(|a| (pet_ability(&p.name, a), Some(pi))));
        }
        let ab: HashMap<String, usize> =
            all.iter().enumerate().map(|(i, (a, _))| (a.name.clone(), i)).collect();
        if ab.len() != all.len() {
            return Err("duplicate ability names".into());
        }
        let pet: HashMap<String, usize> = spec.pets.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();
        if pet.len() != spec.pets.len() {
            return Err("duplicate pet names".into());
        }
        let buff: HashMap<String, usize> =
            spec.buffs.iter().enumerate().map(|(i, b)| (b.name.clone(), i)).collect();
        let mut dot = HashMap::new();
        for (a, _) in &all {
            if let Some(d) = &a.dot {
                if !["pandemic", "extend", "reset"].contains(&d.refresh.as_str()) {
                    return Err(format!("dot '{}': refresh must be pandemic, extend or reset", d.name));
                }
                let n = dot.len();
                dot.insert(d.name.clone(), n);
            }
        }
        let names = Names { res: &res, ab: &ab, buff: &buff, dot: &dot, pet: &pet, scope: "" };
        let scoped = |pi: Option<usize>| Names { scope: pi.map_or("", |i| spec.pets[i].name.as_str()), ..names };

        let rolls = std::cell::RefCell::new(Vec::<String>::new());
        let effects_in = |v: &[Effect], at: &str, n: &Names| -> Result<Vec<CEffect>, String> {
            compile_effects(v, n, at, &mut rolls.borrow_mut())
        };
        let effects = |v: &[Effect], at: &str| effects_in(v, at, &names);
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
        let ab_src: Vec<usize> = all
            .iter()
            .map(|(a, _)| src_of(a.source.as_deref().unwrap_or(&a.name), &mut src_names))
            .collect();
        for (a, _) in &all {
            if let Some(d) = &a.dot {
                src_of(&d.name, &mut src_names);
            }
        }
        src_names.push("auto_attack".into());

        let mut abilities = Vec::new();
        let mut dots = Vec::new();
        for (ai, (a, pi)) in all.iter().enumerate() {
            let pi = *pi;
            let n = scoped(pi);
            let pet_name = pi.map(|i| spec.pets[i].name.as_str());
            let effects = |v: &[Effect], at: &str| effects_in(v, at, &n);
            let school = a.damage.as_ref().map(|d| d.school.as_str()).unwrap_or("physical");
            let ctx = |e: String| format!("ability '{}': {e}", a.name);
            let dot_idx = match &a.dot {
                Some(d) => {
                    dots.push(CDot {
                        on_tick: effects(&d.on_tick, &format!("{}/on_tick", d.name)).map_err(ctx)?,
                        on_crit: effects(&d.on_crit, &format!("{}/on_crit", d.name)).map_err(ctx)?,
                        // banked damage already carries the hit's buffs: only buffs naming the DoT apply
                        buff_mods: if d.banked {
                            per_buff(&spec.buffs, |b| b.ability_pct.get(&d.name).copied().unwrap_or(0.0))
                        } else {
                            buff_mods(&spec.buffs, &d.name, &d.name, &d.school, pet_name)
                        },
                        crit_mods: crit_mods(&spec.buffs, &d.name, &d.name),
                        physical: d.school == "physical",
                        src: src_names.iter().position(|x| *x == d.name).unwrap(),
                        pet: pi,
                        def: d.clone(),
                    });
                    Some(dots.len() - 1)
                }
                None => None,
            };
            let (cost, pet_cost) = match pi {
                None => (rmap(&a.cost).map_err(ctx)?, 0.0),
                Some(_) => {
                    if a.cost.keys().any(|k| k != "energy") {
                        return Err(ctx("a pet ability can only cost the pet's energy".into()));
                    }
                    if a.free_with.is_some() || a.damage.as_ref().is_some_and(|d| d.weapon_mult != 0.0) {
                        return Err(ctx("pet abilities can't use free_with or weapon_mult".into()));
                    }
                    (vec![], a.cost.get("energy").copied().unwrap_or(0.0))
                }
            };
            abilities.push(CAbility {
                pet: pi,
                pet_slot: pi.map_or(0, |i| ai - all.iter().position(|x| x.1 == Some(i)).unwrap()),
                pet_cost,
                scale: a
                    .damage
                    .as_ref()
                    .and_then(|d| d.scale.as_ref())
                    .map(|e| expr::parse(e, &n).map_err(|err| ctx(format!("damage scale '{e}': {err}"))))
                    .transpose()?,
                cost,
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
                on_free: effects(&a.on_free, &format!("{}/on_free", a.name)).map_err(ctx)?,
                src: ab_src[ai],
                feeds: a
                    .damage
                    .iter()
                    .flat_map(|d| &d.feed)
                    .map(|f| Ok((n.dot(&f.dot)?, f.pct / 100.0, f.per_mastery)))
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
                buff_mods: buff_mods(&spec.buffs, &a.name, a.source.as_deref().unwrap_or(&a.name), school, pet_name),
                cast_mods: per_buff(&spec.buffs, |b| keyed(&b.cast_time_pct, &a.name, a.source.as_deref().unwrap_or(&a.name))),
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
            if abilities[i].pet.is_some() {
                return Err(format!("rotation: '{}' is a pet ability", r.action));
            }
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
            .filter(|p| !p.on_hit)
            .map(|p| Ok((get(&ab, "ability", &p.ability)?, p.per_minute, p.hasted)))
            .collect::<Result<_, String>>()?;
        let hit_procs = spec
            .procs
            .iter()
            .filter(|p| p.on_hit)
            .map(|p| {
                Ok(HitProc { ab: get(&ab, "ability", &p.ability)?, rppm: p.per_minute, hasted: p.hasted, chance: p.chance, icd: p.icd })
            })
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

        let mut pets = Vec::new();
        for (pi, p) in spec.pets.iter().enumerate() {
            let n = scoped(Some(pi));
            let ctx = |e: String| format!("pet '{}': {e}", p.name);
            let melee = p
                .melee
                .as_ref()
                .map(|m| Ok::<_, String>((n.ab(&m.ability)?, m.speed, m.hasted)))
                .transpose()
                .map_err(ctx)?;
            let actions: Vec<(usize, Option<Expr>)> = if p.rotation.is_empty() {
                p.abilities
                    .iter()
                    .filter(|a| !a.triggered_only && p.melee.as_ref().is_none_or(|m| m.ability != a.name))
                    .map(|a| Ok((n.ab(&a.name)?, None)))
                    .collect::<Result<_, String>>()
            } else {
                p.rotation
                    .iter()
                    .map(|r| {
                        let c = r.condition.as_ref().map(|s| expr::parse(s, &n).map_err(|e| format!("rotation '{}': {e}", r.action))).transpose()?;
                        Ok((n.ab(&r.action)?, c))
                    })
                    .collect()
            }
            .map_err(ctx)?;
            for (i, _) in &actions {
                let a = &abilities[*i];
                if a.pet != Some(pi) {
                    return Err(ctx(format!("'{}' is not one of its abilities", a.def.name)));
                }
                if a.def.gcd <= 0.0 && a.def.cast_time <= 0.0 && a.def.cooldown <= 0.0 && a.pet_cost <= 0.0 {
                    return Err(ctx(format!("'{}' has no gcd, cast time, cooldown or cost: it would never stop", a.def.name)));
                }
            }
            let min_cost = actions.iter().map(|(i, _)| abilities[*i].pet_cost).filter(|&x| x > 0.0).fold(INF, f64::min);
            if min_cost < INF && p.energy.is_none() {
                return Err(ctx("abilities cost energy but the pet has none".into()));
            }
            pets.push(CPet {
                duration: p.duration,
                max_active: p.max_active,
                coeff: p.owner_coeff.clone(),
                inherit_haste: p.inherit_haste,
                inherit_crit: p.inherit_crit,
                crit: p.crit,
                energy: p.energy.unwrap_or(0.0),
                min_cost: if min_cost < INF { min_cost } else { 0.0 },
                delay: p.delay,
                melee,
                actions,
                n_abilities: p.abilities.len(),
                on_summon: effects_in(&p.on_summon, &format!("pet/{}/on_summon", p.name), &n).map_err(ctx)?,
                on_expire: effects_in(&p.on_expire, &format!("pet/{}/on_expire", p.name), &n).map_err(ctx)?,
            });
        }

        let roll_names = rolls.into_inner();
        let mut stream_names = vec!["fight_length".to_string()];
        stream_names.extend(src_names.iter().map(|n| format!("crit/{n}")));
        stream_names.extend(src_names.iter().map(|n| format!("range/{n}")));
        stream_names.extend(spec.procs.iter().filter(|p| !p.on_hit).map(|p| format!("rppm/{}", p.ability)));
        stream_names.extend(roll_names.iter().map(|n| format!("roll/{n}")));
        stream_names.extend(spec.procs.iter().filter(|p| p.on_hit).map(|p| format!("hit_proc/{}", p.ability)));
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
            auto_mods: buff_mods(&spec.buffs, "auto_attack", "auto_attack", "physical", None),
            buffs: spec.buffs.clone(),
            abilities,
            dots,
            pets,
            pet_names: spec.pets.iter().map(|p| p.name.clone()).collect(),
            rotation,
            precombat,
            procs,
            hit_procs,
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
            vers_buffs: per_buff(&spec.buffs, |b| b.versatility_pct),
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
    // RPPM per proc, one per chance-roll site, then one per on-hit proc.
    fn crit_stream(&self, src: usize) -> usize { 1 + src }
    fn range_stream(&self, src: usize) -> usize { 1 + self.names.len() + src }
    fn proc_stream(&self, k: usize) -> usize { 1 + 2 * self.names.len() + k }
    fn roll_stream(&self, rid: usize) -> usize { 1 + 2 * self.names.len() + self.procs.len() + rid }
    fn hit_proc_stream(&self, k: usize) -> usize { self.stream_keys.len() - self.hit_procs.len() + k }
}

const FIGHT_STREAM: usize = 0;

// ---------- per-iteration state ----------

#[derive(Clone, Copy)]
enum Ev {
    Ready,
    Auto,
    DotTick(u32, u32),
    Trigger(u32),
    Proc(u32),
    CastEnd(u32),
    BuffExpire(u32),
    // (pet key, generation)
    PetAct(u32, u32),
    PetCast(u32, u32),
    PetSwing(u32, u32),
    PetExpire(u32, u32),
}

/// Event queue entry: (time in us, insertion order, event). Ordered by time,
/// then insertion order, earliest first; the order is unique, so the event
/// itself never takes part in comparisons.
struct Queued(u64, u64, Ev);

impl PartialEq for Queued {
    fn eq(&self, o: &Self) -> bool { self.0 == o.0 && self.1 == o.1 }
}
impl Eq for Queued {}
impl PartialOrd for Queued {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}
impl Ord for Queued {
    // reversed: BinaryHeap is a max-heap
    fn cmp(&self, o: &Self) -> Ordering { (o.0, o.1).cmp(&(self.0, self.1)) }
}

#[derive(Clone)]
struct PetInst {
    alive: bool,
    /// Bumped on every summon and departure: events carrying an older value are stale.
    generation: u32,
    exp: f64,
    spawned: f64,
    energy: f64,
    gcd_end: f64,
    /// Ability being cast.
    casting: usize,
    /// Per pet ability: when its cooldown is ready.
    cd: Vec<f64>,
}

/// Event key for a pet instance.
fn pkey(p: usize, s: usize) -> u32 { ((p as u32) << 16) | s as u32 }
fn unkey(k: u32) -> (usize, usize) { ((k >> 16) as usize, (k & 0xFFFF) as usize) }

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
    dot_stk: Vec<u32>,
    /// Damage waiting to be dealt by banked DoTs.
    bank: Vec<f64>,
    /// Per on-hit proc: last attempt, real-PPM time banked since the last
    /// success (bad-luck protection), and when its internal cooldown ends.
    hp_last: Vec<f64>,
    hp_blp: Vec<f64>,
    hp_ready: Vec<f64>,
    /// Instance slots per pet; free slots are reused.
    pets: Vec<Vec<PetInst>>,
    /// Pet whose ability is executing (its crit applies).
    cur_pet: Option<usize>,
    /// (pet, slot) that `self.*` refers to.
    ctx: Option<(usize, usize)>,
    gcd_ready: f64,
    /// The player is casting until this time.
    cast_end: f64,
    q: BinaryHeap<Queued>,
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
            Var::DotStack(i) => if self.dot_exp[i] > self.now { self.dot_stk[i] as f64 } else { 0.0 },
            Var::PetActive(_) | Var::PetRemains(_) | Var::SelfEnergy | Var::SelfRemains => self.pet_var(v),
            Var::Const(x) => x,
        }
    }
}

impl State<'_> {
    #[inline(never)]
    fn pet_var(&self, v: &Var) -> f64 {
        match *v {
            Var::PetActive(p) => (0..self.pets[p].len()).filter(|&s| self.pet_active(p, s)).count() as f64,
            Var::PetRemains(p) => (0..self.pets[p].len())
                .filter(|&s| self.pet_active(p, s))
                .map(|s| self.pets[p][s].exp - self.now)
                .fold(0.0, f64::max),
            Var::SelfEnergy => self.ctx.map_or(0.0, |(p, s)| self.pets[p][s].energy),
            Var::SelfRemains => match self.ctx {
                Some((p, s)) if self.pet_active(p, s) => self.pets[p][s].exp - self.now,
                _ => 0.0,
            },
            _ => unreachable!(),
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
            dot_stk: vec![0; c.dots.len()],
            bank: vec![0.0; c.dots.len()],
            hp_last: vec![0.0; c.hit_procs.len()],
            hp_blp: vec![0.0; c.hit_procs.len()],
            hp_ready: vec![0.0; c.hit_procs.len()],
            pets: vec![Vec::new(); c.pets.len()],
            cur_pet: None,
            ctx: None,
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
        self.q.push(Queued(us(t), self.seq, e));
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

    /// Stacks a buff's bonuses count: at most one for `flat` buffs.
    fn mod_stacks(&self, i: usize) -> f64 {
        if self.c.buffs[i].flat { self.stacks(i).min(1.0) } else { self.stacks(i) }
    }

    /// Sum of `pct x stacks` over a list of (buff, pct) modifiers.
    fn buff_sum(&self, mods: &[(usize, f64)]) -> f64 {
        mods.iter().map(|&(b, p)| p * self.mod_stacks(b)).sum()
    }

    fn mastery(&self) -> f64 { self.c.spec.player.mastery + self.buff_sum(&self.c.mastery_buffs) }

    fn haste_mult(&self) -> f64 {
        let mut h = 1.0 + self.c.spec.player.haste;
        for &(b, p) in &self.c.haste_buffs {
            h *= 1.0 + p * self.mod_stacks(b);
        }
        h
    }

    fn gcd_len(&self, base: f64, hasted: bool) -> f64 {
        if !hasted {
            return base;
        }
        let mut g = self.haste_mult();
        for &(b, p) in &self.c.gcd_buffs {
            g *= 1.0 + p * self.mod_stacks(b);
        }
        (base / g).max(0.75)
    }

    fn cast_time(&self, i: usize) -> f64 {
        let a = &self.c.abilities[i];
        let mut ct = if a.def.cast_hasted { a.def.cast_time / self.haste_mult() } else { a.def.cast_time };
        for &(b, p) in &a.cast_mods {
            ct *= (1.0 - p * self.mod_stacks(b)).max(0.0);
        }
        ct
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
            len /= 1.0 + p * self.mod_stacks(b);
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
            CEffect::Buff { buff, stacks, chance, dur, ref set, rid } => {
                if !self.roll(rid, chance) {
                    return;
                }
                let set = set.as_ref().map(|e| e.eval(self).round().max(0.0) as u32);
                if set == Some(0) {
                    self.apply(&CEffect::Consume { buff, all: true });
                    return;
                }
                let k0 = self.mod_stacks(buff);
                let def = &self.c.buffs[buff];
                let dur = dur.unwrap_or(def.duration);
                if self.c.buff_kind[buff] == BuffKind::Overlap {
                    let now = self.now;
                    let v = &mut self.ov_exp[buff];
                    v.retain(|&t| t > now);
                    if set.is_some() {
                        v.clear();
                    }
                    for _ in 0..set.unwrap_or(stacks) {
                        if v.len() as u32 >= def.max_stacks {
                            // replace the stack closest to running out
                            let k = (0..v.len()).min_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
                            v.swap_remove(k);
                        }
                        v.push(now + dur);
                    }
                    self.buff_exp[buff] = now + dur;
                } else {
                    let cur = if self.buff_up(buff) { self.buff_stk[buff] } else { 0 };
                    if cur == 0 {
                        self.stored[buff] = 0.0;
                    }
                    self.buff_stk[buff] = set.unwrap_or(cur + stacks).min(def.max_stacks);
                    self.buff_exp[buff] = if def.refresh_extend && cur > 0 { self.buff_exp[buff] + dur } else { self.now + dur };
                    self.watch_expiry(buff);
                }
                if self.log {
                    println!("{:8.3} buff {} -> {}", self.now, def.name, self.stacks(buff));
                }
                self.rescale_recharges(buff, k0, self.mod_stacks(buff));
            }
            CEffect::Consume { buff, all } => {
                if !self.buff_up(buff) {
                    return;
                }
                let k0 = self.mod_stacks(buff);
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
                self.rescale_recharges(buff, k0, self.mod_stacks(buff));
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
            CEffect::DotStack { dot, add, ref set, min } => {
                if self.dot_exp[dot] <= self.now {
                    return;
                }
                let cur = self.dot_stk[dot] as i64;
                let n = match set {
                    Some(e) => e.eval(self).round() as i64,
                    None => (cur + add as i64).max(cur.min(min as i64)),
                };
                let n = n.min(self.c.dots[dot].def.max_stacks as i64);
                if n <= 0 {
                    self.cancel_dot(dot);
                } else {
                    self.dot_stk[dot] = n as u32;
                }
                if self.log {
                    println!("{:8.3} dot {} stacks -> {}", self.now, self.c.dots[dot].def.name, self.dot_stk[dot]);
                }
            }
            CEffect::CancelDot { dot } => self.cancel_dot(dot),
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
            CEffect::Summon { pet, count, dur } => self.summon(pet, count, dur),
            CEffect::Dismiss { pet, count, ref each } => self.dismiss(pet, count, each),
            CEffect::ExtendPet { pet, secs } => self.extend_pet(pet, secs),
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

    #[inline(always)]
    fn crit_chance(&self, bonus: f64, mods: &[(usize, f64)]) -> f64 {
        if let Some(p) = self.cur_pet
            && !self.c.pets[p].inherit_crit {
                return self.c.pets[p].crit + bonus + self.buff_sum(mods);
            }
        let mut c = self.c.spec.player.crit + bonus;
        for &(b, p) in &self.c.crit_buffs {
            c += p * self.mod_stacks(b);
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
    /// skip versatility, armor and buff multipliers. `crit_scaling`: damage
    /// also grows with the player's crit chance.
    #[allow(clippy::too_many_arguments)]
    fn hit(&mut self, src: usize, base: f64, mult: f64, mods: &[(usize, f64)], crit_mods: &[(usize, f64)], physical: bool, crit_bonus: f64, crit_dmg: f64, crit_scaling: bool, raw: bool) -> (bool, f64) {
        let p = &self.c.spec.player;
        let mut m = mult;
        if crit_scaling {
            m *= 1.0 + self.crit_chance(0.0, &[]);
        }
        if !raw {
            m *= p.global_mult * (1.0 + p.versatility + self.buff_sum(&self.c.vers_buffs));
            if physical {
                m *= self.c.spec.fight.physical_mult;
            }
            for &(b, pct) in mods {
                m *= 1.0 + pct * self.mod_stacks(b);
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

    /// (attack power, spell power) of the owner, or of a pet from the owner's.
    #[inline(always)]
    fn power(&self, pet: Option<usize>) -> (f64, f64) {
        let (ap, sp) = (self.ap(), self.sp());
        match pet {
            None => (ap, sp),
            Some(p) => {
                let k = &self.c.pets[p].coeff;
                (k.ap_from_ap * ap + k.ap_from_sp * sp, k.sp_from_sp * sp + k.sp_from_ap * ap)
            }
        }
    }

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
            self.dot_stk[di] = 1;
            if def.tick_on_apply {
                self.dot_tick(di, 1.0);
            }
            self.start_ticking(di);
        }
    }

    fn cancel_dot(&mut self, di: usize) {
        self.dot_exp[di] = -1.0;
        self.dot_gen[di] += 1;
        self.dot_stk[di] = 0;
        self.bank[di] = 0.0;
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
                m *= 1.0 + pct * self.mod_stacks(b);
            }
            self.record(d.src, part * m, false);
        } else if d.def.ap_coeff != 0.0 || d.def.sp_coeff != 0.0 {
            let mastery = if d.def.mastery { 1.0 + d.def.mastery_scale * self.mastery() } else { 1.0 };
            let (ap, sp) = self.power(d.pet);
            let mut base = (d.def.ap_coeff * ap + d.def.sp_coeff * sp) * frac;
            let prev = std::mem::replace(&mut self.cur_pet, d.pet);
            if d.def.per_stack && d.def.max_stacks > 1 {
                base *= self.dot_stk[di] as f64;
            }
            let (crit, _) = self.hit(d.src, base, d.def.mult * mastery, &d.buff_mods, &d.crit_mods, d.physical, d.def.crit_bonus, d.def.crit_damage_bonus, d.def.crit_scaling, false);
            self.cur_pet = prev;
            if crit && d.def.extend_on_crit > 0.0 && self.dot_exp[di] > self.now {
                self.dot_exp[di] += d.def.extend_on_crit;
            }
            if crit {
                for e in &d.on_crit {
                    self.apply(e);
                }
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
            if a.pet.is_none() && a.def.cooldown > 0.0 && self.charges_now(i) == 0 {
                return;
            }
        }
        if self.log {
            let res: Vec<String> = c.res_names.iter().zip(&self.res).map(|(n, v)| format!("{n}={v}")).collect();
            println!("{:8.3} {} {} [{}]", self.now, if triggered { "trigger" } else { "cast" }, a.def.name, res.join(" "));
        }
        self.depth += 1;
        self.cur.push(a.src);
        let prev_pet = std::mem::replace(&mut self.cur_pet, a.pet);
        if !triggered {
            let free = a.free_with.is_some_and(|b| self.buff_up(b));
            if free {
                self.apply(&CEffect::Consume { buff: a.free_with.unwrap(), all: false });
                for e in &a.on_free {
                    self.apply(e);
                }
            } else {
                for &(r, amt) in &a.cost {
                    self.res[r] -= amt;
                }
            }
        }
        for &(r, amt) in &a.gain {
            self.gain(r, amt);
        }
        if a.pet.is_none() {
            self.start_cd(i);
        }
        for e in &a.on_start {
            self.apply(e);
        }
        if let Some(d) = &a.def.damage {
            let (ap, sp) = self.power(a.pet);
            let scale = a.scale.as_ref().map(|e| e.eval(self));
            for _ in 0..d.hits {
                let w = if d.weapon_mult > 0.0 { self.weapon_roll(a.src) * d.weapon_mult } else { 0.0 };
                let base = match a.stored_from {
                    Some(b) => std::mem::take(&mut self.stored[b]),
                    None => d.ap_coeff * ap + d.sp_coeff * sp + w + d.amount,
                };
                let base = match scale { Some(x) => base * x, None => base };
                let raw = a.stored_from.is_some();
                let mastery = if d.mastery && !raw { 1.0 + d.mastery_scale * self.mastery() } else { 1.0 };
                let (crit, dmg) = self.hit(a.src, base, d.mult * mastery, &a.buff_mods, &a.crit_mods, a.physical, d.crit_bonus, d.crit_damage_bonus, d.crit_scaling, raw);
                for &(di, pct, per_mastery) in &a.feeds {
                    let share = pct + per_mastery * self.mastery();
                    self.feed(di, dmg * share);
                }
                if a.pet.is_none() && !a.def.no_procs {
                    self.hit_procs();
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
        self.cur_pet = prev_pet;
        self.cur.pop();
        self.depth -= 1;
    }

    fn pet_active(&self, p: usize, s: usize) -> bool {
        let i = &self.pets[p][s];
        i.alive && i.exp > self.now
    }

    fn pet_haste(&self, p: usize) -> f64 { if self.c.pets[p].inherit_haste { self.haste_mult() } else { 1.0 } }

    #[inline(never)]
    fn summon(&mut self, p: usize, count: u32, dur: Option<f64>) {
        let c = self.c;
        let pet = &c.pets[p];
        let dur = dur.unwrap_or(pet.duration);
        for _ in 0..count {
            if pet.max_active > 0 {
                let active: Vec<usize> = (0..self.pets[p].len()).filter(|&s| self.pet_active(p, s)).collect();
                if active.len() as u32 >= pet.max_active {
                    let s = *active.iter().min_by(|&&a, &&b| self.pets[p][a].exp.total_cmp(&self.pets[p][b].exp)).unwrap();
                    self.pet_leave(p, s, false);
                }
            }
            let s = match self.pets[p].iter().position(|i| !i.alive) {
                Some(s) => s,
                None => {
                    self.pets[p].push(PetInst {
                        alive: false, generation: 0, exp: 0.0, spawned: 0.0, energy: 0.0, gcd_end: 0.0, casting: 0,
                        cd: vec![0.0; pet.n_abilities],
                    });
                    self.pets[p].len() - 1
                }
            };
            let now = self.now;
            let i = &mut self.pets[p][s];
            i.alive = true;
            i.generation += 1;
            i.exp = if dur > 0.0 { now + dur } else { INF };
            i.spawned = now;
            i.energy = pet.energy;
            i.gcd_end = now;
            i.cd.fill(0.0);
            let (exp, g, k) = (i.exp, i.generation, pkey(p, s));
            if self.log {
                println!("{:8.3} summon {}", now, c.pet_names[p]);
            }
            if exp < INF {
                self.push(exp, Ev::PetExpire(k, g));
            }
            if !pet.actions.is_empty() {
                self.push(now + pet.delay, Ev::PetAct(k, g));
            }
            if pet.melee.is_some() {
                self.push(now + pet.delay, Ev::PetSwing(k, g));
            }
            let prev = self.ctx.replace((p, s));
            for e in &pet.on_summon {
                self.apply(e);
            }
            self.ctx = prev;
        }
    }

    #[inline(never)]
    fn dismiss(&mut self, pet: usize, count: u32, each: &[CEffect]) {
        let mut v: Vec<usize> = (0..self.pets[pet].len()).filter(|&s| self.pet_active(pet, s)).collect();
        v.sort_by(|&a, &b| self.pets[pet][a].spawned.total_cmp(&self.pets[pet][b].spawned));
        if count > 0 {
            v.truncate(count as usize);
        }
        for s in v {
            let prev = self.ctx.replace((pet, s));
            for e in each {
                self.apply(e);
            }
            self.ctx = prev;
            self.pet_leave(pet, s, false);
        }
    }

    #[inline(never)]
    fn extend_pet(&mut self, pet: usize, secs: f64) {
        for s in 0..self.pets[pet].len() {
            if self.pet_active(pet, s) && self.pets[pet][s].exp < INF {
                let i = &mut self.pets[pet][s];
                i.exp += secs;
                let (t, g) = (i.exp, i.generation);
                self.push(t, Ev::PetExpire(pkey(pet, s), g));
            }
        }
    }

    /// An instance leaves; `expire_fx`: run the pet's `on_expire` effects.
    fn pet_leave(&mut self, p: usize, s: usize, expire_fx: bool) {
        let c = self.c;
        let i = &mut self.pets[p][s];
        if !i.alive {
            return;
        }
        i.alive = false;
        i.generation += 1;
        if self.log {
            println!("{:8.3} {} leaves", self.now, c.pet_names[p]);
        }
        if expire_fx {
            let prev = self.ctx.replace((p, s));
            for e in &c.pets[p].on_expire {
                self.apply(e);
            }
            self.ctx = prev;
        }
    }

    /// Is the event for this instance still current?
    fn pet_current(&self, k: u32, g: u32) -> Option<(usize, usize)> {
        let (p, s) = unkey(k);
        (self.pets[p][s].generation == g && self.pet_active(p, s)).then_some((p, s))
    }

    /// Out of energy for every action: the instance leaves.
    fn pet_check_energy(&mut self, p: usize, s: usize) -> bool {
        let m = self.c.pets[p].min_cost;
        if m > 0.0 && self.pets[p][s].alive && self.pets[p][s].energy + 1e-9 < m {
            self.pet_leave(p, s, true);
            return false;
        }
        true
    }

    /// A pet instance performs an ability: pays energy, starts its own cooldown.
    fn pet_execute(&mut self, p: usize, s: usize, ab: usize) {
        let a = &self.c.abilities[ab];
        let h = self.pet_haste(p);
        let now = self.now;
        let i = &mut self.pets[p][s];
        i.energy -= a.pet_cost;
        if a.def.cooldown > 0.0 {
            i.cd[a.pet_slot] = now + if a.def.cooldown_hasted { a.def.cooldown / h } else { a.def.cooldown };
        }
        let prev = self.ctx.replace((p, s));
        self.execute(ab, true);
        self.ctx = prev;
    }

    /// A pet instance picks and starts its next action.
    fn pet_act(&mut self, p: usize, s: usize) {
        let c = self.c;
        let pet = &c.pets[p];
        if !self.pet_check_energy(p, s) {
            return;
        }
        let now = self.now;
        let prev = self.ctx.replace((p, s));
        let (mut pick, mut next, mut waiting) = (None, INF, false);
        for (ab, cond) in &pet.actions {
            let a = &c.abilities[*ab];
            let i = &self.pets[p][s];
            let ready = i.cd[a.pet_slot];
            if ready > now + 1e-9 {
                next = next.min(ready);
                continue;
            }
            if i.energy + 1e-9 < a.pet_cost {
                continue;
            }
            if a.requires_buff.is_some_and(|b| !self.buff_up(b)) || cond.as_ref().is_some_and(|e| e.eval(self) == 0.0) {
                waiting = true;
                continue;
            }
            pick = Some(*ab);
            break;
        }
        self.ctx = prev;
        let (k, g) = (pkey(p, s), self.pets[p][s].generation);
        match pick {
            Some(ab) => {
                let d = &c.abilities[ab].def;
                let h = self.pet_haste(p);
                let ct = if d.cast_hasted { d.cast_time / h } else { d.cast_time };
                let gcd = if d.gcd <= 0.0 { 0.0 } else if d.gcd_hasted { (d.gcd / h).max(0.75) } else { d.gcd };
                self.pets[p][s].gcd_end = now + gcd;
                if ct > 0.0 {
                    self.pets[p][s].casting = ab;
                    self.push(now + ct, Ev::PetCast(k, g));
                } else {
                    self.pet_execute(p, s, ab);
                    if self.pet_current(k, g).is_some() && self.pet_check_energy(p, s) {
                        self.push(now + gcd, Ev::PetAct(k, g));
                    }
                }
            }
            None => {
                if waiting {
                    next = next.min(now + 0.5);
                }
                if next < INF {
                    self.push(next, Ev::PetAct(k, g));
                }
            }
        }
    }

    /// Roll every on-hit proc for a direct hit of the player's.
    fn hit_procs(&mut self) {
        let c = self.c;
        for (k, hp) in c.hit_procs.iter().enumerate() {
            if self.now < self.hp_ready[k] || c.abilities[hp.ab].requires_buff.is_some_and(|b| !self.buff_up(b)) {
                continue;
            }
            let chance = if hp.rppm > 0.0 {
                // SimC's real_ppm_t: one attempt per timestamp; the chance grows
                // with the time since the last attempt (capped) and, past 1.5
                // average intervals without a proc, with bad-luck protection.
                if self.hp_last[k] == self.now {
                    continue;
                }
                let dt = (self.now - self.hp_last[k]).min(RPPM_MAX_INTERVAL);
                self.hp_last[k] = self.now;
                self.hp_blp[k] += dt;
                let rate = hp.rppm * if hp.hasted { self.haste_mult() } else { 1.0 };
                let blp = 1.0 + (self.hp_blp[k].min(RPPM_MAX_BLP) * rate / 60.0 - 1.5) * 3.0;
                rate * dt / 60.0 * blp.max(1.0)
            } else {
                hp.chance
            };
            if self.rng.f64(c.hit_proc_stream(k)) < chance {
                self.hp_blp[k] = 0.0;
                self.hp_ready[k] = self.now + hp.icd;
                self.execute(hp.ab, true);
            }
        }
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

    fn start(&mut self) {
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
    }

    fn run(mut self) -> IterResult {
        self.start();
        while self.step() {}
        self.out
    }

    /// Process the next event. False once the fight is over.
    #[inline(always)]
    fn step(&mut self) -> bool {
        let c = self.c;
        let Some(Queued(t, _, ev)) = self.q.pop() else { return false };
        self.now = t as f64 / 1e6;
        if self.now >= self.end {
            return false;
        }
        let auto_src = c.names.len() - 1;
        match ev {
            Ev::Auto => {
                match c.auto_replace {
                    Some(i) => self.execute(i, true),
                    None => {
                        let base = self.weapon_roll(auto_src);
                        self.hit(auto_src, base, c.auto_mult, &c.auto_mods, &[], true, 0.0, 0.0, false, false);
                        self.out.casts[auto_src] += 1;
                        self.hit_procs();
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
                    return true;
                }
                let d = &c.dots[di].def;
                let tick = self.tick_len(d);
                let frac = ((self.now - self.dot_last[di]) / tick).clamp(0.0, 1.0);
                self.dot_tick(di, frac);
                self.dot_last[di] = self.now;
                if self.now + tick <= self.dot_exp[di] + 1e-6 {
                    self.push(self.now + tick, Ev::DotTick(di as u32, g));
                } else if !d.whole_ticks && self.dot_exp[di] > self.now + 1e-6 {
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
            Ev::PetAct(k, g) => {
                if let Some((p, s)) = self.pet_current(k, g) {
                    self.pet_act(p, s);
                }
            }
            Ev::PetCast(k, g) => {
                if let Some((p, s)) = self.pet_current(k, g) {
                    let ab = self.pets[p][s].casting;
                    self.pet_execute(p, s, ab);
                    if self.pet_current(k, g).is_some() && self.pet_check_energy(p, s) {
                        let t = self.pets[p][s].gcd_end.max(self.now);
                        self.push(t, Ev::PetAct(k, g));
                    }
                }
            }
            Ev::PetSwing(k, g) => {
                if let Some((p, s)) = self.pet_current(k, g) {
                    let (ab, speed, hasted) = c.pets[p].melee.unwrap();
                    self.pet_execute(p, s, ab);
                    if self.pet_current(k, g).is_some() {
                        let t = if hasted { speed / self.pet_haste(p) } else { speed };
                        self.push(self.now + t, Ev::PetSwing(k, g));
                    }
                }
            }
            Ev::PetExpire(k, g) => {
                let (p, s) = unkey(k);
                let i = &self.pets[p][s];
                // stale if extended or gone since
                if i.alive && i.generation == g && (i.exp - self.now).abs() < 1e-6 {
                    self.pet_leave(p, s, true);
                }
            }
            Ev::BuffExpire(b) => {
                let b = b as usize;
                // stale if the buff was refreshed, extended or consumed since
                if (self.buff_exp[b] - self.now).abs() < 1e-6 && self.buff_stk[b] > 0 {
                    let k = self.buff_stk[b] as f64;
                    let k0 = if c.buffs[b].flat { k.min(1.0) } else { k };
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
                    return true;
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
        true
    }
}

pub fn run_iteration(c: &Compiled, seed: u64, log: bool) -> IterResult {
    State::new(c, seed, log).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare owner with no crit, haste or versatility, so pet damage is exact.
    const BASE: &str = r#"
name = "pet test"
[player]
spell_power = 1000
crit = 0.0
haste = 0.0
versatility = 0.0
[fight]
duration = 100
variance = 0

[[buffs]]
name = "empowered"
duration = 15
max_stacks = 30
pet_pct = { overlord = 10 }

[[buffs]]
name = "owner_only"
duration = 100
damage_pct = 50

[[buffs]]
name = "everyone"
duration = 100
damage_pct = 50
pets = true

[[pets]]
name = "imp"
duration = 40
energy = 100
owner_coeff = { sp_from_sp = 0.5 }
on_expire = [{ type = "trigger", ability = "imp_fades" }]
[[pets.abilities]]
name = "firebolt"
cast_time = 2.0
gcd = 0
cost = { energy = 20 }
damage = { sp_coeff = 1.0, school = "fire" }

[[pets]]
name = "hound"
duration = 12
melee = { ability = "melee", speed = 2.0 }
[[pets.abilities]]
name = "melee"
gcd = 0
damage = { ap_coeff = 1.0, school = "shadow" }
[[pets.abilities]]
name = "maul"
cooldown = 60
damage = { ap_coeff = 3.0, school = "shadow" }

[[pets]]
name = "overlord"
duration = 15
max_active = 2
inherit_crit = false
[[pets.abilities]]
name = "blast"
cast_time = 2.0
gcd = 0
damage = { sp_coeff = 2.0, school = "shadow" }

[[abilities]]
name = "summon_imp"
triggered_only = true
on_cast = [{ type = "summon", pet = "imp" }]

[[abilities]]
name = "summon_hound"
triggered_only = true
on_cast = [{ type = "summon", pet = "hound" }]

[[abilities]]
name = "summon_overlord"
triggered_only = true
on_cast = [
  { type = "buff", buff = "empowered", set = "pet.imp.active+pet.hound.active" },
  { type = "summon", pet = "overlord" },
]

[[abilities]]
name = "extend_hounds"
triggered_only = true
on_cast = [{ type = "extend_pet", pet = "hound", seconds = 15 }]

[[abilities]]
name = "implode"
triggered_only = true
on_cast = [{ type = "dismiss", pet = "imp", count = 3, each = [{ type = "trigger", ability = "implode_hit" }] }]

[[abilities]]
name = "implode_hit"
triggered_only = true
damage = { sp_coeff = 1.5, school = "fire", scale = "self.energy/100" }

[[abilities]]
name = "imp_fades"
triggered_only = true

[[abilities]]
name = "buff_owner"
triggered_only = true
on_cast = [{ type = "buff", buff = "owner_only" }]

[[abilities]]
name = "buff_everyone"
triggered_only = true
on_cast = [{ type = "buff", buff = "everyone" }]

[[abilities]]
name = "wait"
gcd = 1.0

[[rotation]]
action = "wait"
"#;

    fn compile(text: &str) -> Compiled { Compiled::new(toml::from_str(text).unwrap()).unwrap() }

    fn ab(c: &Compiled, n: &str) -> usize { c.abilities.iter().position(|a| a.def.name == n).unwrap() }
    fn src(c: &Compiled, n: &str) -> usize { c.names.iter().position(|a| a == n).unwrap() }
    fn pet(c: &Compiled, n: &str) -> usize { c.pet_names.iter().position(|a| a == n).unwrap() }

    /// Run every event up to time `t`, then stand at `t`.
    fn advance(st: &mut State, t: f64) {
        while st.q.peek().is_some_and(|e| e.0 as f64 / 1e6 <= t) && st.step() {}
        st.now = t;
    }

    fn cast(st: &mut State, n: &str) {
        let i = ab(st.c, n);
        st.execute(i, true);
    }

    fn active(st: &State, p: &str) -> f64 { st.var(&Var::PetActive(pet(st.c, p))) }

    fn state(c: &Compiled) -> State<'_> {
        let mut st = State::new(c, 1, false);
        st.start();
        st
    }

    #[test]
    fn energy_limits_casts_and_expiry_effects_fire() {
        let c = compile(BASE);
        let mut st = state(&c);
        cast(&mut st, "summon_imp");
        advance(&mut st, 9.9);
        assert_eq!(st.out.casts[src(&c, "imp_firebolt")], 4);
        assert_eq!(active(&st, "imp"), 1.0);
        advance(&mut st, 10.1);
        // the fifth bolt spends the last energy: it leaves right after
        assert_eq!(st.out.casts[src(&c, "imp_firebolt")], 5);
        assert_eq!(active(&st, "imp"), 0.0);
        assert_eq!(st.out.casts[src(&c, "imp_fades")], 1);
        // half the owner's spell power
        assert!((st.out.damage[src(&c, "imp_firebolt")] - 5.0 * 500.0).abs() < 1e-9);
    }

    #[test]
    fn concurrent_instances_keep_their_own_timers() {
        let c = compile(BASE);
        let mut st = state(&c);
        for _ in 0..3 {
            cast(&mut st, "summon_hound");
        }
        advance(&mut st, 5.0);
        cast(&mut st, "summon_hound");
        cast(&mut st, "summon_hound");
        advance(&mut st, 11.9);
        assert_eq!(active(&st, "hound"), 5.0);
        assert!((st.var(&Var::PetRemains(pet(&c, "hound"))) - 5.1).abs() < 1e-6);
        advance(&mut st, 12.1);
        assert_eq!(active(&st, "hound"), 2.0);
        advance(&mut st, 17.1);
        assert_eq!(active(&st, "hound"), 0.0);
        // swings at 0, 2, ..., 10 for each hound; one maul each (60s cooldown)
        assert_eq!(st.out.casts[src(&c, "hound_melee")], 5 * 6);
        assert_eq!(st.out.casts[src(&c, "hound_maul")], 5);
        // pet attack power from the owner's spell power is 0 by default
        let c2 = compile(&BASE.replace("name = \"hound\"\n", "name = \"hound\"\nowner_coeff = { ap_from_sp = 2.0 }\n"));
        let mut st = state(&c2);
        cast(&mut st, "summon_hound");
        advance(&mut st, 0.5);
        assert!((st.out.damage[src(&c2, "hound_melee")] - 2000.0).abs() < 1e-9);
    }

    #[test]
    fn max_active_replaces_the_oldest() {
        let c = compile(BASE);
        let mut st = state(&c);
        for _ in 0..3 {
            cast(&mut st, "summon_overlord");
        }
        assert_eq!(active(&st, "overlord"), 2.0);
    }

    #[test]
    fn pet_counts_work_in_conditions() {
        let text = BASE.replace(
            "[[rotation]]\naction = \"wait\"",
            "[[rotation]]\naction = \"summon_hound_now\"\nif = \"pet.hound.active<3\"\n\n[[rotation]]\naction = \"wait\"",
        ) + "\n[[abilities]]\nname = \"summon_hound_now\"\ngcd = 1.0\non_cast = [{ type = \"summon\", pet = \"hound\" }]\n";
        let c = compile(&text);
        let mut st = state(&c);
        advance(&mut st, 11.5);
        assert_eq!(st.out.casts[src(&c, "summon_hound_now")], 3);
        advance(&mut st, 12.5);
        // the first hound left at 12: one more
        assert_eq!(st.out.casts[src(&c, "summon_hound_now")], 4);
        assert_eq!(active(&st, "hound"), 3.0);
    }

    #[test]
    fn dismiss_consumes_oldest_and_scales_by_their_energy() {
        let c = compile(BASE);
        let mut st = state(&c);
        for _ in 0..4 {
            cast(&mut st, "summon_imp");
        }
        advance(&mut st, 4.5); // two bolts each: 60 energy left
        cast(&mut st, "summon_imp"); // 100 energy, newest
        cast(&mut st, "implode");
        assert_eq!(active(&st, "imp"), 2.0);
        assert_eq!(st.out.hits[src(&c, "implode_hit")], 3);
        assert!((st.out.damage[src(&c, "implode_hit")] - 3.0 * 1500.0 * 0.6).abs() < 1e-9);
        // dismissed imps don't run their on_expire effects
        assert_eq!(st.out.casts[src(&c, "imp_fades")], 0);
        advance(&mut st, 30.0);
        // the rest cast out their energy and fade normally
        assert_eq!(active(&st, "imp"), 0.0);
        assert_eq!(st.out.casts[src(&c, "imp_fades")], 2);
    }

    #[test]
    fn extend_pet_moves_the_expiry() {
        let c = compile(BASE);
        let mut st = state(&c);
        cast(&mut st, "summon_hound");
        advance(&mut st, 5.0);
        cast(&mut st, "extend_hounds");
        assert!((st.var(&Var::PetRemains(pet(&c, "hound"))) - 22.0).abs() < 1e-6);
        advance(&mut st, 12.5);
        assert_eq!(active(&st, "hound"), 1.0);
        advance(&mut st, 27.1);
        assert_eq!(active(&st, "hound"), 0.0);
    }

    #[test]
    fn buff_set_from_pet_count_and_pet_damage_bonuses() {
        let c = compile(BASE);
        let mut st = state(&c);
        for _ in 0..3 {
            cast(&mut st, "summon_imp");
        }
        cast(&mut st, "summon_hound");
        cast(&mut st, "summon_hound");
        cast(&mut st, "summon_overlord");
        let emp = c.buffs.iter().position(|b| b.name == "empowered").unwrap();
        assert_eq!(st.stacks(emp), 5.0);
        advance(&mut st, 2.1);
        // 2 x 1000 sp, +10% per stack
        assert!((st.out.damage[src(&c, "overlord_blast")] - 2000.0 * 1.5).abs() < 1e-9);
        // owner-wide bonuses reach pets only from buffs flagged `pets`
        let bolts = src(&c, "imp_firebolt");
        let before = st.out.damage[bolts];
        cast(&mut st, "buff_owner");
        advance(&mut st, 4.1);
        assert!((st.out.damage[bolts] - before - 3.0 * 500.0).abs() < 1e-9);
        cast(&mut st, "buff_everyone");
        advance(&mut st, 6.1);
        assert!((st.out.damage[bolts] - before - 3.0 * 500.0 - 3.0 * 750.0).abs() < 1e-9);
        // set = 0 removes the buff
        let mut st = state(&c);
        cast(&mut st, "summon_overlord");
        assert_eq!(st.stacks(emp), 0.0);
    }

    #[test]
    fn crit_and_haste_inheritance() {
        let text = BASE.replace("crit = 0.0\nhaste = 0.0", "crit = 1.0\nhaste = 1.0");
        let c = compile(&text);
        let mut st = state(&c);
        cast(&mut st, "summon_imp");
        cast(&mut st, "summon_overlord");
        advance(&mut st, 5.1);
        // doubled haste: 1s casts
        assert_eq!(st.out.casts[src(&c, "imp_firebolt")], 5);
        let s = src(&c, "imp_firebolt");
        assert_eq!(st.out.crits[s], st.out.hits[s]);
        // the overlord has its own crit chance (0)
        assert_eq!(st.out.crits[src(&c, "overlord_blast")], 0);
        assert_eq!(st.out.casts[src(&c, "overlord_blast")], 5);
        let c = compile(&text.replace("name = \"imp\"\n", "name = \"imp\"\ninherit_haste = false\n"));
        let mut st = state(&c);
        cast(&mut st, "summon_imp");
        advance(&mut st, 5.1);
        assert_eq!(st.out.casts[src(&c, "imp_firebolt")], 2);
    }

    #[test]
    fn calibration_reaches_pet_sources() {
        let mut spec: Spec = toml::from_str(BASE).unwrap();
        let cal: Calibration = toml::from_str("[sources.imp_firebolt]\nmult = 2.0\n").unwrap();
        spec.apply_calibration(&cal).unwrap();
        let c = Compiled::new(spec).unwrap();
        let mut st = state(&c);
        cast(&mut st, "summon_imp");
        advance(&mut st, 2.1);
        assert!((st.out.damage[src(&c, "imp_firebolt")] - 1000.0).abs() < 1e-9);
        let mut spec: Spec = toml::from_str(BASE).unwrap();
        let bad: Calibration = toml::from_str("[sources.firebolt]\nmult = 2.0\n").unwrap();
        assert!(spec.apply_calibration(&bad).is_err());
    }

    #[test]
    fn spec_errors() {
        let bad = |t: &str| Compiled::new(toml::from_str(t).unwrap()).err().unwrap();
        let e = bad(&BASE.replace("[[rotation]]\naction = \"wait\"", "[[rotation]]\naction = \"imp_firebolt\""));
        assert!(e.contains("pet ability"), "{e}");
        let e = bad(&BASE.replace("energy = 100\n", ""));
        assert!(e.contains("energy"), "{e}");
        let e = bad(&BASE.replace("name = \"melee\"\ngcd = 0", "name = \"melee\"\ngcd = 0\ncost = { holy_power = 1 }"));
        assert!(e.contains("energy"), "{e}");
        let e = bad(&BASE.replace("pet = \"hound\" }]", "pet = \"wolf\" }]"));
        assert!(e.contains("unknown pet 'wolf'"), "{e}");
    }

/// No crit, haste or versatility: one strike a second, gear-style procs.
    const PROCS: &str = r#"
name = "proc test"
[player]
attack_power = 1000
crit = 0.0
haste = 0.0
versatility = 0.0
[fight]
duration = 300
variance = 0
[resources.power]
max = 10
start = 10

[[buffs]]
name = "free"
duration = 100
[[buffs]]
name = "marked"
duration = 100
[[buffs]]
name = "window"
duration = 20
[[buffs]]
name = "vers"
duration = 100
versatility_pct = 10

[[abilities]]
name = "spend"
gcd = 0
cost = { power = 5 }
free_with = "free"
on_free = [{ type = "buff", buff = "marked" }]

[[abilities]]
name = "strike"
gcd = 1.0
damage = { ap_coeff = 1.0, school = "holy" }

[[abilities]]
name = "flat"
triggered_only = true
no_procs = true
damage = { amount = 500, school = "holy" }

[[abilities]]
name = "gated"
triggered_only = true
no_procs = true
requires_buff = "window"
damage = { amount = 1, school = "holy" }

[[procs]]
ability = "flat"
per_minute = 6
on_hit = true

[[procs]]
ability = "gated"
on_hit = true
chance = 1.0
icd = 2.0

[[rotation]]
action = "strike"
"#;

    #[test]
    fn on_free_effects_only_when_the_buff_pays() {
        let c = compile(PROCS);
        let mut st = state(&c);
        let (marked, free) = (c.spec.buffs.iter().position(|b| b.name == "marked").unwrap(), ab(&c, "spend"));
        st.execute(free, false);
        assert_eq!(st.res[0], 5.0);
        assert!(!st.buff_up(marked));
        st.apply(&CEffect::Buff { buff: 0, stacks: 1, chance: 1.0, dur: None, set: None, rid: 0 });
        st.execute(free, false);
        assert_eq!(st.res[0], 5.0, "a free cast costs nothing");
        assert!(st.buff_up(marked));
    }

    #[test]
    fn flat_damage_and_versatility_buffs() {
        let c = compile(PROCS);
        let mut st = state(&c);
        cast(&mut st, "flat");
        assert_eq!(st.out.damage[src(&c, "flat")], 500.0);
        let vers = c.spec.buffs.iter().position(|b| b.name == "vers").unwrap();
        st.apply(&CEffect::Buff { buff: vers, stacks: 1, chance: 1.0, dur: None, set: None, rid: 0 });
        cast(&mut st, "flat");
        assert!((st.out.damage[src(&c, "flat")] - 1050.0).abs() < 1e-9);
    }

    #[test]
    fn on_hit_procs_follow_rate_buff_and_icd() {
        let c = compile(PROCS);
        let r = run_iteration(&c, 5, false);
        // 6 a minute over 300s, a little more with bad-luck protection
        let n = r.casts[src(&c, "flat")];
        assert!((25..=45).contains(&n), "real-PPM procs {n}");
        assert_eq!(r.casts[src(&c, "gated")], 0, "no rolls while its buff is down");

        let mut st = state(&c);
        advance(&mut st, 10.0);
        let window = c.spec.buffs.iter().position(|b| b.name == "window").unwrap();
        st.apply(&CEffect::Buff { buff: window, stacks: 1, chance: 1.0, dur: None, set: None, rid: 0 });
        advance(&mut st, 40.0);
        // one per hit, at most one per 2s internal cooldown, for 20s
        assert_eq!(st.out.casts[src(&c, "gated")], 10);
    }

    #[test]
    fn example_spec_runs() {
        let c = compile(include_str!("../specs/examples/summoner.toml"));
        let r = run_iteration(&c, 3, false);
        for n in ["imp_firebolt", "hound_melee", "hound_maul", "overlord_blast", "implode_hit", "bolt"] {
            assert!(r.damage[src(&c, n)] > 0.0, "{n} dealt no damage");
        }
        assert!(r.casts[src(&c, "imp_fades")] > 0);
    }
}
