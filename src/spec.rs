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
    pub rotation: Vec<RotationEntry>,
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
    pub attack_power: f64,
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
    #[serde(default)]
    pub on_cast: Vec<Effect>,
    /// Effects when a direct-damage hit crits.
    #[serde(default)]
    pub on_crit: Vec<Effect>,
    /// Only ever fired by triggers/procs, never chosen by the rotation.
    #[serde(default)]
    pub triggered_only: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DamageDef {
    #[serde(default)]
    pub ap_coeff: f64,
    /// Fraction of weapon damage added (normalized to the weapon's own speed).
    #[serde(default)]
    pub weapon_mult: f64,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
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
    /// Number of separate hits per cast.
    #[serde(default = "one_u32")]
    pub hits: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DotDef {
    pub name: String,
    pub ap_coeff: f64,
    pub tick_time: f64,
    pub duration: f64,
    #[serde(default = "yes")]
    pub hasted: bool,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
    #[serde(default = "one")]
    pub mult: f64,
    #[serde(default)]
    pub crit_bonus: f64,
    #[serde(default)]
    pub crit_damage_bonus: f64,
    /// Seconds added to the DoT each time a tick crits.
    #[serde(default)]
    pub extend_on_crit: f64,
    /// Deal one tick immediately when freshly applied.
    #[serde(default)]
    pub tick_on_apply: bool,
    /// What reapplying does while ticking: "pandemic" (default: carry over up to
    /// 30% of duration), "extend" (add full duration to what's left) or "reset".
    #[serde(default = "default_refresh")]
    pub refresh: String,
    #[serde(default)]
    pub on_tick: Vec<Effect>,
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
    #[serde(default)]
    pub crit_pct: f64,
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
/// clean: per damage source, a residual multiplier and crit-chance offset
/// fitted from observed logs.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Calibration {
    #[serde(default)]
    pub sources: HashMap<String, CalibEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CalibEntry {
    #[serde(default = "one")]
    pub mult: f64,
    #[serde(default)]
    pub crit_bonus: f64,
}

impl Spec {
    /// Fold a calibration overlay into the spec's damage definitions.
    pub fn apply_calibration(&mut self, cal: &Calibration) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for a in &mut self.abilities {
            if let (Some(d), Some(c)) = (&mut a.damage, cal.sources.get(&a.name)) {
                d.mult *= c.mult;
                d.crit_bonus += c.crit_bonus;
                seen.insert(a.name.clone());
            }
            if let Some(d) = &mut a.dot
                && let Some(c) = cal.sources.get(&d.name) {
                    d.mult *= c.mult;
                    d.crit_bonus += c.crit_bonus;
                    seen.insert(d.name.clone());
                }
        }
        for k in cal.sources.keys() {
            if !seen.contains(k) {
                return Err(format!("calibration: no damage source named '{k}'"));
            }
        }
        Ok(())
    }
}
