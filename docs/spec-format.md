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
a DoT's stacks), `cancel_dot`, `extend_buff`, and `if` (`{ type = "if", if = "<condition>", then = [...],
else = [...] }`). Abilities run `on_start` (before the hit), `on_crit` /
`on_noncrit` (per hit) and `on_cast` (after) effects, can have a `cast_time`,
and can report under a shared `source`. Damage can take a share of mastery
(`mastery_scale`) and grow with crit chance (`crit_scaling`).
Buffs can modify damage, crit (globally or per ability), crit damage, haste,
mastery, power, GCD, per-ability cast time and recharge rate, or store a
percentage of damage dealt for a later burst; they can stack with independent
timers (`overlap`), count stacks without multiplying bonuses (`flat`), extend
on refresh (`refresh_extend`), follow the target's health, and fire
`on_expire` effects. DoTs support hasted ticks, partial final ticks (or
`whole_ticks` only), `pandemic`/`extend`/`reset` refresh, stacks
(`max_stacks`, `per_stack`, `dot.X.stack`), extend-on-crit, on-tick and
on-crit effects, and `banked` rolling damage fed by hits.

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
  An instance whose energy can't pay for any action leaves right away and
  runs `on_expire`. A rotation ability needs a GCD, cast time, cooldown or
  cost, so the loop always advances.
- **Effects:**
  - `{ type = "summon", pet = "imp", count = 3, duration = 20 }` (`duration`
    optional),
  - `{ type = "dismiss", pet = "imp", count = 6, each = [...] }`: up to
    `count` (0 = all) instances leave, oldest first; `each` runs once per
    instance, before it goes, and its `on_expire` doesn't,
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
