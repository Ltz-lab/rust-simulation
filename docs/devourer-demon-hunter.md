# Devourer Demon Hunter (Void-Scarred)


`specs/devourer_voidscarred.toml`, built like the other specs: coefficients,
costs, cooldowns and durations from DB2 (SimC's `spell_query` on build
12.1.5.70575), mechanics read from SimC's demon hunter module
(`sc_demon_hunter.cpp`) and its default Void-Scarred APL, and pacing observed
in SimC's combat log. Everything below is generic engine data (resources,
banked DoTs, channels, cooldown resets, one-shot buffs), not class code.

The spec runs on two resources and one counter:

- **Fury** (140) is spent by Void Ray (100) and built by Consume (+8), Devour
  (+16), Soul Immolation (+6 a tick) and by consuming soul fragments (+4 each).
- **Soul fragments** (cap 10) are shattered by ~25% of direct hits (Shattered
  Souls + Sweet Release), by Consume/Devour (+2, Predator's Thirst) and by Soul
  Immolation (+1 every other tick). Reap and Cull eat 4, or 10 while Moment of
  Craving is up (the Void Ray's last tick grants it).
- **Void Metamorphosis stacks**: every fragment consumed outside Metamorphosis
  adds one; 35 stacks open Void Metamorphosis. Inside it fragments build
  Collapsing Star stacks instead, and 30 of those open Collapsing Star.

Void Metamorphosis drains fury and ends at zero; the spec models the window as
a fixed 24s buff (the log averages 25.2s of uptime, 24s of it spent on
actions once SimC's ~150ms per-action latency is taken out). Inside it Consume
becomes Devour, Reap becomes Cull, Voidblade becomes Pierce the Veil and
Hungering Slash becomes Reaper's Toll, so the two pairings share one cast and
one cooldown, with the damage row chosen by the buff. Each hero ability
triggers one Voidsurge per Metamorphosis (the first also repeats through
Volatile Instinct); Voidsurge is a +10%-per-stack, 12s buff. Hungering Slash
windows come from Voidblade/Pierce The Veil and The Hunt/Predator's Wake, and
Hungering Slash/Reaper's Toll grant the Vengeful Retreat used off the GCD.

Damage numbers are the game's spell-power coefficients, with Mastery: Monster
Within's 1.2× (Enduring Torment) as `mastery_scale` and the Metamorphosis
window's extra 0.4 × mastery as a flat buff modifier. Everything else the game
data describes (talents, the spec aura, hero-tree passives) is read by
`tools/effects.py` into `derived/devourer_voidscarred.toml`; the residual
multipliers are fitted per source in `calibration/devourer_voidscarred.toml`.

## Results

Reference: SimC's Midnight S2 sample Devourer profile (Void-Scarred), core kit
only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk,
5,000 iterations.

| | DPS |
|---|---:|
| reference | 164,261 |
| azsim | 166,689 (±133) |
| difference | **+1.48%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| cull_damage | 35,054 | 40,664 | +16.0% | 34.7 | 40.2 |
| reap_damage | 23,116 | 23,026 | -0.4% | 30.9 | 30.8 |
| devour | 15,481 | 16,990 | +9.8% | 46.1 | 50.6 |
| voidsurge | 15,501 | 15,815 | +2.0% | 24.8 | 25.3 |
| void_ray_tick_meta | 12,935 | 11,979 | -7.4% | 293.6 | 272.0 |
| reapers_toll_damage | 12,657 | 8,142 | -35.7% | 17.0 | 11.0 |
| void_ray_tick | 9,914 | 11,825 | +19.3% | 313.6 | 374.5 |
| consume | 10,207 | 6,511 | -36.2% | 32.7 | 21.0 |
| burning_blades | 7,429 | 8,837 | +18.9% | 61.3 | 73.0 |
| pierce_the_veil_damage | 6,378 | 4,314 | -32.4% | 10.3 | 7.0 |
| collapsing_star_damage | 4,583 | 5,085 | +10.9% | 5.6 | 6.2 |
| the_hunt_dot | 3,017 | 3,195 | +5.9% | 26.7 | 28.3 |
| predators_wake_damage | 3,107 | 2,640 | -15.0% | 5.3 | 4.5 |
| voidblade_damage | 1,363 | 2,901 | +112.8% | 3.5 | 7.4 |
| the_hunt_damage | 834 | 1,634 | +95.9% | 1.4 | 2.7 |
| hungering_slash_damage | 1,055 | 1,594 | +51.0% | 2.1 | 3.2 |
| rune_of_unleashed_fire | 800 | 801 | +0.1% | 38.9 | 39.0 |
| voidstep | 517 | 383 | -26.0% | 19.1 | 14.1 |
| rune_of_unleashed_fire_lingering | 268 | 343 | +27.9% | 208.6 | 267.2 |
| vengeful_retreat_damage | 13 | 9 | -31.1% | 20.5 | 14.1 |

## Validation: stat probes

The spec's stats are the buffed snapshot with the gear procs' time average
folded in (crit 26.8% in the log against a 22.3% snapshot). `--set` applies the
probe's stat delta to that base.

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +11.56% | +11.36% | -0.20 pts |
| +500 haste | +12.84% | +11.20% | -1.64 pts |
| +500 mastery | +8.94% | +8.76% | -0.19 pts |
| +500 versatility | +8.74% | +8.79% | +0.06 pts |
| +200 intellect | +5.86% | +6.00% | +0.13 pts |

Mastery matches the game's model exactly: the probe's +9.97 points of mastery
go through 1.2× (Enduring Torment) and the Metamorphosis window's extra
0.4 × mastery, which is where the reference's +8.94% comes from.

## Known gaps

- **Action latency.** SimC adds ~150ms of GCD lag to every action. Without it
  the engine fits more actions into each Metamorphosis window, so Cull and
  Devour run ~10-16% high while Consume (-36%), Reaper's Toll (-36%) and
  Pierce the Veil (-32%) run low: the same total time, shifted between
  windows. Shortening the modelled window to 24s (from the 25.2s average) was
  the compromise that lands the total at +1.5%; a generic per-action lag
  option would replace it.
- **Voidblade vs Pierce the Veil** share one 30s cooldown; which of the two a
  cooldown lands in (56% of the fight is Metamorphosis) splits 7.4/7.0 here
  against SimC's 3.5/10.3.
- **Left in calibration, not modelled:** Feast of Souls (+1%/stack, ~98%
  uptime), Devourer's Bite (+12% from caster, ~100%), Rolling Torment, the
  spec's own +38% cosmic aura, and the Void Elf's Entropic Embrace. They are
  near-constant multipliers, so the residual absorbs them; the mastery and
  crit probes confirm the stat response.
- **Hungering Slash** is cast from the APL's AoE branch in SimC, so its
  single-target count (2.1) is mostly noise; the spec casts it when its 6s
  window is about to expire (3.2 times).
- The gear procs other than Rune of Unleashed Fire (Venomcursed, Soul Fang,
  the rites) are folded into the base stats as time averages, as in the other
  specs.
