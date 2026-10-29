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
