# Spec format, briefly

```toml
[[abilities]]
name = "hammer_of_wrath"
requires_buff = "avenging_wrath"
cooldown = 7.5
cooldown_hasted = true
charges = 2
gain = { holy_power = 1 }
damage = { ap_coeff = 2.15807, school = "holy", mastery = true }
on_cast = [
  { type = "trigger", ability = "blade_of_justice_walk_into_light" },
  { type = "trigger", ability = "hammer_of_wrath_second_sunrise", chance = 0.20, delay = 0.325 },
]
on_crit = [{ type = "trigger", ability = "sun_sear_apply" }]

[[rotation]]
action = "hammer_of_wrath"
if = "cooldown.hammer_of_wrath.charges=2"
```

Effects: `buff` (optionally with its own `duration`), `consume`,
`reset_cooldown`, `reduce_cooldown`, `gain`, `trigger` (with `chance`,
`delay`, `mastery_chance`), `extend_dot`, `dot_stack` (add, remove or `set`
a DoT's stacks), `cancel_dot`, `apply_dot`, `extend_buff`, and `if` (`{ type = "if", if = "<condition>", then = [...],
else = [...] }`). Abilities run `on_start` (before the hit), `on_crit` /
`on_noncrit` (per hit) and `on_cast` (after) effects, can have a `cast_time`,
and can report under a shared `source`. Damage can take a share of mastery
(`mastery_scale`), grow with crit chance (`crit_scaling`), or have its crit
chance multiplied (`crit_chance_multiplier`, e.g. Improved Shatter's x1.5).
Buffs can modify damage, crit (globally or per ability), crit damage, haste,
mastery, power, GCD, per-ability cast time and recharge rate, or store a
percentage of damage dealt for a later burst; they can stack with independent
timers (`overlap`), count stacks without multiplying bonuses (`flat`), extend
on refresh (`refresh_extend`), retain their original expiry when stacked
(`refresh_duration = false`), follow the target's health, fire
`on_expire` effects, copy a share of each hit as another ability's damage
(`echo`), grow with haste (`haste_duration`) or change auto-attack speed only
(`attack_speed_pct`, `attack_speed_per_haste`). DoTs support hasted ticks, partial final ticks (or
`whole_ticks` only), `pandemic`/`extend`/`reset`/`rolling` refresh, tick-rate buffs, stacks
(`max_stacks`, `per_stack`, `dot.X.stack`), extend-on-crit, on-tick and
on-crit effects, and `banked` rolling damage fed by direct hits or ticks. Feed entries may
restrict contributions with `crit_only = true`, an `if` expression, and a
`chance` (default 1: every hit; each entry rolls its own random stream).
`mastery_power` applies independent copies of the mastery factor for
multischool damage (default 1).

A DoT with `overlap = true` keeps an independent expiration for each
application, up to `max_stacks`, replacing the oldest at the cap. Its ticks
share a cadence and, with `per_stack`, deal damage proportional to the number
of active stacks; `on_expire` runs once per stack on natural expiration. This
is an approximation when each application has its own tick cadence.
`snapshot_channel = true` (a simpler alternative to the `channel` table
below) starts the ability and its DoT at cast start, locks actions for the
cast duration, and snapshots the cast speed for its ticks and duration.
`preserve_late_free = true` leaves a `free_with` buff unconsumed if it arrives
while a hard cast is underway, while still making that cast free.

An ability paid by its `free_with` buff runs `on_free` effects as the buff is
consumed (a set bonus "when Divine Purpose is consumed"). `damage.amount` adds
flat damage (gear procs, scaled from item level), and `no_procs` keeps an
ability's hits from triggering on-hit procs. Buffs also take
`versatility_pct`, and gear `stats` per stack (`primary`, `crit_rating`, ...),
which runs with gear lines turn into the character's own percent bonuses.

`[[procs]]` fire an ability at random: by default as a Poisson clock
(`per_minute`, `hasted`); with `on_hit = true` they are rolled on the
player's direct hits as SimC's real PPM (one attempt per timestamp, the time
since the last attempt capped at 3.5s, bad-luck protection), or with a flat
`chance` per hit, plus an optional internal cooldown `icd`. Nothing is rolled
while the ability's `requires_buff` is down, which makes "while X is up, hits
can proc Y" plain data. Use `on_tick = true` to include periodic hits (with or without `on_hit`),
and `sources = ["name", ...]` to restrict eligible damage sources. Hits from
pets and `no_procs` abilities don't roll.
Each on-hit proc has its own `hit_proc/<ability>` random stream.

Trinket effects and set bonuses are spec fragments in this same format, added
with gear lines; see [gear.md](gear.md#trinkets-and-set-bonuses).

## Energy, combo points, dual wield

```toml
[resources.energy]
max = 100
start = 100
regen = 10                  # per second, continuous
regen_hasted = true         # x current haste, buffs included

