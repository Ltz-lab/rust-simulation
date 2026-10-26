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

Effects: `buff`, `consume`, `reset_cooldown`, `reduce_cooldown`, `gain`,
`trigger` (with `chance`, `delay`, `mastery_chance`), `extend_dot`,
`extend_buff`, and `if` (`{ type = "if", if = "<condition>", then = [...],
else = [...] }`). Abilities run `on_start` (before the hit), `on_crit` /
`on_noncrit` (per hit) and `on_cast` (after) effects, can have a `cast_time`,
and can report under a shared `source`.
Buffs can modify damage, crit (globally or per ability), crit damage, haste,
mastery, power, GCD and per-ability recharge rate, or store a percentage of
damage dealt for a later burst; they can stack with independent timers
(`overlap`), follow the target's health, and fire `on_expire` effects. DoTs
support hasted ticks, partial final ticks, `pandemic`/`extend`/`reset` refresh,
extend-on-crit, on-tick effects, and `banked` rolling damage fed by hits.
