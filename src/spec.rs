//! Spec definition format. A spec is pure data: abilities, buffs and a
//! priority rotation. Nothing class-specific lives in engine code.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Spec {
    pub name: String,
    pub player: PlayerStats,
    #[serde(default)]
    pub fight: Fight,
    #[serde(default)]
    pub resources: HashMap<String, ResourceDef>,
    pub abilities: Vec<AbilityDef>,
    #[serde(default)]
    pub buffs: Vec<BuffDef>,
    #[serde(default)]
    pub auto_attack: Option<AutoAttackDef>,
    /// Abilities cast before the pull (e.g. buffs), in order.
    #[serde(default)]
    pub precombat: Vec<String>,
    /// Random procs not tied to a cast (gear effects): fire `ability` at a
    /// Poisson rate of `per_minute`.
    #[serde(default)]
    pub procs: Vec<ProcDef>,
    /// Summonable pets: each active instance runs its own action loop.
    #[serde(default)]
    pub pets: Vec<PetDef>,
    pub rotation: Vec<RotationEntry>,
}

/// A pet or summon. Any number of instances can be active at once (up to
/// `max_active`), each with its own timer, energy, cooldowns and action loop.
/// Its abilities are reported (and calibrated, and keyed in buffs) as
/// `<pet>_<ability>`.
#[derive(Debug, Deserialize, Clone)]
pub struct PetDef {
    pub name: String,
    /// Seconds each instance stays (0 = until dismissed).
    #[serde(default)]
    pub duration: f64,
    /// Cap on active instances (0 = none): a new summon replaces the one
    /// closest to expiring.
    #[serde(default)]
    pub max_active: u32,
    #[serde(default)]
    pub owner_coeff: OwnerCoeff,
    /// Use the owner's haste (with buffs) for casts, GCDs and swings; else none.
    #[serde(default = "yes")]
    pub inherit_haste: bool,
    /// Use the owner's crit chance (with global crit buffs); else `crit`.
    #[serde(default = "yes")]
    pub inherit_crit: bool,
    #[serde(default)]
    pub crit: f64,
    /// Energy each instance starts with. Abilities' `cost = { energy = .. }`
    /// spends it; an instance that can't afford any action leaves.
    #[serde(default)]
    pub energy: Option<f64>,
    /// Seconds from summon to its first action and swing (travel, leap).
    #[serde(default)]
    pub delay: f64,
    #[serde(default)]
    pub melee: Option<PetMelee>,
    pub abilities: Vec<AbilityDef>,
    /// Action priority list (pet-local ability names). Empty: every ability in
    /// order, except `triggered_only` ones and the melee ability.
    #[serde(default)]
    pub rotation: Vec<RotationEntry>,
    /// Effects when an instance arrives / leaves on its own (expiry, out of
    /// energy). `self.*` in them refers to that instance.
    #[serde(default)]
    pub on_summon: Vec<Effect>,
    #[serde(default)]
    pub on_expire: Vec<Effect>,
}

/// Pet power from the owner's: pet sp = sp_from_sp x owner sp + sp_from_ap x owner ap, etc.
#[derive(Debug, Deserialize, Clone)]
pub struct OwnerCoeff {
    #[serde(default = "one")]
    pub sp_from_sp: f64,
    #[serde(default)]
    pub sp_from_ap: f64,
    #[serde(default = "one")]
    pub ap_from_ap: f64,
    #[serde(default)]
    pub ap_from_sp: f64,
}

impl Default for OwnerCoeff {
    fn default() -> Self { OwnerCoeff { sp_from_sp: 1.0, sp_from_ap: 0.0, ap_from_ap: 1.0, ap_from_sp: 0.0 } }
}