[player]
weapon = { min = 0, max = 0, speed = 1.8 }
off_hand = { min = 0, max = 0, speed = 1.8 }   # own swing timer, first swing at half

[auto_attack]
replace_with = "mh_swing"
off_hand_replace_with = "oh_swing"   # without it: half the off-hand's weapon damage
wakes = false               # swings don't re-check the rotation (default true)

[fight]
idle_poll = 0.1             # re-check while nothing is usable (default 0.5)

[[abilities]]
name = "eviscerate"
cost = { energy = 35, combo_points = 1 }
cost_max = { combo_points = 7 }     # spend up to 7
damage = { ap_coeff = 0.37, scale = "spent.combo_points" }
on_cast = [{ type = "gain", resource = "energy", amount_expr = "4*spent.combo_points" }]

[[buffs]]
name = "shadow_blades"
duration = 16
echo = { ability = "shadow_blades_attack", pct = 20, school_pct = { physical = 27.4 }, sources = ["eviscerate"], pre_armor = false }
```

An idle player wakes when regeneration pays for a rotation ability's cost.
`spent.<resource>` is what the last cast that paid that resource paid;
`trigger` takes `chance_expr`, and `cooldown.X.duration` is the current
(hasted) cooldown. An echo's copy is final damage (no versatility or buffs
again), scaled by the echo ability's own multiplier and per stack of the buff;
echoes don't copy echoes. Attack-speed buffs resize a swing in progress.

## Rage, rolling bleeds, cooldown ties

```toml
[fight]
ready_tolerance = 1e-5      # cooldowns this close to done count as ready (default 1e-9)

[[abilities]]
name = "bladestorm"
channel = { ticks = 7, ticks_expr = "7+buff.imminent_demise.stack" }   # tick count from an expression

[abilities.dot]
name = "deep_wounds"
refresh = "rolling"         # SimC's rolling periodic: nothing is lost on a refresh

on_cast = [
  { type = "reduce_cooldown", ability = "avatar", seconds_expr = "spent.rage/20" },
  { type = "reset_cooldown", ability = "overpower", chance = 0.3, keep_recharge = true },
]
```

A `rolling` refresh keeps the tick cadence, runs a full duration from the next
tick, and rolls the damage of the ticks left into the new ones (SimC's
`rolling_ta_multiplier`); `rolling_pandemic` and `apply_dot`'s `strength`
change that (see "Tick-rate buffs and partial rolling applications" below). A channel's `ticks_expr` is evaluated as it starts
(with `tick_time_expr` on its DoT, more ticks in the same time).
`reduce_cooldown` takes `seconds_expr`. `reset_cooldown` with `keep_recharge`
gives a charge back and lets the recharge in progress carry on, as SimC's
`cooldown_t::reset` does (without it the recharge starts over). The event
clock runs in whole microseconds, so a hasted cooldown exactly N hasted GCDs
long (Mortal Strike: 6s, 4 GCDs) needs `ready_tolerance` to come up on the
Nth GCD the way SimC's does.

## Pets and summons

A `[[pets]]` entry defines something you summon. Any number of instances can
be out at once, each with its own timer, energy, per-instance cooldowns and
action loop. `specs/examples/summoner.toml` uses every feature below.

```toml
[[pets]]
name = "imp"
duration = 40               # 0 = until dismissed
max_active = 0              # cap; a new summon replaces the one closest to expiring
energy = 100                # optional; abilities' cost = { energy = .. } spend it
energy_regen = 10           # optional, per second, capped at energy; default 0
owner_coeff = { sp_from_sp = 0.6 }   # also sp_from_ap, ap_from_ap (default 1), ap_from_sp
inherit_haste = true        # default; false: unhasted
inherit_crit = true         # default; false: uses `crit` instead
delay = 0.0                 # before its first action and swing
melee = { ability = "melee", speed = 2.0 }   # optional auto attack (hasted by default)
on_summon = [...]           # effects as an instance arrives
on_expire = [...]           # ... and as it leaves on its own (timer, out of energy)

