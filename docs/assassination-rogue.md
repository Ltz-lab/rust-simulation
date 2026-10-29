# Assassination Rogue (Deathstalker)

Draft: the spec, derived modifiers and a first calibration are in place and the
economy and hit counts line up with the reference, but gear, registration and
the gaps listed at the bottom are still open.

`specs/assassination_deathstalker.toml`, built from DB2, SimC's rogue module
(`sc_rogue.cpp`) and its default APL, with the reference being SimC's Midnight
S2 Assassination profile (Deathstalker) cut to the core kit (no trinkets, tier
set, consumables or Bloodlust), 300s Patchwerk, 5,000 iterations.

## Results

| | DPS |
|---|---:|
| reference | 148,195 |
| azsim (`-n 2000 --seed 1`, with calibration) | 149,319 |
| difference | **+0.76%** |

Hit counts per fight (structurally the important part) are within a couple of
percent for the rotation: 54.4 Envenoms (reference 54.3), 14.2 Garrotes (13.8),
11.2 Ruptures (10.4), 120.0 Mutilates (121.9), 5.3 Kingsbanes (5.2),
2.9 Deathmarks (2.9), 1.9 Vanishes (1.9) and 7.1 Thistle Teas (7.0). Amplifying and Deadly
Poison each proc 694 times against 657.

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| envenom | 29,489 | 29,524 | +0.1% | 54.3 | 54.4 |
| rupture | 19,618 | 19,506 | -0.6% | 212.7 | 211.4 |
| garrote | 15,341 | 15,285 | -0.4% | 214.3 | 213.3 |
| auto_attack_mh | 12,175 | 12,348 | +1.4% | 452.1 | 458.3 |
| kingsbane | 9,897 | 9,894 | -0.0% | 41.4 | 41.3 |
| amplifying_poison | 7,538 | 7,958 | +5.6% | 657.6 | 693.9 |
| deadly_poison_instant | 7,543 | 7,956 | +5.5% | 657.4 | 693.5 |
| auto_attack_oh | 5,946 | 5,755 | -3.2% | 450.4 | 435.9 |
| mutilate_mh | 5,811 | 5,725 | -1.5% | 121.9 | 120.0 |
| hunt_them_down | 5,222 | 5,198 | -0.5% | 673.4 | 670.2 |
| deathstalkers_mark | 5,116 | 5,194 | +1.5% | 53.0 | 53.7 |
| implacable_strikes_nature | 3,992 | 4,030 | +1.0% | 26.1 | 26.3 |
| internal_bleeding | 3,075 | 3,211 | +4.4% | 95.5 | 99.6 |
| deadly_poison_dot | 3,173 | 3,149 | -0.8% | 215.2 | 213.2 |
| mutilate_oh | 2,843 | 2,802 | -1.5% | 121.9 | 120.0 |
| mutilated_flesh | 2,528 | 2,674 | +5.8% | 97.7 | 103.3 |
| breath_of_corruption | 2,550 | 2,552 | +0.1% | 20.2 | 20.2 |
| implacable_strikes_physical | 1,959 | 1,978 | +1.0% | 26.1 | 26.3 |
| searing_lightwood | 1,548 | 1,544 | -0.3% | 48.6 | 48.5 |
| deathmark | 1,017 | 1,008 | -0.9% | 34.4 | 34.1 |
| rune_of_unleashed_fire | 934 | 936 | +0.2% | 46.0 | 46.0 |
| rune_of_unleashed_fire_lingering | 292 | 359 | +22.8% | 229.9 | 282.3 |
| shiv | 305 | 324 | +6.4% | 18.8 | 20.0 |
| fan_of_knives | 270 | 287 | +6.5% | 5.7 | 6.1 |
| ambush | 7 | 48 | +606.0% | 0.0 | 0.3 |
| **total** | **148,195** | **149,319** | **+0.76%** | | |

## Held-out stat probes

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +10.21% | +9.63% | -0.59 |
| +500 haste | +7.94% | +8.55% | +0.61 |
| +500 mastery | +9.94% | +9.65% | -0.29 |
| +500 vers | +8.84% | +8.80% | -0.04 |
| +200 agility | +6.34% | +6.49% | +0.15 |

