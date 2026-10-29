# Frost Mage (Spellslinger)

`specs/frost_mage_spellslinger.toml`, built the same way as the other specs:
coefficients, cast times, charges and durations from DB2 (`tools/spelldb.py`),
the rotation from SimC's default Spellslinger APL, and the rest observed in
SimC's log. Frost is a shatter spec: Frostbolt, Flurry, Glacial Spike, Ray of
Frost and splinters stack **Freezing** on the target, and Ice Lance spends it
(6 stacks, 12 while Thermal Void is up) for a hit several times its own size.
The Spellslinger splinter chain (Force of Will, Augury Abounds, Splinterstorm)
is most of the hit count and about a sixth of the damage.

## Results

Reference: SimC's Midnight S2 sample Frost profile (Spellslinger), core kit only
(no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations.

| | DPS |
|---|---:|
| reference | 149,542 |
| azsim | 150,315 (±78) |
| difference | **+0.52%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| shatter_ice_lance | 46,448 | 48,202 | +3.8% | 124.4 | 129.3 |
| frost_splinter | 21,383 | 21,499 | +0.5% | 1300.3 | 1308.2 |
| ray_of_frost | 20,149 | 19,263 | -4.4% | 63.5 | 60.8 |
| frozen_orb_bolt | 13,294 | 13,384 | +0.7% | 338.2 | 340.6 |
| flurry_bolt | 12,801 | 12,032 | -6.0% | 161.1 | 151.5 |
| glacial_spike | 11,974 | 11,970 | -0.0% | 14.7 | 14.8 |
| ice_lance | 8,258 | 8,575 | +3.8% | 124.4 | 129.3 |
| hand_of_frost | 6,876 | 6,824 | -0.8% | 62.2 | 61.7 |
| frostbolt | 5,406 | 5,621 | +4.0% | 35.6 | 37.0 |
| rune_of_unleashed_fire | 1,033 | 1,034 | +0.0% | 50.3 | 50.4 |
| void_eruption | 771 | 769 | -0.3% | 28.9 | 28.9 |
| acidbrood_scourge | 767 | 765 | -0.2% | 21.6 | 21.6 |
| rune_of_unleashed_fire_lingering | 313 | 368 | +17.6% | 243.9 | 287.2 |
| **total** | **149,542** | **150,315** | **+0.52%** | | |

Cast counts behind those hits: Flurry 38.0 vs 40.3, Frozen Orb 11.7 vs 11.3,
Ray of Frost 7.6 vs 8.0, Glacial Spike 14.8 vs 14.7, Augury Abounds 31.7 procs.
Crit rates match source by source (Shatter 50.4% vs 50.5%, Ice Lance 33.5% vs
33.6%, Flurry 33.6% vs 33.6%), which is the point of modelling the crit chain
rather than averaging it.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +10.35% | +10.11% | -0.24 pts |
| +500 haste | +7.25% | +8.04% | +0.79 pts |
| +500 mastery | +9.03% | +9.73% | +0.71 pts |
| +500 versatility | +8.68% | +8.79% | +0.11 pts |
| +200 intellect | +6.31% | +6.66% | +0.35 pts |

The stat priority order (crit > mastery > vers > haste > int) matches the
reference.

## What's modelled

- **Freezing / Shatter.** Freezing is a 20-stack, 15 s debuff. Frostbolt (1),
  each Flurry bolt (1), Glacial Spike (3), each Ray of Frost tick (1), splinters
  (15%) and Hand of Frost (1) add stacks; Frostbolt and Glacial Spike crits add
  one more (Frostbite). Ice Lance shatters `min(6 x (1 + Thermal Void), stacks)`
  stacks as one hit and consumes them; with Fingers of Frost it shatters 6 (12)
  and consumes nothing. Shatter's crit chance is 1.5x normal (Improved
  Shatter), which needed one generic engine field, `crit_chance_multiplier`.