[[pets.abilities]]          # ordinary ability definitions
name = "firebolt"
cast_time = 2.0
gcd = 0
cost = { energy = 20 }
cooldown_on_start = true    # optional: start pet cooldown at cast start
damage = { sp_coeff = 0.9, school = "fire", mastery = true }

[[pets.rotation]]           # optional; default: every ability in order,
action = "firebolt"         # except triggered_only ones and the melee ability
if = "self.energy>=20"
```

- **Names.** A pet's abilities, `source`s and DoTs are named `<pet>_<name>`
  everywhere outside the pet: in reports (`imp_firebolt`, the name
  `tools/observe.py` gives pet sources), calibration files, buffs'
  `ability_pct`/`ability_crit_pct`, and triggers from the owner. Inside the
  pet, local names work too.
- **Damage.** Coefficients scale with the pet's power, taken from the
  owner's current attack and spell power (power buffs included) through
  `owner_coeff`. Versatility, mastery (for `mastery = true`), `global_mult`
  and armor are the owner's. Crit and haste follow `inherit_crit` /
  `inherit_haste`; the inherited crit includes the owner's global crit buffs.
- **Buffs.** `damage_pct` and `school_pct` reach pets only from buffs with
  `pets = true`. `pet_pct = { imp = 10 }` is a per-pet bonus.
- **Action loop.** Each instance picks the first usable entry of its
  rotation: off its own cooldown, affordable, `requires_buff` up, condition
  true. A cast takes its (hasted) cast time, then the instance waits out the
  ability's GCD (hasted, at least 0.75s; `gcd = 0` for back-to-back casts).
  Without regeneration, an instance whose energy can't pay for any action leaves
  right away and runs `on_expire`; with regeneration it waits for energy,
  waking a microsecond past the moment it can pay (the event clock's
  resolution, so a pet never stalls on its own timestamp).
  `melee.pause_while_casting = true` restarts the swing timer after a cast
  that overlaps a scheduled swing. A rotation ability needs a GCD, cast time, cooldown or
  cost, so the loop always advances.
- **Effects:**
  - `{ type = "summon", pet = "imp", count = 3, duration = 20 }` (`duration`
    optional). `energy` and `damage_mult` optionally override starting energy
    and direct pet damage for each summoned instance,
  - `{ type = "dismiss", pet = "imp", count = 6, each = [...] }`: up to
    `count` (0 = all) instances leave, oldest first; `each` runs once per
    instance, before it goes, and its `on_expire` doesn't. `pool = ["hound"]`
    samples up to `count` active instances without replacement across the named
    pet and pool; only instances of the named pet are dismissed,
  - `{ type = "extend_pet", pet = "hound", seconds = 15 }`: every active
    instance,
  - `buff` takes `set = "<expression>"` to set the stack count instead of
    adding (`0` removes the buff), e.g. one stack per demon out.
- **Conditions.** `pet.<name>.active` (also `count`, `up`) is the number of
  active instances, `pet.<name>.remains` the longest time one has left.
  `self.energy` and `self.remains` refer to the instance an effect or pet
  action runs for: in pet rotations, `on_summon`/`on_expire`, `dismiss`'s
  `each`, and anything those trigger without a delay. `damage.scale = "<expression>"`
  multiplies a hit, e.g. `scale = "self.energy/100"` for an
  Implosion-style hit per dismissed imp.
- **Limits.** A pet ability's DoT is shared by all instances, like any DoT,
  and ticks at the owner's haste. `reset_cooldown`/`reduce_cooldown` don't
  reach pets' per-instance cooldowns. Pet events don't wake an idle owner:
  the owner reacts at its next decision.

Randomness stays keyed by name: pet sources get `crit/imp_firebolt`
streams, shared by all instances, and pet effects' chance rolls get
`roll/<pet ability>/...` and `roll/pet/<pet>/on_summon/...` streams.

## Channels and resource expressions

A player ability can use `channel = { ticks = 7, finish_delay = 0.25,
interrupt_if = "mana.pct>=95", interrupt_auto_attack = true, on_finish = [...] }`
with an attached DoT. With `interrupt_auto_attack`, swings that come due while
the channel runs deal no damage and re-arm a swing later (SimC's
`interrupt_auto_attack`, e.g. Boomstick).
Its tick interval, haste scaling and tick-on-apply behavior come from the DoT;
the channel occupies the player until its last tick or until the interrupt
condition holds after a tick once the GCD has elapsed. `on_finish` runs for
both completion and interruption; `finish_delay` applies only on completion.
Channels require a positive tick interval and count, no cast time, an unbanked
DoT, and a player ability.

Resources may specify `max_per_mastery` and `regen_per_mastery`, multiplying
their base cap, starting value and regeneration by `1 + coefficient * player.mastery`.
Buffs can multiply regeneration with `resource_regen_pct = { mana = 100 }`.
An ability can replace fixed costs with `cost_expr = { mana = "mana.max*0.02" }`;
negative expression results are clamped to zero. Expressions also expose
`resource.max`, `resource.pct`, `mastery`, `haste`, `gcd.remains`,
`prev_gcd.<ability>` and `cooldown.X.full_recharge_time` (time until every
charge is back, 0 at full charges). A `gain` with `amount_expr` may subtract resource.
A trigger may use `count = "<expression>"` and `interval` to schedule repeated
activations; the count is evaluated at the trigger and limited to 1000.

## Additional effect timing and selection

`on_begin` runs for an owner action after its cast time is determined and
before the cast starts, including instant casts. `choose` uniformly selects
one eligible choice: `{ type = "choose", choices = [{ if = "condition", effects = [...] }, ...] }`.
The `if` is optional; no eligible choices means no effect. Each selection has
its own named random stream.

`{ type = "for_each_pet", pet = "imp", each = [...] }` executes effects once
for each active instance with `self` bound to that instance. It snapshots the
instances at entry: newly summoned or replaced instances do not participate,
and an instance dismissed before its turn is skipped. Delayed triggers use
the normal pet event lifetime rules.

DoTs may specify `duration_expr` and `tick_time_expr` to compute their base
duration on application and base tick interval when scheduling each tick.
Haste and the existing refresh rules apply to those values.

For shared buffs, `refresh_duration = false` preserves the existing expiration
when stacks are added or set. `extend_buff` accepts negative seconds to shorten
an active timer, expiring it no earlier than the current time.

## Tick-rate buffs and partial rolling applications

```toml
[[buffs]]
name = "screams_of_the_void"
duration = 3
tick_rate_pct = { shadow_word_pain = 40, vampiric_touch = 40 }   # per stack, by DoT name

