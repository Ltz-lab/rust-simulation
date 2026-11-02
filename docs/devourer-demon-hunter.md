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

Void Metamorphosis drains fury and ends at zero. The engine has no drain, so the
window is a base length (9.5s) plus 1/22 s for every Fury gained while it is up
(`extend_per_gain`; SimC drains 15/s plus 1.4·exp(0.0775·s), a tenth of that while
channelling Void Ray or casting Collapsing Star, 22/s on average over a window).
That puts the average window on SimC's 26.7s, and lets a window shrink when a Reap
takes the place of a Devour (24 Fury) as it does in SimC. The window empties fury
when it ends, with Rolling Torment paying 5 Fury per leftover Collapsing Star stack.
Inside it Consume becomes Devour, Reap becomes Cull, Voidblade becomes Pierce
the Veil and Hungering Slash becomes Reaper's Toll, so the two pairings share
one cast, with the damage row chosen by the buff. Reap and Cull are one 8s,
2-charge, hasted cooldown (`shares_cooldown`), and they gather 4 souls, 10 with
Moment of Craving, or the fragments there are; Soulshaper adds 8% damage per soul
gathered. The Void Ray's last tick gives a charge back, Devour knocks a second off
the recharge and Collapsing Star reduces Voidblade's cooldown by 10s (Voidrush).
Each hero ability triggers one
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
| azsim | 164,254 (±158) |
| difference | **-0.00%** |

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
With gear lines and no kit, azsim gives 164,331 DPS against SimC's 164,322
(+0.01%, `tools/check_gear.py ... --no-trinkets --dps`).

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

Measured on the sample profile (SimC 20,000 iterations at 0.06% error, azsim 20,000
iterations, seed 1, raid buffs on, kit on, trinkets as in the profile):

| | SimC | azsim |
|---|---:|---:|
| set bonuses off | 229,226 | 225,691 |
| 2pc only | 236,880 (+3.34%) | 233,251 (+3.35%) |
| full set | 253,768 (+10.71%) | 251,375 (+11.38%) |
| 4pc over the 2pc | +7.13% | +7.77% |

The tier gain is within 0.7 points of SimC and the full profile within 1%. Soulburst procs
13.0 a fight with the 2pc alone in SimC and 14.0 with the full set; azsim 14.0 and 15.0.

What it took to get there, each checked against SimC with an instrumented build that
switches the 4pc effects off one at a time (full set 253,884; charge back off 251,712, Moment of
Craving off 253,656, the two spawned fragments off 246,816, all three off 245,233; the +10%
Reap/Cull damage is the rest). The charge was the 4pc's biggest lever in azsim (+3.9 points of
the +14.4 it showed before) and is worth +0.9% in SimC:

- **One Reap/Cull pool** (`shares_cooldown`). Reap and Cull had a cooldown each (16s and
  8s) and the engine could not share them, so a charge given back was worth a full extra cast.
- **Moment of Craving gathers what is there.** Reap and Cull cost 4 fragments, 10 with the
  buff, capped by the fragments in hand (the extra 6 used to be free stacks), so the casts
  after one wait for fragments as they do in SimC.
- **Soulshaper**: +8% Reap/Cull damage per soul gathered (the talent the fitted Reap
  multiplier of 1.9 was standing in for). A Reap that eats 10 is worth 1.37 times one that eats
  4, so an extra cast that comes at the cost of the next one's fragments adds little.
- **The Metamorphosis window follows Fury income.** In SimC a Reap or Cull in the place of a
  Devour (24 Fury) shortens the window: with the pool at 6 charges (8.5 more casts) the
  window falls from 26.7s to 25.6s and the DPS gain is +0.4%. A fixed 25s window made every
  extra cast pure gain, which is most of the old overshoot.
- Soulburst's hit shatters a fragment on 9.5% of hits (SimC: 1.2 of 13 a fight).