- **Fingers of Frost** (2 stacks, 15 s): Frostbolt 15%, Glacial Spike 100%
  (Flash Freeze) + 15%, Frozen Orb 100% on cast (Everlasting Frost) + 3% per
  bolt.
- **Brain Freeze** (15 s): Frostbolt 35%, Glacial Spike 25%, Frozen Orb 100%
  (Wintertide). A proc resets one Flurry charge (2 charges, 30 s recharge).
  Flurry with Brain Freeze is +50% damage and grants Thermal Void (8 s), which
  Ice Lance consumes for the double shatter.
- **Icicles / Glacial Spike**: one Icicle every 5 s hasted, 5 stacks unlock
  Glacial Spike; the cast spends them for a 13.0 SP hit and 3 Freezing.
- **Ray of Frost**: 8 ticks of 4.5 SP over 4 s (hasted), 1 Freezing and 1
  splinter per tick, Hand of Frost on every second tick, 4 splinters on cast
  (Signature Spell), Splinterstorm for 10 s, 60 s cooldown less 0.2 s per
  shatter stack (Glaciate).
- **Frozen Orb**: 30 bolts of 0.777 SP every 0.5 s, 3% Fingers of Frost each,
  four Splintering Orbs splinters, 60 s cooldown less 0.3 s per splinter
  (Spellfrost Teachings).
- **Splinters**: 0.453 SP each, 15% to add a Freezing stack (Infused
  Splinters), one per Frostbolt/Flurry/Ray tick, 3 per Glacial Spike, 4 per
  Ray of Frost, 2 per Brain Freeze (Shifting Shards), 4 per Frozen Orb, and
  `shatter stacks / 2` per Ice Lance (Force of Will). Splinterstorm doubles
  every conjure. Each splinter is an Augury Abounds attempt: 10%
  bad-luck-protected roll with a 0.5 s internal cooldown, 8 more splinters.
- **Buffs**: Hand of Frost (5 x 1% damage, 8 s, from every Hand of Frost hit),
  Permafrost Lances (+10% Shatter, 15 s, from Frozen Orb), Overflowing Energy
  (+10% Frostbolt/Glacial Spike crit per stack, spent on a crit), Brainstorm
  (+1% intellect per Brain Freeze, overlapping).
- **Gear procs**: Rune of Unleashed Fire, Acidbrood Scourge, Void Eruption,
  and the time-averaged secondary-stat procs folded into `[player]`.

## Known gaps

- **Flurry 38.0 vs 40.3 casts (-6% DPS on flurry_bolt).** The reference's
  charge economy wastes fewer Brain Freeze resets than a `reset_cooldown` that
  keeps the recharge running; the residual is within the ±10% flag threshold
  but is the largest count gap.
- **Ray of Frost 7.6 vs 8.0 casts.** Glaciate's cooldown reduction is fitted
  at 0.2 s per shatter stack (SimC's spell data reads 0.1 s); at 0.1 s the spec
  casts Ray ~6 times per fight instead of 8.
- **Hand of Frost's shatter proc chance** is fitted at 22% (+10% with Thermal
  Void) to match the observed 62 hits per fight; SimC's talent values (10% +
  1% per shatter stack) would give about 21 from shatters instead of 30.
- **Icicles tick every 5 s hasted.** That is what the reference log shows
  (3.9 s at 26% haste); the DB2 talent row lists a 6 s period.
- **rune_of_unleashed_fire_lingering** ticks 287 vs 244 per fight (+17.6%),
  the same gap the Fire spec has: stacked applications extend the DoT further
  than SimC's.
- The damage spells' derived multipliers are missing for Frostbolt, Ice Lance
  and Glacial Spike because the observations record the cast spell id (116,
  30455, 199786) rather than the damage spell (228597, 228598, 228600), so the
  talent modifiers land in the calibration residual instead
  (Frostbolt x1.88, Glacial Spike x1.89).