[[abilities]]
name = "shadow_word_madness"
[abilities.dot]
name = "shadow_word_madness"
sp_coeff = 2.61625
tick_time = 2
duration = 6
refresh = "rolling"
rolling_pandemic = true    # pandemic-capped duration on a refresh

[[abilities]]
name = "tentacle_slam"
on_cast = [{ type = "apply_dot", dot = "shadow_word_madness", strength = 0.25 }]
```

- **`tick_rate_pct`** divides the named DoTs' tick interval by
  `1 + Σ pct/100 × stacks` while the buff is up. As in SimC, the interval is
  re-evaluated when each tick is scheduled: gaining or losing the buff changes
  the tick after the one already pending. The tick in flight still deals a
  whole tick, and the shortened final tick is measured against the interval it
  was scheduled with. A channel takes the rate in effect when it starts.
- **`rolling_pandemic = true`** (only with `refresh = "rolling"`) makes a
  refresh's new duration the `pandemic` one (duration plus up to 30% of it
  carried over) instead of the time to the next tick plus a full duration.
  The damage is still rolled: the per-tick multiplier becomes
  `(ticks_left × old_mult + duration/tick × strength) / new_ticks_left`
  (SimC's `rolling_ta_multiplier`), ticks counted fractionally from the
  pending tick on. A fresh application ticks at its `strength`. Rolling DoTs
  can't be `overlap` or `banked`. `extend_dot` adds time at the current
  multiplier.
- **`apply_dot`** applies another ability's DoT (by DoT name) as if that
  ability had been cast, without its direct damage, cost or cooldown.
  `strength` (default 1) scales the new ticks and needs a `rolling` DoT.
  Channel DoTs can't be applied this way.
