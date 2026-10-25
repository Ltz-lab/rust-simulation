//! Event-driven combat engine. Knows nothing about any class: everything
//! comes from the compiled spec.

use crate::expr::{self, Env, Expr, Resolver, Var};
use crate::spec::*;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

const INF: f64 = f64::INFINITY;
const MAX_TRIGGER_DEPTH: u32 = 8;

/// xoshiro256++ — small, fast, good-quality RNG; no dependency churn.
struct Rng([u64; 4]);
impl Rng {
    fn seed(mut x: u64) -> Self {
        let mut s = [0u64; 4];
        for v in &mut s {
            x = x.wrapping_add(0x9E3779B97F4A7C15); // splitmix64
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            *v = z ^ (z >> 31);
        }
        Rng(s)
    }
    fn next_u64(&mut self) -> u64 {
        let s = &mut self.0;
        let r = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        r
    }
    /// Uniform in [0, 1).
    fn f64(&mut self) -> f64 { (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64) }
    fn range(&mut self, lo: f64, hi: f64) -> f64 { lo + (hi - lo) * self.f64() }
}

// ---------- compiled spec ----------

#[derive(Debug, Clone)]
enum CEffect {
    Buff { buff: usize, stacks: u32, chance: f64 },
    Consume { buff: usize, all: bool },
    Reset { ab: usize, chance: f64 },
    Reduce { ab: usize, secs: f64, chance: f64 },
    Gain { res: usize, amt: f64, chance: f64 },
    Trigger { ab: usize, chance: f64, delay: f64, mastery_chance: Option<f64> },
    ExtendDot { dot: usize, secs: f64 },
}

