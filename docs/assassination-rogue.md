# Assassination Rogue (Deathstalker)

`specs/assassination_deathstalker.toml`, built from DB2, SimC's rogue module
(`sc_rogue.cpp`) and its default APL, with the reference being SimC's Midnight
S2 Assassination profile (Deathstalker) cut to the core kit (no trinkets, tier
set, consumables or Bloodlust), 300s Patchwerk, 5,000 iterations.

Calibration round 1 is done: hit and cast counts are within a few percent of
the reference and the stat probes track within a point. Gear and
the gaps at the bottom are still open.

## Results

| | DPS |
|---|---:|
| reference | 148,195 |
| azsim (`-n 10000 --seed 1`, with calibration) | 148,533 |
| difference | **+0.23%** |

Hit counts per fight (structurally the important part) are close for the
rotation: 54.6 Envenoms (reference 54.3), 14.5 Garrotes (13.8), 11.2 Ruptures
(10.4), 120.5 Mutilates (121.9), 5.3 Kingsbanes (5.2), 2.9 Deathmarks (2.9),
1.9 Vanishes (1.9), 18.9 Shivs (18.8) and 7.0 Thistle Teas (6.1). Amplifying
and Deadly Poison each proc 674 times against 657.

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| envenom | 29,489 | 29,579 | +0.3% | 54.3 | 54.6 |
| rupture | 19,618 | 19,519 | -0.5% | 212.7 | 211.7 |
| garrote | 15,341 | 15,286 | -0.4% | 214.3 | 213.6 |
| auto_attack_mh | 12,175 | 12,020 | -1.3% | 452.1 | 446.9 |
| kingsbane | 9,897 | 9,887 | -0.1% | 41.4 | 41.4 |
| deadly_poison_instant | 7,543 | 7,720 | +2.3% | 657.4 | 673.7 |
| amplifying_poison | 7,538 | 7,719 | +2.4% | 657.6 | 674.0 |
| auto_attack_oh | 5,946 | 5,891 | -0.9% | 450.4 | 446.5 |
| mutilate_mh | 5,811 | 5,742 | -1.2% | 121.9 | 120.5 |
| hunt_them_down | 5,222 | 5,186 | -0.7% | 673.4 | 669.2 |
| deathstalkers_mark | 5,116 | 5,199 | +1.6% | 53.0 | 53.9 |
| implacable_strikes_nature | 3,992 | 4,004 | +0.3% | 26.1 | 26.4 |
| internal_bleeding | 3,075 | 3,210 | +4.4% | 95.5 | 99.8 |
| deadly_poison_dot | 3,173 | 3,141 | -1.0% | 215.2 | 213.2 |
| mutilate_oh | 2,843 | 2,804 | -1.4% | 121.9 | 120.5 |
| mutilated_flesh | 2,528 | 2,675 | +5.8% | 97.7 | 103.5 |
| breath_of_corruption | 2,550 | 2,555 | +0.2% | 20.2 | 20.3 |
| implacable_strikes_physical | 1,959 | 1,971 | +0.6% | 26.1 | 26.4 |
| searing_lightwood | 1,548 | 1,550 | +0.1% | 48.6 | 48.7 |
| deathmark | 1,017 | 1,006 | -1.1% | 34.4 | 34.1 |
| rune_of_unleashed_fire | 934 | 935 | +0.1% | 46.0 | 46.1 |
| shiv | 305 | 305 | +0.2% | 18.8 | 18.9 |
| rune_of_unleashed_fire_lingering | 292 | 272 | -6.8% | 229.9 | 214.6 |
| fan_of_knives | 270 | 288 | +6.9% | 5.7 | 6.1 |
| ambush | 7 | 0 | -100.0% | 0.0 | 0.0 |
| **total** | **148,195** | **148,533** | **+0.23%** | | |

## Held-out stat probes

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +10.21% | +9.74% | -0.48 |
| +500 haste | +7.94% | +8.61% | +0.66 |
| +500 mastery | +9.94% | +9.66% | -0.28 |
| +500 vers | +8.84% | +8.80% | -0.04 |
| +200 agility | +6.34% | +6.50% | +0.15 |