/// A pet's auto attack: executes `ability` every `speed` seconds.
#[derive(Debug, Deserialize, Clone)]
pub struct PetMelee {
    pub ability: String,
    pub speed: f64,
    #[serde(default = "yes")]
    pub hasted: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcDef {
    pub ability: String,
    pub per_minute: f64,
    #[serde(default)]
    pub hasted: bool,
}

/// Final, fully-buffed character stats. Gear -> stats is a later layer;
/// for now we take the stats a character actually has.
#[derive(Debug, Deserialize, Clone)]
pub struct PlayerStats {
    #[serde(default)]
    pub attack_power: f64,
    /// Casters: damage coefficients `sp_coeff` scale with this.
    #[serde(default)]
    pub spell_power: f64,
    /// Attack power that strength doesn't move (weapon damage feeding the
    /// formula). Ability damage scales with attack_power + this.
    #[serde(default)]
    pub weapon_attack_power: f64,
    /// Fractions: 0.25 = 25%.
    pub crit: f64,
    pub haste: f64,
    pub versatility: f64,
    /// Mastery as a fractional damage bonus for abilities flagged `mastery = true`.
    #[serde(default)]
    pub mastery: f64,
    #[serde(default = "default_crit_mult")]
    pub crit_multiplier: f64,
    /// Multiplier on all damage (racials, raid buffs, generic auras).
    #[serde(default = "one")]
    pub global_mult: f64,
    #[serde(default)]
    pub weapon: Option<Weapon>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Weapon {
    pub min: f64,
    pub max: f64,
    pub speed: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fight {
    #[serde(default = "default_fight_len")]
    pub duration: f64,
    /// Fight length varies uniformly by +/- this fraction.
    #[serde(default = "default_variance")]
    pub variance: f64,
    /// Damage multiplier applied to physical hits (boss armor).
    #[serde(default = "default_armor")]
    pub physical_mult: f64,
}

impl Default for Fight {
    fn default() -> Self {
        Fight { duration: 300.0, variance: 0.2, physical_mult: default_armor() }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceDef {
    pub max: f64,
    #[serde(default)]
    pub start: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AbilityDef {
    pub name: String,
    /// Game spell id (documentation / data-pipeline cross reference).
    #[serde(default)]
    #[allow(dead_code)]
    pub spell_id: Option<u32>,
    #[serde(default = "default_gcd")]
    pub gcd: f64,
    #[serde(default = "yes")]
    pub gcd_hasted: bool,
    #[serde(default)]
    pub cooldown: f64,
    #[serde(default)]
    pub cooldown_hasted: bool,
    #[serde(default = "one_u32")]
    pub charges: u32,
    /// Cast time in seconds (0 = instant). The ability resolves when the cast
    /// finishes; the player can't start anything else meanwhile.
    #[serde(default)]
    pub cast_time: f64,
    #[serde(default = "yes")]
    pub cast_hasted: bool,
    #[serde(default)]
    pub cost: HashMap<String, f64>,
    #[serde(default)]
    pub gain: HashMap<String, f64>,
    #[serde(default)]
    pub damage: Option<DamageDef>,
    #[serde(default)]
    pub dot: Option<DotDef>,
    /// If this buff is up, the ability costs nothing and consumes one stack.
    #[serde(default)]
    pub free_with: Option<String>,
    /// Ability is only usable while this buff is up.
    #[serde(default)]
    pub requires_buff: Option<String>,
    /// Effects applied as the ability goes off, before its damage (a spell's
    /// travel time: buffs gained or spent on cast affect the hit).
    #[serde(default)]
    pub on_start: Vec<Effect>,
    #[serde(default)]
    pub on_cast: Vec<Effect>,
    /// Effects when a direct-damage hit crits.
    #[serde(default)]
    pub on_crit: Vec<Effect>,
    /// Effects when a direct-damage hit doesn't crit.
    #[serde(default)]
    pub on_noncrit: Vec<Effect>,
    /// Report damage under this source name instead of the ability's own
    /// (several variants of one spell, e.g. hardcast vs instant).
    #[serde(default)]
    pub source: Option<String>,
    /// Only ever fired by triggers/procs, never chosen by the rotation.
    #[serde(default)]
    pub triggered_only: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DamageDef {
    #[serde(default)]
    pub ap_coeff: f64,
    #[serde(default)]
    pub sp_coeff: f64,
    /// Fraction of weapon damage added (normalized to the weapon's own speed).
    #[serde(default)]
    pub weapon_mult: f64,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
    /// Share of mastery applied when `mastery` is set (e.g. 0.75 for a bonus
    /// rolled uniformly between 50% and 100% of mastery).
    #[serde(default = "one")]
    pub mastery_scale: f64,
    /// Damage is further increased by the player's crit chance.
    #[serde(default)]
    pub crit_scaling: bool,
    /// Residual multiplier: talents/auras not modelled individually.
    /// This is the knob log calibration tunes.
    #[serde(default = "one")]
    pub mult: f64,
    #[serde(default)]
    pub crit_bonus: f64,
    /// Added to the player's crit damage multiplier for this ability.
    #[serde(default)]
    pub crit_damage_bonus: f64,
    /// Deal the damage stored by this buff (see BuffDef::store_damage_pct)
    /// instead of a coefficient, and empty the store.
    #[serde(default)]
    pub stored_from: Option<String>,
    /// Multiply the damage by this condition-language expression (e.g.
    /// `self.energy/100` for a hit made per dismissed pet).
    #[serde(default)]
    pub scale: Option<String>,
    /// Number of separate hits per cast.
    #[serde(default = "one_u32")]
    pub hits: u32,
    /// Each hit adds a share of its damage to banked DoTs (see DotDef::banked).
    #[serde(default)]
    pub feed: Vec<FeedDef>,
}

/// Bank `pct + per_mastery x mastery` of a hit's damage into a banked DoT.
#[derive(Debug, Deserialize, Clone)]
pub struct FeedDef {
    pub dot: String,
    #[serde(default)]
    pub pct: f64,
    #[serde(default)]
    pub per_mastery: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DotDef {
    pub name: String,
    #[serde(default)]
    pub ap_coeff: f64,
    #[serde(default)]
    pub sp_coeff: f64,
    /// Rolling DoT: no coefficient; deals damage banked by `feed` from other
    /// hits, spread evenly over the ticks left. Each feed resets the duration
    /// and keeps the tick cadence. Ticks don't crit (the banked damage already did).
    #[serde(default)]
    pub banked: bool,
    pub tick_time: f64,
    pub duration: f64,
    #[serde(default = "yes")]
    pub hasted: bool,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
    #[serde(default = "one")]
    pub mastery_scale: f64,
    #[serde(default)]
    pub crit_scaling: bool,
    #[serde(default = "one")]
    pub mult: f64,
    #[serde(default)]
    pub crit_bonus: f64,
    #[serde(default)]
    pub crit_damage_bonus: f64,
    /// Stack cap. A fresh application has one stack; refreshing keeps the
    /// count; `dot_stack` effects add or remove stacks.
    #[serde(default = "one_u32")]
    pub max_stacks: u32,
    /// Tick damage is multiplied by the stack count (false: stacks only count).
    #[serde(default = "yes")]
    pub per_stack: bool,
    /// Seconds added to the DoT each time a tick crits.
    #[serde(default)]
    pub extend_on_crit: f64,
    /// Only whole ticks: no shortened final tick at expiry (a pet's attacks).
    #[serde(default)]
    pub whole_ticks: bool,
    /// Deal one tick immediately when freshly applied.
    #[serde(default)]
    pub tick_on_apply: bool,
    /// What reapplying does while ticking: "pandemic" (default: carry over up to
    /// 30% of duration), "extend" (add full duration to what's left) or "reset".
    #[serde(default = "default_refresh")]
    pub refresh: String,
    #[serde(default)]
    pub on_tick: Vec<Effect>,
    /// Effects when a tick crits.
    #[serde(default)]
    pub on_crit: Vec<Effect>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutoAttackDef {
    #[serde(default = "one")]
    pub mult: f64,
    /// Each swing executes this ability instead of dealing weapon damage.
    #[serde(default)]
    pub replace_with: Option<String>,
    #[serde(default)]
    pub on_hit: Vec<Effect>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    Buff {
        buff: String,
        #[serde(default = "one_u32")]
        stacks: u32,
        #[serde(default = "one")]
        chance: f64,
        /// Lasts this long instead of the buff's own duration.
        #[serde(default)]
        duration: Option<f64>,
        /// Set the stack count to this expression's value instead of adding
        /// `stacks` (0 removes the buff).
        #[serde(default)]
        set: Option<String>,
    },
    Consume {
        buff: String,
        #[serde(default)]
        all: bool,
    },
    ResetCooldown {
        ability: String,
        #[serde(default = "one")]
        chance: f64,
    },
    ReduceCooldown {
        ability: String,
        seconds: f64,
        #[serde(default = "one")]
        chance: f64,
    },
    Gain {
        resource: String,
        amount: f64,
        #[serde(default = "one")]
        chance: f64,
    },
    /// Add time to a DoT that is currently ticking.
    ExtendDot {
        dot: String,
        seconds: f64,
    },
    /// Change a ticking DoT's stack count: add `stacks` (negative removes,
    /// never below `min`), or `set` it to the value of a condition-language
    /// expression. A DoT left with no stacks stops.
    DotStack {
        dot: String,
        #[serde(default)]
        stacks: i32,
        #[serde(default)]
        set: Option<String>,
        #[serde(default)]
        min: u32,
    },
    /// Stop a DoT.
    CancelDot {
        dot: String,
    },
    /// Add time to a buff that is currently up.
    ExtendBuff {
        buff: String,
        seconds: f64,
    },
    /// Apply `then` if the rotation-style condition holds, else `else`.
    If {
        #[serde(rename = "if")]
        cond: String,
        #[serde(default)]
        then: Vec<Effect>,
        #[serde(default, rename = "else")]
        otherwise: Vec<Effect>,
    },
    /// Summon `count` instances of a pet (optionally for their own `duration`).
    Summon {
        pet: String,
        #[serde(default = "one_u32")]
        count: u32,
        #[serde(default)]
        duration: Option<f64>,
    },
    /// Make up to `count` (0 = all) active instances of a pet leave, oldest
    /// first, applying `each` once per instance (`self.*` refers to it) and
    /// skipping its `on_expire`.
    Dismiss {
        pet: String,
        #[serde(default)]
        count: u32,
        #[serde(default)]
        each: Vec<Effect>,
    },
    /// Add time to every active instance of a pet.
    ExtendPet {
        pet: String,
        seconds: f64,
    },
    /// Execute another ability for free, off the GCD (optionally delayed).
    /// `mastery_chance`: proc chance = player mastery x this factor (overrides `chance`).
    Trigger {
        ability: String,
        #[serde(default = "one")]
        chance: f64,
        #[serde(default)]
        delay: f64,
        #[serde(default)]
        mastery_chance: Option<f64>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct BuffDef {
    pub name: String,
    pub duration: f64,
    #[serde(default = "one_u32")]
    pub max_stacks: u32,
    /// Per-stack bonuses, in percent (20 = +20%).
    #[serde(default)]
    pub damage_pct: f64,
    #[serde(default)]
    pub school_pct: HashMap<String, f64>,
    #[serde(default)]
    pub ability_pct: HashMap<String, f64>,
    /// Per-pet damage bonus (percent), keyed by pet name.
    #[serde(default)]
    pub pet_pct: HashMap<String, f64>,
    /// `damage_pct` and `school_pct` also apply to pets' damage.
    #[serde(default)]
    pub pets: bool,
    #[serde(default)]
    pub crit_pct: f64,
    /// Per-ability crit chance (percent; 100 = guaranteed crit).
    #[serde(default)]
    pub ability_crit_pct: HashMap<String, f64>,
    /// Added to the crit damage multiplier (percent of normal damage; 10 = +0.1).
    #[serde(default)]
    pub crit_damage_pct: f64,
    /// Flat mastery points (percent: 2 = +0.02 mastery).
    #[serde(default)]
    pub mastery_pct: f64,
    /// Attack/spell power increase (percent), e.g. a primary-stat buff.
    #[serde(default)]
    pub power_pct: f64,
    #[serde(default)]
    pub haste_pct: f64,
    /// Reduces the global cooldown only (percent speed-up).
    #[serde(default)]
    pub gcd_pct: f64,
    /// Per-ability recharge speed-up while up (percent; 50 = recharges 1.5x faster).
    #[serde(default)]
    pub cooldown_rate_pct: HashMap<String, f64>,
    /// While up, this percent of all damage dealt is stored (for a later burst).
    #[serde(default)]
    pub store_damage_pct: f64,
    /// Per-ability cast time reduction while up (percent; 100 = instant).
    #[serde(default)]
    pub cast_time_pct: HashMap<String, f64>,
    /// Reapplying while up adds the duration to what's left instead of
    /// restarting it.
    #[serde(default)]
    pub refresh_extend: bool,
    /// Bonuses apply once while up, whatever the stack count (stacks only
    /// count uses, e.g. "your next 8 casts").
    #[serde(default)]
    pub flat: bool,
    /// Every application is its own stack with its own timer ("multiple
    /// applications may overlap"), instead of one shared timer.
    #[serde(default)]
    pub overlap: bool,
    /// Up whenever the target's health (percent, falling linearly over the
    /// fight) is below / above this, without being applied.
    #[serde(default)]
    pub below_health: Option<f64>,
    #[serde(default)]
    pub above_health: Option<f64>,
    /// Effects when the buff runs out on its own (not when consumed).
    #[serde(default)]
    pub on_expire: Vec<Effect>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RotationEntry {
    pub action: String,
    #[serde(default, rename = "if")]
    pub condition: Option<String>,
}

fn default_crit_mult() -> f64 { 2.0 }
fn one() -> f64 { 1.0 }
fn one_u32() -> u32 { 1 }
fn yes() -> bool { true }
fn default_gcd() -> f64 { 1.5 }
fn default_school() -> String { "physical".into() }
fn default_fight_len() -> f64 { 300.0 }
fn default_variance() -> f64 { 0.2 }
fn default_armor() -> f64 { 0.70 }
fn default_refresh() -> String { "pandemic".into() }

/// Calibration overlay, kept in its own file so the hand-written spec stays
/// clean: per damage source, the multiplier derived from the game's spell
/// modifiers (tools/effects.py) and a residual multiplier and crit-chance
/// offset fitted from observed logs (tools/calibrate.py); per buff, the
/// damage modifiers it carries in the game data.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Calibration {
    #[serde(default)]
    pub sources: HashMap<String, CalibEntry>,
    #[serde(default)]
    pub buffs: HashMap<String, BuffPatch>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CalibEntry {
    /// From talents and spec auras in the game data.
    #[serde(default = "one")]
    pub derived: f64,
    /// Residual: whatever the derived layer doesn't explain.
    #[serde(default = "one")]
    pub mult: f64,
    #[serde(default)]
    pub crit_bonus: f64,
}

/// Damage bonuses added to a spec buff (percent, keyed like BuffDef's).
#[derive(Debug, Deserialize, Clone, Default)]
pub struct BuffPatch {
    #[serde(default)]
    pub ability_pct: HashMap<String, f64>,
    #[serde(default)]
    pub school_pct: HashMap<String, f64>,
}

/// A pet ability as the engine sees it: name, `source` and DoT name
/// prefixed with `<pet>_`.
pub fn pet_ability(pet: &str, a: &AbilityDef) -> AbilityDef {
    let mut a = a.clone();
    a.name = format!("{pet}_{}", a.name);
    a.source = a.source.map(|s| format!("{pet}_{s}"));
    if let Some(d) = &mut a.dot {
        d.name = format!("{pet}_{}", d.name);
    }
    a
}

impl Spec {
    /// Fold a calibration overlay into the spec's damage definitions.
    pub fn apply_calibration(&mut self, cal: &Calibration) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        let mut patch = |a: &mut AbilityDef, prefix: &str| {
            let src = format!("{prefix}{}", a.source.as_ref().unwrap_or(&a.name));
            if let (Some(d), Some(c)) = (&mut a.damage, cal.sources.get(&src)) {
                d.mult *= c.derived * c.mult;
                d.crit_bonus += c.crit_bonus;
                seen.insert(src);
            }
            if let Some(d) = &mut a.dot {
                let name = format!("{prefix}{}", d.name);
                if let Some(c) = cal.sources.get(&name) {
                    d.mult *= c.derived * c.mult;
                    d.crit_bonus += c.crit_bonus;
                    seen.insert(name);
                }
            }
        };
        for a in &mut self.abilities {
            patch(a, "");
        }
        for p in &mut self.pets {
            let prefix = format!("{}_", p.name);
            for a in &mut p.abilities {
                patch(a, &prefix);
            }
        }
        for k in cal.sources.keys() {
            if !seen.contains(k) {
                return Err(format!("calibration: no damage source named '{k}'"));
            }
        }
        for (name, p) in &cal.buffs {
            let b = self.buffs.iter_mut().find(|b| &b.name == name)
                .ok_or_else(|| format!("calibration: no buff named '{name}'"))?;
            for (k, v) in &p.ability_pct {
                *b.ability_pct.entry(k.clone()).or_default() += v;
            }
            for (k, v) in &p.school_pct {
                *b.school_pct.entry(k.clone()).or_default() += v;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_applies_derived_residual_and_buff_patches() {
        let mut spec: Spec = toml::from_str(include_str!("../specs/retribution_herald.toml")).unwrap();
        let fv = |s: &Spec| s.abilities.iter().find(|a| a.name == "final_verdict").unwrap().damage.clone().unwrap();
        let aw = |s: &Spec| s.buffs.iter().find(|b| b.name == "avenging_wrath").unwrap().ability_pct.clone();
        let base = fv(&spec).mult;
        let cal: Calibration = toml::from_str(
            r#"
            [sources."final_verdict"]
            derived = 1.5
            mult = 2.0
            crit_bonus = 0.01
            [buffs."avenging_wrath".ability_pct]
            "final_verdict" = 20
            "#,
        )
        .unwrap();
        spec.apply_calibration(&cal).unwrap();
        assert!((fv(&spec).mult - base * 3.0).abs() < 1e-12);
        assert!((fv(&spec).crit_bonus - 0.01).abs() < 1e-12);
        assert_eq!(aw(&spec).get("final_verdict"), Some(&20.0));

        let bad: Calibration = toml::from_str("[buffs.\"nope\".ability_pct]\nx = 1").unwrap();
        assert!(spec.apply_calibration(&bad).is_err());
    }
}