The reference's priority is crit > mastery > vers > haste > agility; azsim
gives mastery > crit > vers > haste > agility, swapping the top two by 0.6
points.

## What is modelled

- **Energy and combo points.** 250 energy (100 + Vigor 2/2 + Path of Blood),
  13.2/s base (10 x Motivated Murderer x Vigor 2/2), hasted. Seven combo
  points (Deeper and Sanguine Stratagem). Unhasted 1s GCDs.
- **Builders and finishers.** Mutilate (two strikes, 2 points, Doomblade banks
  25% of each strike into a rolling bleed), Shiv (10 energy through Toxic
  Stiletto, 20s hasted cooldown), Fan of Knives, Ambush, Envenom (0.395 AP per
  point, Rapid Injection's +40%, Amplifying Poison's +35% for 10 stacks),
  Rupture and Garrote (Razor Wire's +6s), and Seal Fate's point per builder crit.
  The reference records no Relentless Strikes energy, so none is modelled.
- **Poisons.** Dragon-Tempered Blades keeps Deadly and Amplifying Poison both
  active; each rolls on every direct hit at (30% + 30% Envenom) x 0.70 + 5%
  Improved Poisons, doubled while Deathmark ticks (one extra roll per poison).
  Envenom's buff also drives Inspiring Strike.
- **Cooldowns.** Kingsbane (direct + a 14s unhasted dot, five 0.2s Implacable
  strikes that each grant a point, and the stacking +20% Kingsbane buff that
  ends in Regicide's Reward), Deathmark (16s dot, +75% to bleeds and poisons,
  80 energy over its ticks, Finish the Job, then Lingering Darkness), Vanish
  (Ambush and a stealth Garrote), Thistle Tea (3 charges, 100 energy).
- **Deathstalker.** Garrote applies Deathstalker's Mark (3 stacks), finishers
  spend a stack and trigger Unshakeable Drive, the last stack arms Darkest
  Night (+35% and a guaranteed crit on the next 5+ point Envenom, which
  re-applies the Mark). Quietus Celeris spends a stack immediately 30% of the
  time. Hunt Them Down rides every auto attack on a marked target.
- **The APL.** A paraphrase of SimC's default single-target priority list:
  Thistle Tea at low energy, Deathmark before Kingsbane, Vanish when the
  current Garrote is not improved, Garrote (stealth, Improved Garrote window,
  refreshable), Rupture, Envenom (Envenom expiring, Deathmark, energy above
  70%, or 7 points under Darkest Night), Fan of Knives at 6 points under
  Darkest Night, then Ambush/Shiv/Mutilate.

## Known gaps

- **Gear is not modelled.** There is no `[gear]`/`[gear.kit]` block, so
  `--gear` does not work and the stat snapshot is hand-entered.
- **Stat procs are folded into the snapshot.** The fight's average haste
  (`gear_haste`, +8.77%, from the 18.83 energy/s regenerated against 13.2 x
  1.311) and average crit (0.3840, from the 38.4% crit rate of the poisons)
  include the gear's stat procs and Alacrity. Only the three gear *damage*
  procs are modelled separately (Breath of Corruption, Searing Lightwood, Rune
  of Unleashed Fire and its lingering dot).
- **Improved Garrote** only buffs Garrote while its 6s window is up; SimC
  snapshots +50% into the whole 24s dot. That, Lethal Dose, Zoldyck Recipe,
  Precise Killer, Implacable's +20% to the Mark and Hunt Them Down, and the
  remaining gear stat procs sit in the calibration residuals (Garrote's is
  x2.80, Internal Bleeding's x2.42, the auto attacks x6.7).
- Ambush is used in ~30% of fights against 5% in the reference (0.03% DPS),
  and the Rune of Unleashed Fire lingering dot ticks 6.1 times per proc
  against 5.0.

Commands used:

```sh
python3 tools/effects.py target/db2 specs/assassination_deathstalker.toml \
    reference/assa_core_observations.json profiles/MID2_Rogue_Assassination.simc \
    --out derived/assassination_deathstalker.toml
python3 tools/calibrate.py specs/assassination_deathstalker.toml \
    reference/assa_core_observations.json calibration/assassination_deathstalker.toml \
    --derived derived/assassination_deathstalker.toml -n 1000
python3 tools/validate_stats.py ...
```