The reference's priority is crit > mastery > vers > haste > agility, and azsim
gives the same order.

## What is modelled

- **Energy and combo points.** 250 energy (100 + Vigor 2/2 + Path of Blood),
  13.2/s base (10 x Motivated Murderer x Vigor 2/2), hasted. Seven combo
  points (Deeper and Sanguine Stratagem). Unhasted 1s GCDs.
- **Builders and finishers.** Mutilate (two strikes, 2 points, Doomblade banks
  25% of each strike into a rolling bleed), Shiv (10 energy through Toxic
  Stiletto, its 30s cooldown cut to 15s and not hasted), Fan of Knives, Ambush,
  Envenom (0.395 AP per point, Rapid Injection's +40%, Amplifying Poison's
  +35% for 10 stacks), Rupture and Garrote (Razor Wire's +6s), and Seal Fate's
  point per builder crit. The reference records no Relentless Strikes energy,
  so none is modelled.
- **Poisons.** Dragon-Tempered Blades keeps Deadly and Amplifying Poison both
  active; each rolls on every direct hit at (30% + 30% Envenom) x 0.70 + 5%
  Improved Poisons. The Envenom half is a second, conditional entry (0.21 on
  top of the 0.26), so the buff decides whether it rolls. A lethal poison that
  procs while Deathmark ticks fires a second time. Envenom's buff also drives
  Inspiring Strike.
- **Auto attacks.** A swing lands at 0.84 and Skyfury repeats 16% of the hits
  that land, always as a hit (0.84 x 0.16 of a swing). Kingsbane's Implacable
  strikes roll poisons; Kingsbane itself does not.
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
  include the gear's stat procs and Alacrity. The reference's white-swing
  cadence is about 4% faster than the snapshot reproduces (479 swings a fight
  against 459), so the auto-attack hit chance carries the difference (0.84 per
  swing against the reference's 0.94 hits per swing). Only the three gear
  *damage* procs are modelled separately (Breath of Corruption, Searing
  Lightwood, Rune of Unleashed Fire and its lingering dot).
- **Improved Garrote** only buffs Garrote while its 6s window is up; SimC
  snapshots +50% into the whole 24s dot. That, Lethal Dose, Zoldyck Recipe,
  Precise Killer, Implacable's +20% to the Mark and Hunt Them Down, and the
  remaining gear stat procs sit in the calibration residuals (Garrote's is
  x2.80, Internal Bleeding's x2.42, the auto attacks x6.7).
- **Rupture is refreshed a little early.** 11.2 casts against 10.4 (the dot's
  tick count matches, 211.7 against 212.7), which carries Rupture's Internal
  Bleeding to 99.9 ticks against 95.5.
- Ambush is no longer used (Vanish waits for Garrote to be ready), matching
  the reference's 0.05 casts a fight. The Rune of Unleashed Fire lingering dot
  ticks 4.7 times per proc against 5.0, and the poison procs come out 2.3%
  high.

Commands used:

```sh
python3 tools/effects.py target/db2 specs/assassination_deathstalker.toml \
    reference/assa_core_observations.json profiles/MID2_Rogue_Assassination.simc \
    --out derived/assassination_deathstalker.toml
python3 tools/calibrate.py specs/assassination_deathstalker.toml \
    reference/assa_core_observations.json calibration/assassination_deathstalker.toml \
    --derived derived/assassination_deathstalker.toml -n 2000
python3 tools/validate_stats.py specs/assassination_deathstalker.toml \
    calibration/assassination_deathstalker.toml \
    reference/assa_core_observations.json \
    "+500 crit=reference/assa_probe_crit_rating.json" \
    "+500 haste=reference/assa_probe_haste_rating.json" \
    "+500 mastery=reference/assa_probe_mastery_rating.json" \
    "+500 vers=reference/assa_probe_versatility_rating.json" \
    "+200 agility=reference/assa_probe_agility.json"
```