The Soulburst hit multipliers (1.84 and 1.23) are still the fitted ones: no source for them
turned up. They put the non-crit hit on SimC's (87,754 and 97,767), about two thirds of
what Consume and Devour get per coefficient point.

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| cull_damage | 35,054 | 36,710 | +4.7% | 34.7 | 36.3 |
| reap_damage | 23,116 | 22,398 | -3.1% | 30.9 | 29.9 |
| voidsurge | 15,501 | 16,672 | +7.6% | 24.8 | 26.6 |
| devour | 15,481 | 14,003 | -9.5% | 46.1 | 41.7 |
| void_ray_tick_meta | 12,935 | 12,246 | -5.3% | 293.6 | 278.1 |
| reapers_toll_damage | 12,657 | 12,154 | -4.0% | 17.0 | 16.3 |
| consume | 10,207 | 11,294 | +10.6% | 32.7 | 36.3 |
| void_ray_tick | 9,914 | 6,955 | -29.8% | 313.6 | 220.1 |
| burning_blades | 7,429 | 7,701 | +3.7% | 61.3 | 63.7 |
| pierce_the_veil_damage | 6,378 | 6,022 | -5.6% | 10.3 | 9.8 |
| collapsing_star_damage | 4,583 | 5,357 | +16.9% | 5.6 | 6.5 |
| predators_wake_damage | 3,107 | 3,549 | +14.2% | 5.3 | 6.1 |
| the_hunt_dot | 3,017 | 3,321 | +10.1% | 26.7 | 29.5 |
| voidblade_damage | 1,363 | 1,337 | -1.9% | 3.5 | 3.4 |
| hungering_slash_damage | 1,055 | 2,130 | +101.8% | 2.1 | 4.3 |
| the_hunt_damage | 834 | 871 | +4.4% | 1.4 | 1.4 |
| rune_of_unleashed_fire | 800 | 801 | +0.1% | 38.9 | 38.9 |
| voidstep | 517 | 555 | +7.3% | 19.1 | 20.5 |
| rune_of_unleashed_fire_lingering | 268 | 256 | -4.6% | 208.6 | 199.3 |
| vengeful_retreat_damage | 13 | 14 | +0.1% | 20.5 | 20.5 |

## Validation: stat probes

The spec's stats are the buffed snapshot with the gear procs' time average
folded in (crit 26.8% in the log against a 22.3% snapshot). `--set` applies the
probe's stat delta to that base.

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +11.56% | +11.32% | -0.25 pts |
| +500 haste | +12.84% | +14.26% | +1.42 pts |
| +500 mastery | +8.94% | +8.78% | -0.16 pts |
| +500 versatility | +8.74% | +8.79% | +0.06 pts |
| +200 intellect | +5.86% | +6.00% | +0.13 pts |

Mastery matches the game's model exactly: the probe's +9.97 points of mastery
go through 1.2× (Enduring Torment) and the Metamorphosis window's extra
0.4 × mastery, which is where the reference's +8.94% comes from.

## Known gaps

- **Metamorphosis is a base length plus Fury income, not a drain.** SimC starts the window
  with Fury and drains it at a rate that grows with time and drops to a tenth while
  channelling Void Ray; the spec uses one average rate (22/s), so the window does not
  shorten when the Void Ray channel comes late, and Fury sits at its cap in the window
  (gains count toward the window even when the cap wastes them).
- **Fragments are not objects.** SimC picks a fragment up 0.23s after it spawns, consumes the
  oldest when more than 10 exist (giving its Fury and stack), and picks some up by
  proximity (24 a fight) and with `pick_up_fragment` (19); azsim keeps a count, credits Fury at
  creation and loses the fragments above 10 (36 a fight, stacks included). A Reap needs 4 here
  (SimC takes what is there, 2 of 73 casts had fewer).
- **Soul Splitter** (14.6 extra fragments a fight in SimC) is not modelled.
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