#[derive(Debug, Clone)]
struct CAbility {
    def: AbilityDef,
    cost: Vec<(usize, f64)>,
    gain: Vec<(usize, f64)>,
    free_with: Option<usize>,
    requires_buff: Option<usize>,
    on_cast: Vec<CEffect>,
    on_crit: Vec<CEffect>,
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
    physical: bool,
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
    names: Vec<String>,
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

fn buff_mods(buffs: &[BuffDef], ability: &str, school: &str) -> Vec<(usize, f64)> {
    let mut v = Vec::new();
    for (i, b) in buffs.iter().enumerate() {
        let pct = b.damage_pct
            + b.school_pct.get(school).copied().unwrap_or(0.0)
            + b.ability_pct.get(ability).copied().unwrap_or(0.0);
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

        let effect = |e: &Effect| -> Result<CEffect, String> {
            Ok(match e {
                Effect::Buff { buff: b, stacks, chance } => {
                    CEffect::Buff { buff: get(&buff, "buff", b)?, stacks: *stacks, chance: *chance }
                }
                Effect::Consume { buff: b, all } => {
                    CEffect::Consume { buff: get(&buff, "buff", b)?, all: *all }
                }
                Effect::ResetCooldown { ability, chance } => {
                    CEffect::Reset { ab: get(&ab, "ability", ability)?, chance: *chance }
                }
                Effect::ReduceCooldown { ability, seconds, chance } => CEffect::Reduce {
                    ab: get(&ab, "ability", ability)?,
                    secs: *seconds,
                    chance: *chance,
                },
                Effect::Gain { resource, amount, chance } => CEffect::Gain {
                    res: get(&res, "resource", resource)?,
                    amt: *amount,
                    chance: *chance,
                },
                Effect::ExtendDot { dot: d, seconds } => {
                    CEffect::ExtendDot { dot: get(&dot, "dot", d)?, secs: *seconds }
                }
                Effect::Trigger { ability, chance, delay, mastery_chance } => CEffect::Trigger {
                    ab: get(&ab, "ability", ability)?,
                    chance: *chance,
                    delay: *delay,
                    mastery_chance: *mastery_chance,
                },
            })
        };
        let effects = |v: &[Effect]| -> Result<Vec<CEffect>, String> { v.iter().map(&effect).collect() };
        let rmap = |m: &HashMap<String, f64>| -> Result<Vec<(usize, f64)>, String> {
            m.iter().map(|(k, v)| Ok((get(&res, "resource", k)?, *v))).collect()
        };

        let mut abilities = Vec::new();
        let mut dots = Vec::new();
        for a in &spec.abilities {
            let school = a.damage.as_ref().map(|d| d.school.as_str()).unwrap_or("physical");
            let ctx = |e: String| format!("ability '{}': {e}", a.name);
            let dot_idx = match &a.dot {
                Some(d) => {
                    dots.push(CDot {
                        on_tick: effects(&d.on_tick).map_err(ctx)?,
                        buff_mods: buff_mods(&spec.buffs, &d.name, &d.school),
                        physical: d.school == "physical",
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
                on_cast: effects(&a.on_cast).map_err(ctx)?,
                on_crit: effects(&a.on_crit).map_err(ctx)?,
                dot: dot_idx,
                stored_from: a
                    .damage
                    .as_ref()
                    .and_then(|d| d.stored_from.as_ref())
                    .map(|b| get(&buff, "buff", b))
                    .transpose()
                    .map_err(ctx)?,
                buff_mods: buff_mods(&spec.buffs, &a.name, school),
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
                effects(&a.on_hit)?,
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
        let mut src_names: Vec<String> = spec.abilities.iter().map(|a| a.name.clone()).collect();
        src_names.extend(dots.iter().map(|d| d.def.name.clone()));
        src_names.push("auto_attack".into());

        Ok(Compiled {
            res_max: res_names.iter().map(|n| spec.resources[n].max).collect(),
            res_start: res_names.iter().map(|n| spec.resources[n].start).collect(),
            res_names,
            auto_mods: buff_mods(&spec.buffs, "auto_attack", "physical"),
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
            names: src_names,
            spec,
        })
    }

    /// Damage sources in result order: abilities, then dots, then auto attack.
    pub fn source_names(&self) -> &[String] { &self.names }
}

// ---------- per-iteration state ----------

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Ev {
    Ready,
    Auto,
    DotTick(u32, u32),
    Trigger(u32),
    Proc(u32),
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
    rng: Rng,
    now: f64,
    end: f64,
    res: Vec<f64>,
    charges: Vec<u32>,
    recharge_at: Vec<f64>,
    cd_len: Vec<f64>,
    buff_exp: Vec<f64>,
    buff_stk: Vec<u32>,
    stored: Vec<f64>,
    dot_exp: Vec<f64>,
    dot_gen: Vec<u32>,
    dot_last: Vec<f64>,
    gcd_ready: f64,
    q: BinaryHeap<Reverse<(u64, u64, Ev)>>,
    seq: u64,
    depth: u32,
    /// Stack of abilities currently executing (for resource attribution).
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
            Var::BuffRemains(i) => (self.buff_exp[i] - self.now).max(0.0),
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
        let mut rng = Rng::seed(seed);
        let f = &c.spec.fight;
        let end = f.duration * (1.0 + rng.range(-f.variance, f.variance));
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
            stored: vec![0.0; c.buffs.len()],
            dot_exp: vec![-1.0; c.dots.len()],
            dot_gen: vec![0; c.dots.len()],
            dot_last: vec![0.0; c.dots.len()],
            gcd_ready: 0.0,
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

    fn buff_up(&self, i: usize) -> bool { self.buff_exp[i] > self.now }

    fn stacks(&self, i: usize) -> f64 {
        if self.buff_up(i) { self.buff_stk[i] as f64 } else { 0.0 }
    }

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

    fn roll(&mut self, chance: f64) -> bool { chance >= 1.0 || self.rng.f64() < chance }

    fn apply(&mut self, e: &CEffect) {
        match *e {
            CEffect::Buff { buff, stacks, chance } => {
                if !self.roll(chance) {
                    return;
                }
                let def = &self.c.buffs[buff];
                let cur = if self.buff_up(buff) { self.buff_stk[buff] } else { 0 };
                if cur == 0 {
                    self.stored[buff] = 0.0;
                }
                self.buff_stk[buff] = (cur + stacks).min(def.max_stacks);
                self.buff_exp[buff] = self.now + def.duration;
                if self.log {
                    println!("{:8.3} buff {} -> {}", self.now, def.name, self.buff_stk[buff]);
                }
            }
            CEffect::Consume { buff, all } => {
                if !self.buff_up(buff) {
                    return;
                }
                if all || self.buff_stk[buff] <= 1 {
                    self.buff_stk[buff] = 0;
                    self.buff_exp[buff] = self.now;
                } else {
                    self.buff_stk[buff] -= 1;
                }
            }
            CEffect::Reset { ab, chance } => {
                if !self.roll(chance) {
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
            CEffect::Reduce { ab, secs, chance } => {
                if !self.roll(chance) {
                    return;
                }
                if self.recharge_at[ab] != INF {
                    self.recharge_at[ab] -= secs;
                    self.sync_cd(ab);
                }
            }
            CEffect::Gain { res, amt, chance } => {
                if !self.roll(chance) {
                    return;
                }
                self.gain(res, amt);
            }
            CEffect::ExtendDot { dot, secs } => {
                if self.dot_exp[dot] > self.now {
                    self.dot_exp[dot] += secs;
                }
            }
            CEffect::Trigger { ab, chance, delay, mastery_chance } => {
                let chance = match mastery_chance {
                    Some(f) => self.c.spec.player.mastery * f,
                    None => chance,
                };
                if !self.roll(chance) {
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

    fn crit_chance(&self, bonus: f64) -> f64 {
        let mut c = self.c.spec.player.crit + bonus;
        for &(b, p) in &self.c.crit_buffs {
            c += p * self.stacks(b);
        }
        c
    }

    /// Shared damage roll for every source. Returns whether it crit.
    /// `raw`: base is already final damage (e.g. a stored-damage burst) —
    /// skip versatility, armor and buff multipliers.
    #[allow(clippy::too_many_arguments)]
    fn hit(&mut self, src: usize, base: f64, mult: f64, mods: &[(usize, f64)], physical: bool, crit_bonus: f64, crit_dmg: f64, raw: bool) -> bool {
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
        let crit = self.rng.f64() < self.crit_chance(crit_bonus);
        let dmg = base * m * if crit { p.crit_multiplier + crit_dmg } else { 1.0 };
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
        crit
    }

    fn ap(&self) -> f64 {
        let p = &self.c.spec.player;
        p.attack_power + p.weapon_attack_power
    }

    fn weapon_roll(&mut self) -> f64 {
        match &self.c.spec.player.weapon {
            Some(w) => if w.max > w.min { self.rng.range(w.min, w.max) } else { w.min },
            None => 0.0,
        }
    }

    fn tick_len(&self, def: &DotDef) -> f64 {
        if def.hasted { def.tick_time / self.haste_mult() } else { def.tick_time }
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
            self.dot_gen[di] += 1;
            self.dot_last[di] = self.now;
            let g = self.dot_gen[di];
            self.push(self.now + self.tick_len(def), Ev::DotTick(di as u32, g));
        }
    }

    /// One DoT tick; `frac` < 1 for the shortened final tick.
    fn dot_tick(&mut self, di: usize, frac: f64) {
        let c = self.c;
        let d = &c.dots[di];
        let mastery = if d.def.mastery { 1.0 + c.spec.player.mastery } else { 1.0 };
        let base = d.def.ap_coeff * self.ap() * frac;
        let src = c.abilities.len() + di;
        let crit = self.hit(src, base, d.def.mult * mastery, &d.buff_mods, d.physical, d.def.crit_bonus, d.def.crit_damage_bonus, false);
        if crit && d.def.extend_on_crit > 0.0 && self.dot_exp[di] > self.now {
            self.dot_exp[di] += d.def.extend_on_crit;
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
        self.cur.push(i);
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
        if let Some(d) = &a.def.damage {
            let ap = self.ap();
            for _ in 0..d.hits {
                let w = if d.weapon_mult > 0.0 { self.weapon_roll() * d.weapon_mult } else { 0.0 };
                let base = match a.stored_from {
                    Some(b) => std::mem::take(&mut self.stored[b]),
                    None => d.ap_coeff * ap + w,
                };
                let raw = a.stored_from.is_some();
                let mastery = if d.mastery && !raw { 1.0 + c.spec.player.mastery } else { 1.0 };
                let crit = self.hit(i, base, d.mult * mastery, &a.buff_mods, a.physical, d.crit_bonus, d.crit_damage_bonus, raw);
                if crit {
                    for e in &a.on_crit {
                        self.apply(e);
                    }
                }
            }
        }
        if let Some(di) = a.dot {
            self.apply_dot(di);
        }
        for e in &a.on_cast {
            self.apply(e);
        }
        self.out.casts[i] += 1;
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
        let dt = -(1.0 - self.rng.f64()).ln() / rate;
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
                            let base = self.weapon_roll();
                            self.hit(auto_src, base, c.auto_mult, &c.auto_mods, true, 0.0, 0.0, false);
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
                Ev::Ready => {
                    if self.now + 2e-6 < self.gcd_ready {
                        continue;
                    }
                    match self.choose() {
                        Some(i) => {
                            self.execute(i, false);
                            let d = &c.abilities[i].def;
                            if d.gcd > 0.0 {
                                self.gcd_ready = self.now + self.gcd_len(d.gcd, d.gcd_hasted);
                            }
                            self.push(self.gcd_ready.max(self.now), Ev::Ready);
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
