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
    /// Continuous, unhasted pet energy regeneration (capped at initial energy).
    #[serde(default)]
    pub energy_regen: f64,
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
    /// Casting interrupts swings; restart the timer when the cast finishes.
    #[serde(default)]
    pub pause_while_casting: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcDef {
    pub ability: String,
    #[serde(default)]
    pub per_minute: f64,
    #[serde(default)]
    pub hasted: bool,
    /// Rolled on the player's direct hits instead of on a clock: SimC's real
    /// PPM (`per_minute`, with bad-luck protection) or a flat `chance` per hit,
    /// at most once per timestamp. Hits from `no_procs` abilities and pets
    /// don't count, and nothing is rolled while the ability's `requires_buff`
    /// is down (a proc that only runs during a buff).
    #[serde(default)]
    pub on_hit: bool,
    /// Also (or exclusively) roll on the owner's periodic damage.
    #[serde(default)]
    pub on_tick: bool,
    /// Restrict attempts to these damage source names (empty: every source).
    #[serde(default)]
    pub sources: Vec<String>,
    /// Flat chance per hit (on_hit procs without `per_minute`).
    #[serde(default)]
    pub chance: f64,
    /// Internal cooldown in seconds after a successful proc (on_hit procs).
    #[serde(default)]
    pub icd: f64,
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
    /// Off-hand weapon (dual wield): its own swing timer; the first swing
    /// comes at half its swing time.
    #[serde(default)]
    pub off_hand: Option<Weapon>,
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
    /// Seconds between re-checks of the rotation while nothing is usable.
    #[serde(default = "default_idle_poll")]
    pub idle_poll: f64,
    /// A cooldown counts as ready this close to its end (seconds). The event
    /// clock runs in whole microseconds, so 1e-6 makes a cooldown exactly N
    /// GCDs long come up for the Nth GCD, as in SimC.
    #[serde(default = "default_ready_tolerance")]
    pub ready_tolerance: f64,
}

