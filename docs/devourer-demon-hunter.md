# Devourer Demon Hunter (Void-Scarred)


`specs/devourer_voidscarred.toml`, built like the other specs: coefficients,
costs, cooldowns and durations from DB2 (SimC's `spell_query` on build
12.1.5.70575), mechanics read from SimC's demon hunter module
(`sc_demon_hunter.cpp`) and its default Void-Scarred APL, and pacing observed
in SimC's combat log. Everything below is generic engine data (resources,
banked DoTs, channels, cooldown resets, one-shot buffs), not class code.

The spec runs on two resources and one counter:

- **Fury** (140) is spent by Void Ray (100) and built by Consume (+8), Devour
  (+16), Soul Immolation (+6 a tick), Voidblade/Pierce the Veil (+20), a
  Hungering Slash/Reaper's Toll (+10) and by collecting soul fragments (+4
  each, credited where the fragment is created [db2 1223628]).
- **Soul fragments** (cap 10) are shattered by ~25% of direct hits (Shattered
  Souls + Sweet Release), by Consume/Devour (+2, Predator's Thirst), Soul
  Immolation (+3), Hungering Slash/Reaper's Toll (+2) and Collapsing Star
  (+3). Reap and Cull eat 4, plus 6 while Moment of Craving is up (the Void
  Ray's last tick grants it).
- **Void Metamorphosis stacks**: every fragment consumed outside Metamorphosis
  adds one; 35 stacks open Void Metamorphosis. Inside it fragments build
  Collapsing Star stacks instead, and 30 of those open Collapsing Star.

Void Metamorphosis drains fury and ends at zero; the spec models the window as
a fixed 25s buff (the log averages 25.2s of uptime) and empties fury when it
ends, with Rolling Torment paying 5 Fury per leftover Collapsing Star stack.
Inside it Consume becomes Devour, Reap becomes Cull, Voidblade becomes Pierce
the Veil and Hungering Slash becomes Reaper's Toll, so the two pairings share
one cast, with the damage row chosen by the buff. Reap and Cull share one 8s,
2-charge, hasted cooldown in SimC; azsim has per-ability cooldowns, so Cull
keeps the full 8s (it is the only spender while Metamorphosis is up) and Reap
uses 16s, which keeps the pool's throughput. The Void Ray's last tick gives a
charge back, Devour knocks a second off the recharge and Collapsing Star
reduces Voidblade's cooldown by 10s (Voidrush). Each hero ability triggers one
Voidsurge per Metamorphosis (the first also repeats through Volatile
Instinct); Voidsurge is a +10%-per-stack, 12s buff. Hungering Slash windows
come from Voidblade/Pierce The Veil and The Hunt/Predator's Wake, and
Hungering Slash/Reaper's Toll grant the Vengeful Retreat used off the GCD. The
Hunt is only cast at max void stacks, right after a Vengeful Retreat (its
leap window), so its damage lands inside the Metamorphosis that opens next.

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
| azsim | 164,308 (±101) |
| difference | **+0.03%** |

## Gear

`--gear` on the sample profile reproduces SimC's buffed stats exactly
(`tools/check_gear.py ... --no-trinkets`): every item level and rating
matches, and spell power 3,969, crit 22.30%, haste 20.86%, mastery 44.37%,
versatility 5.28%. The `[gear]` constants come from the stat probes: level-90
base intellect 622 (SimC's naked character), ×1.05 matching gear ×1.03 Arcane
Intellect ×1.10 talent (3969 buffed; +200 int gives +238 spell power), spell
power = intellect, crit 7% plus rating/46, haste from rating alone, 16.2
mastery points × Monster Within's 1.0, and Mark of the Wild's 3%
versatility. The gear stat procs (Venomcursed Critical Strike and its kin) are
folded into `[player]` as a +4.5% crit average, so `[gear.procs]` adds that
back.

`[gear.kit]` adds SimC's default Devourer consumables and Bloodlust on top:
Blooming Feast, the Void-Touched rune, Flask of the Magisters, Thalassian
Phoenix Oil, a primary-stat potion used inside Metamorphosis or the last 30s,
and Bloodlust at the pull. `--set gear.kit.enabled=0` gives the core-kit run.
With gear lines and no kit, azsim gives 164,350 DPS against SimC's 164,317
(+0.02%, 5,000 iterations, seed 42), which is +0.01% against the spec-stats
run.

## Tier set (set 2056, 2pc and 4pc)

With gear lines the spec reads the Void-Scarred class set from `[[gear.effects]]`
(`--set gear.set_bonuses=0` turns it off; `--gear-stats` lists both bonuses as
`modelled`). Numbers are DB2 spell data (1296615, 1296616, 1297433, 1297432) and
SimC's `sc_demon_hunter.cpp`.

| bonus | effect | modelled as |
|---|---|---|
| 2pc | a Reap or Cull that eats 4+ fragments rolls for Soulburst: 6.63% on the first roll, +5.19% per roll since, cap 39.49% (SimC's `soulburst_blp_*` options, not the spell's flat 20%) | a roll counter buff and a `chance_expr` trigger on Reap and Cull |
| 2pc | Soulburst (10s) makes the next Consume or Devour instant (still on the GCD) and adds a 6 SP hit | `consume_soulburst` / `devour_soulburst` abilities, rotation entries after Vengeful Retreat (out) and Collapsing Star (in), as the APL's `consume,if=buff.soulburst.up` |
| 4pc | Reap and Cull +10% | `ability_pct` on both |
| 4pc | the Soulburst cast grants Moment of Craving, gives back a Reap/Cull charge and spawns 2 fragments | `on_cast` effects on the Soulburst hit |

Measured on the sample profile (SimC 20,000 iterations at 0.14% error, azsim 20,000
iterations, seed 1, raid buffs on, kit on, trinkets as in the profile):

| | SimC | azsim |
|---|---:|---:|
| set bonuses off | 230,132 | 232,398 |
| 2pc only | 236,655 (+2.83%) | 240,205 (+3.36%) |
| full set | 254,165 (+10.44%) | 271,130 (+16.67%) |
| 4pc over the 2pc | +7.40% | +12.87% |

The 2pc matches (Soulburst procs 14.0 a fight with the 2pc alone against SimC's 13.9 for the full set; 15.7
in azsim's full set; the hit 87,850 / 99,474 non-crit against 87,754 / 97,767). The 4pc overshoots by about 5 points: the spec's Reap and Cull are throttled by
their cooldowns (the shared 8s pool is split, see Known gaps), so the charge the 4pc gives back is worth a
cast each time (about +5% on its own), while in SimC the fragment supply is the limit and the same procs add
only 5.4 casts (the 2 fragments a proc spawns add 0.3% here). Moment of Craving's +6 stacks are
unconstrained by the fragments left, as for the Void Ray's, and are worth +2% of that. The set's
Soulburst hit does not shatter fragments (SimC: 9.5% of its hits); not modelled. The Reap/Cull split and
the fragment economy are the base spec's and are left alone, so the 4pc's gain is not tuned to SimC's.

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| cull_damage | 35,054 | 35,247 | +0.5% | 34.7 | 34.9 |
| reap_damage | 23,116 | 22,114 | -4.3% | 30.9 | 29.6 |
| voidsurge | 15,501 | 17,117 | +10.4% | 24.8 | 27.4 |
| devour | 15,481 | 12,915 | -16.6% | 46.1 | 38.5 |
| reapers_toll_damage | 12,657 | 14,885 | +17.6% | 17.0 | 20.0 |
| consume | 10,207 | 13,248 | +29.8% | 32.7 | 42.5 |
| void_ray_tick_meta | 12,935 | 12,058 | -6.8% | 293.6 | 273.9 |
| void_ray_tick | 9,914 | 5,720 | -42.3% | 313.6 | 181.1 |
| pierce_the_veil_damage | 6,378 | 7,679 | +20.4% | 10.3 | 12.5 |
| burning_blades | 7,429 | 7,467 | +0.5% | 61.3 | 61.7 |
| collapsing_star_damage | 4,583 | 5,425 | +18.4% | 5.6 | 6.6 |
| predators_wake_damage | 3,107 | 3,878 | +24.8% | 5.3 | 6.6 |
| the_hunt_dot | 3,017 | 3,379 | +12.0% | 26.7 | 30.0 |
| voidblade_damage | 1,363 | 394 | -71.1% | 3.5 | 1.0 |
| hungering_slash_damage | 1,055 | 498 | -52.8% | 2.1 | 1.0 |
| the_hunt_damage | 834 | 612 | -26.6% | 1.4 | 1.0 |
| rune_of_unleashed_fire | 800 | 801 | +0.1% | 38.9 | 39.0 |
| voidstep | 517 | 566 | +9.4% | 19.1 | 20.9 |
| rune_of_unleashed_fire_lingering | 268 | 256 | -4.5% | 208.6 | 199.3 |
| vengeful_retreat_damage | 13 | 14 | +2.1% | 20.5 | 20.9 |

## Validation: stat probes

The spec's stats are the buffed snapshot with the gear procs' time average
folded in (crit 26.8% in the log against a 22.3% snapshot). `--set` applies the
probe's stat delta to that base.

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +11.56% | +11.30% | -0.26 pts |
| +500 haste | +12.84% | +11.71% | -1.13 pts |
| +500 mastery | +8.94% | +8.81% | -0.13 pts |
| +500 versatility | +8.74% | +8.79% | +0.06 pts |
| +200 intellect | +5.86% | +6.00% | +0.13 pts |

Mastery matches the game's model exactly: the probe's +9.97 points of mastery
go through 1.2× (Enduring Torment) and the Metamorphosis window's extra
0.4 × mastery, which is where the reference's +8.94% comes from.

## Known gaps

- **Metamorphosis is a fixed 25s buff, not a fury drain.** SimC starts the
  window with 200 Fury and drains it to zero; the spec empties fury at expiry
  and pays Rolling Torment's 5 Fury per leftover stack, but the window length
  does not react to fury generation.
- **The shared Reap/Cull cooldown is split.** Reap and Cull are one 8s,
  2-charge, hasted cooldown in SimC; the engine's cooldowns are per ability, so
  Cull keeps 8s (it is the only spender while Metamorphosis is up) and Reap
  takes 16s. The total lands right (65 casts against 65.6) but the split
  between Cull (-0%) and Devour (-17%) inside the window, and between Reap and
  Consume outside it, is off by ~15-30%.
- **Outside Void Ray runs low** (181 ticks against 314, -42%): the spec's
  outside Fury economy is smaller than SimC's, mostly because fragments are
  four Fury each credited at creation while SimC's pickup heals also pace the
  income, and because soul immolation is re-applied more often in SimC. The
  freed GCDs go to Consume (+30%). A generic "requires X, costs Y per tick"
  channel-cost option would model Void Ray's 5 Fury a tick directly instead of
  a 100 Fury cost.
- **The Hunt is rare** (1.0 cast against 1.4): it is gated on the Vengeful
  Retreat leap window the way the APL's `voidscarred_hunt_entry` is, and the
  spec's Vengeful Retreat charges are consumed as soon as they are granted.
  Hungering Slash (1.0 against 2.1) and Voidblade (1.0 against 3.5) outside
  Metamorphosis are gated the same way.
- **Left in calibration, not modelled:** Feast of Souls (+1%/stack, ~98%
  uptime), Devourer's Bite (+12% from caster, ~100%), the spec's own +38%
  cosmic aura, and the Void Elf's Entropic Embrace. They are near-constant
  multipliers, so the residual absorbs them; the mastery and crit probes
  confirm the stat response.
- The gear procs other than Rune of Unleashed Fire (Venomcursed, Soul Fang,
  the rites) are folded into the base stats as time averages, as in the other
  specs. The lingering rune DoT is a separate rune's proc in the data, so it
  is applied by 67% of Unleashed Fire procs to match its observed 26
  applications per fight.
- The sample profile's trinkets (Freightrunner's Flask, Gebbo's Bottomless
  Bag) have no template in `data/item_effects.toml`, so a full raid-kit run
  counts them as stats only.