impl Default for Fight {
    fn default() -> Self {
        Fight { duration: 300.0, variance: 0.2, physical_mult: default_armor(), idle_poll: default_idle_poll(), ready_tolerance: default_ready_tolerance() }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceDef {
    pub max: f64,
    #[serde(default)]
    pub start: f64,
    /// Passive regeneration per second (0: the resource only changes by costs and gains).
    #[serde(default)]
    pub regen: f64,
    /// Regeneration scales with the player's current haste (buffs included).
    #[serde(default)]
    pub regen_hasted: bool,
    /// Multiply the resource cap / base regen by 1 + this * player mastery.
    #[serde(default)]
    pub max_per_mastery: f64,
    #[serde(default)]
    pub regen_per_mastery: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AbilityDef {
    pub name: String,
    /// Start the ability and its DoT immediately, locking actions for cast_time.
    /// The DoT's duration and tick interval use the cast's snapshotted speed.
    /// (A simpler alternative to the `channel` table below.)
    #[serde(default)]
    pub snapshot_channel: bool,
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
    /// Pet cooldown begins when its cast starts, rather than when it resolves.
    #[serde(default)]
    pub cooldown_on_start: bool,
    #[serde(default = "one_u32")]
    pub charges: u32,
    /// Cast time in seconds (0 = instant). The ability resolves when the cast
    /// finishes; the player can't start anything else meanwhile.
    #[serde(default)]
    pub cast_time: f64,
    #[serde(default = "yes")]
    pub cast_hasted: bool,
    /// A channel starts immediately and delivers its attached DoT's whole ticks.
    #[serde(default)]
    pub channel: Option<ChannelDef>,
    #[serde(default)]
    pub cost: HashMap<String, f64>,
    /// Per-resource expressions replacing the corresponding fixed cost.
    #[serde(default)]
    pub cost_expr: HashMap<String, String>,
    /// Spend up to this much of a resource in total (at least `cost`), like
    /// a finisher spending every combo point. `spent.<resource>` in
    /// expressions is what the last cast that paid it paid.
    #[serde(default)]
    pub cost_max: HashMap<String, f64>,
    #[serde(default)]
    pub gain: HashMap<String, f64>,
    #[serde(default)]
    pub damage: Option<DamageDef>,
    #[serde(default)]
    pub dot: Option<DotDef>,
    /// If this buff is up, the ability costs nothing and consumes one stack.
    #[serde(default)]
    pub free_with: Option<String>,
    /// If a free-cast buff arrives during a hard cast, use its cost reduction
    /// without consuming a stack. Instant casts still consume it normally.
    #[serde(default)]
    pub preserve_late_free: bool,
    /// Ability is only usable while this buff is up.
    #[serde(default)]
    pub requires_buff: Option<String>,
    /// Effects applied as the ability goes off, before its damage (a spell's
    /// travel time: buffs gained or spent on cast affect the hit).
    #[serde(default)]
    pub on_start: Vec<Effect>,
    /// Player effects after cast time is determined, before the cast begins.
    #[serde(default)]
    pub on_begin: Vec<Effect>,
    #[serde(default)]
    pub on_cast: Vec<Effect>,
    /// Effects when a direct-damage hit crits.
    #[serde(default)]
    pub on_crit: Vec<Effect>,
    /// Effects when a direct-damage hit doesn't crit.
    #[serde(default)]
    pub on_noncrit: Vec<Effect>,
    /// Effects when a cast is paid by `free_with` (applied as the buff is consumed).
    #[serde(default)]
    pub on_free: Vec<Effect>,
    /// Its hits never trigger on-hit procs (gear proc damage).
    #[serde(default)]
    pub no_procs: bool,
    /// Report damage under this source name instead of the ability's own
    /// (several variants of one spell, e.g. hardcast vs instant).
    #[serde(default)]
    pub source: Option<String>,
    /// Only ever fired by triggers/procs, never chosen by the rotation.
    #[serde(default)]
    pub triggered_only: bool,
}

/// Player channel. Tick interval and tick-zero behaviour come from its DoT.
#[derive(Debug, Deserialize, Clone)]
pub struct ChannelDef {
    pub ticks: u32,
    /// Tick count from an expression instead, evaluated as the channel starts.
    #[serde(default)]
    pub ticks_expr: Option<String>,
    /// Unhasted recovery after natural completion; interrupted channels skip it.
    #[serde(default)]
    pub finish_delay: f64,
    /// Re-evaluated after each tick, once the global cooldown is over.
    #[serde(default)]
    pub interrupt_if: Option<String>,
    /// Auto attacks that come due while channeling are suppressed and re-armed
    /// a swing later (SimC's `interrupt_auto_attack`, e.g. Boomstick).
    #[serde(default)]
    pub interrupt_auto_attack: bool,
    /// Runs on both natural completion and interruption.
    #[serde(default)]
    pub on_finish: Vec<Effect>,
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
    /// Flat damage added to the coefficient part (gear procs: item-level scaled).
    #[serde(default)]
    pub amount: f64,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
    /// Share of mastery applied when `mastery` is set (e.g. 0.75 for a bonus
    /// rolled uniformly between 50% and 100% of mastery).
    #[serde(default = "one")]
    pub mastery_scale: f64,
    /// Independent factors of the same mastery bonus (e.g. a dual-school hit).
    #[serde(default = "one_u32")]
    pub mastery_power: u32,
    /// Damage is further increased by the player's crit chance.
    #[serde(default)]
    pub crit_scaling: bool,
    /// Crit chance is multiplied by this (Improved Shatter: x1.5).
    #[serde(default = "one")]
    pub crit_chance_multiplier: f64,
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
    /// Only critical hits contribute to the bank.
    #[serde(default)]
    pub crit_only: bool,
    /// Optional condition checked when the hit lands.
    #[serde(default, rename = "if")]
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DotDef {
    pub name: String,
    /// Each application adds a stack with its own duration and expiry effects.
    #[serde(default)]
    pub overlap: bool,
    #[serde(default)]
    pub on_expire: Vec<Effect>,
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
    /// Optional dynamic base times, evaluated on application / each tick.
    #[serde(default)]
    pub duration_expr: Option<String>,
    #[serde(default)]
    pub tick_time_expr: Option<String>,
    #[serde(default = "yes")]
    pub hasted: bool,
    #[serde(default = "default_school")]
    pub school: String,
    #[serde(default)]
    pub mastery: bool,
    #[serde(default = "one")]
    pub mastery_scale: f64,
    #[serde(default = "one_u32")]
    pub mastery_power: u32,
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
    /// 30% of duration), "extend" (add full duration to what's left), "reset",
    /// or "rolling" (the time to the next tick plus a full duration, with the
    /// damage of the ticks left rolled into the new ticks: nothing is lost).
    #[serde(default = "default_refresh")]
    pub refresh: String,
    /// With `refresh = "rolling"`: the new duration is pandemic-capped
    /// (duration plus up to 30% of it carried over) instead of the time to
    /// the next tick plus a full duration.
    #[serde(default)]
    pub rolling_pandemic: bool,
    #[serde(default)]
    pub on_tick: Vec<Effect>,
    /// Effects when a tick crits.
    #[serde(default)]
    pub on_crit: Vec<Effect>,
    /// Shares of periodic damage fed into rolling DoTs, just as on direct hits.
    #[serde(default)]
    pub feed: Vec<FeedDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutoAttackDef {
    #[serde(default = "one")]
    pub mult: f64,
    /// Each swing executes this ability instead of dealing weapon damage.
    #[serde(default)]
    pub replace_with: Option<String>,
    /// Same, for off-hand swings (`player.off_hand`). Without it they deal
    /// half the off-hand weapon's damage, as `auto_attack_off_hand`.
    #[serde(default)]
    pub off_hand_replace_with: Option<String>,
    #[serde(default)]
    pub on_hit: Vec<Effect>,
    /// Each swing re-checks the rotation (a swing's procs may make something
    /// castable). False: only the GCD, cooldowns, costs and the idle poll do.
    #[serde(default = "yes")]
    pub wakes: bool,
}

/// One uniformly selected, eligible alternative of a `choose` effect.
#[derive(Debug, Deserialize, Clone)]
pub struct EffectChoice {
    #[serde(rename = "if", default)]
    pub condition: Option<String>,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// Select one eligible choice uniformly; do nothing if none is eligible.
    Choose { choices: Vec<EffectChoice> },
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
        /// A charge given back keeps the recharge in progress (SimC's
        /// cooldown_t::reset); without it the recharge starts over.
        #[serde(default)]
        keep_recharge: bool,
    },
    ReduceCooldown {
        ability: String,
        #[serde(default)]
        seconds: f64,
        /// Seconds from a condition-language expression instead (e.g. per
        /// resource spent or per buff stack).
        #[serde(default)]
        seconds_expr: Option<String>,
        #[serde(default = "one")]
        chance: f64,
    },
    Gain {
        resource: String,
        #[serde(default)]
        amount: f64,
        /// Amount from a condition-language expression instead (e.g. a buff's stacks).
        #[serde(default)]
        amount_expr: Option<String>,
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
    /// Apply another ability's DoT as if that ability had been cast, at
    /// `strength` (1 = a normal application). A strength other than 1 needs
    /// `refresh = "rolling"`.
    ApplyDot {
        dot: String,
        #[serde(default = "one")]
        strength: f64,
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
        /// Per-instance overrides (e.g. an empowered summon).
        #[serde(default)]
        energy: Option<f64>,
        #[serde(default = "one")]
        damage_mult: f64,
    },
    /// Make up to `count` (0 = all) active instances of a pet leave, oldest
    /// first, applying `each` once per instance (`self.*` refers to it) and
    /// skipping its `on_expire`.
    Dismiss {
        pet: String,
        #[serde(default)]
        count: u32,
        /// Other eligible pets in a random sample without replacement. Only
        /// selections of `pet` are dismissed; omitted: select only `pet`.
        #[serde(default)]
        pool: Vec<String>,
        #[serde(default)]
        each: Vec<Effect>,
    },
    /// Apply effects once per currently active instance, with its self context.
    ForEachPet {
        pet: String,
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
        /// Proc chance from a condition-language expression (overrides `chance`).
        #[serde(default)]
        chance_expr: Option<String>,
        /// Number of executions, evaluated once after a successful roll.
        #[serde(default)]
        count: Option<String>,
        /// Delay between executions of a multi-trigger.
        #[serde(default)]
        interval: f64,
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
    /// Versatility points (percent: 2 = +0.02 versatility).
    #[serde(default)]
    pub versatility_pct: f64,
    /// Per-stack gear stats ("primary", "crit_rating", "haste_rating",
    /// "mastery_rating", "versatility_rating"). With gear lines they are
    /// turned into the percent fields above from the character's own stats;
    /// without, they do nothing.
    #[serde(default)]
    #[allow(dead_code)]
    pub stats: HashMap<String, f64>,
    #[serde(default)]
    pub haste_pct: f64,
    /// Auto-attack speed only (percent).
    #[serde(default)]
    pub attack_speed_pct: f64,
    /// Adds this share of the player's current haste to `attack_speed_pct`
    /// (1 = "increased by an additional amount equal to your haste").
    #[serde(default)]
    pub attack_speed_per_haste: f64,
    /// Reduces the global cooldown only (percent speed-up).
    #[serde(default)]
    pub gcd_pct: f64,
    /// Per-ability recharge speed-up while up (percent; 50 = recharges 1.5x faster).
    #[serde(default)]
    pub cooldown_rate_pct: HashMap<String, f64>,
    /// Per-DoT tick rate increase while up, keyed by DoT name (percent per
    /// stack; 40 = ticks 1.4x as often). Takes effect from the next tick.
    #[serde(default)]
    pub tick_rate_pct: HashMap<String, f64>,
    /// While up, this percent of all damage dealt is stored (for a later burst).
    #[serde(default)]
    pub store_damage_pct: f64,
    /// Regeneration speed-up per resource (percent per stack).
    #[serde(default)]
    pub resource_regen_pct: HashMap<String, f64>,
    /// Per-ability cast time reduction while up (percent; 100 = instant).
    #[serde(default)]
    pub cast_time_pct: HashMap<String, f64>,
    /// Reapplying while up adds the duration to what's left instead of
    /// restarting it.
    #[serde(default)]
    pub refresh_extend: bool,
    /// Adding/setting stacks preserves an already active shared timer.
    #[serde(default = "yes")]
    pub refresh_duration: bool,
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
    /// Duration grows with haste: x (1 + this x current haste, buffs included) when applied.
    #[serde(default)]
    pub haste_duration: f64,
    /// While up, hits are copied as extra damage (see EchoDef).
    #[serde(default)]
    pub echo: Option<EchoDef>,
}

/// A buff that copies a share of each hit as another ability's damage
/// (e.g. "X% of damage dealt is dealt again as shadow damage"). The copy is
/// final damage: versatility and buffs aren't applied again, only the echo
/// ability's own multiplier (calibration) and crit roll. Echoes don't echo.
#[derive(Debug, Deserialize, Clone)]
pub struct EchoDef {
    /// Ability whose damage definition (and result row) the copy uses.
    pub ability: String,
    /// Percent of the hit copied, per stack of the buff.
    pub pct: f64,
    /// Per-school override of `pct`, keyed by the school of the copied hit.
    #[serde(default)]
    pub school_pct: HashMap<String, f64>,
    /// Only copy hits from these result rows (empty: every source).
    #[serde(default)]
    pub sources: Vec<String>,
    /// Copy the damage before armor (default) or after.
    #[serde(default = "yes")]
    pub pre_armor: bool,
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
fn default_idle_poll() -> f64 { 0.5 }
fn default_ready_tolerance() -> f64 { 1e-9 }

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
