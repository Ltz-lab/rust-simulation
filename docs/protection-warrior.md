# Protection Warrior (Colossus)

`specs/protection_colossus.toml`, calibrated by `calibration/protection_colossus.toml`,
derived modifiers in `derived/protection_colossus.toml`. A tank, simulated only
for the damage it deals: the boss's attacks matter solely as sources of rage
(3 every ~5s), Tough as Nails procs (blocks) and free Revenges (dodge/parry).
First spec of a tank role in azsim; before it wow-sims sent Protection and Blood
characters to SimC. Drafted with `tools/draft_spec.py`, then completed by hand
against SimC's `sc_warrior.cpp`, its default APL (`actions.colossus_st`) and
its log.

**Status: calibrated (round 1), -2.0% on the core kit.**

## Results

Reference: SimC's Midnight S2 sample Protection profile (Colossus), core kit (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 3,000 iterations.
azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 118,635 |
| azsim | 116,248 (±70) |
| difference | **-2.01%** |

Before this spec: none (`supports_gear()` false, wow-sims fell back to SimC).

### Hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| shield_slam | 39,211 | 39,600 | +1.0% | 97.1 | 98.4 |
| execute_damage | 12,350 | 11,535 | -6.6% | 31.0 | 29.0 |
| revenge | 11,482 | 10,721 | -6.6% | 74.5 | 69.5 |
| devastator | 10,392 | 10,351 | -0.4% | 239.3 | 238.6 |
| thunder_clap | 6,281 | 5,966 | -5.0% | 63.7 | 60.6 |
| demolish (3 hits) | 11,122 | 10,646 | -4.3% | 10.0 | 9.6 |
| auto_attack_mh | 5,356 | 5,339 | -0.3% | 239.3 | 238.6 |
| deep_wounds | 4,209 | 4,686 | +11.3% | 158.6 | 176.6 |
| phalanx | 4,274 | 4,127 | -3.4% | 59.3 | 57.3 |
| rend_dot | 4,254 | 4,001 | -5.9% | 149.6 | 140.9 |
| ravager_whirling_blade | 3,593 | 3,325 | -7.5% | 83.3 | 77.2 |

### Stat probes (held out)

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +9.34% | +8.94% | -0.40 pts |
| +500 haste | +6.98% | +7.80% | +0.82 pts |
| +500 mastery | +8.34% | +8.79% | +0.45 pts |
| +500 versatility | +8.82% | +8.82% | -0.00 pts |
| +200 strength | +5.65% | +7.44% | +1.79 pts |

Same stat priority as the reference (crit > versatility > mastery > haste > strength).

### Gear (`--gear`)

`tools/check_gear.py` on the sample profile: every item level and rating matches
SimC; crit, haste and versatility match (crit is the 31% abilities hit at, see
below). DPS from gear is 116,157 against the hand-entered stats' 116,076
(+0.07%).

| character (5,000 iterations) | SimC | azsim | diff |
|---|---:|---:|---:|
| gear, no trinkets, core kit | 118,667 | 116,157 | -2.12% |
| + consumables and Bloodlust (`[gear.kit]`), set bonuses off | 129,950 | 127,927 | -1.6% |
| full profile (trinkets, class set, no potion modelled) | 169,411 | 168,537 | -0.5% |

The full-profile figure is a near cancellation, not an accuracy claim: azsim's
Guillotine runs ~30% hot per hit (104,900 against 76,400) while the potion (about
2-3% in SimC) is missing.

## What is modelled

- **Rage** (max 130): swing +3 and the Devastator it fires +2 (Instigate), Shield
  Slam +19 (+50% in Violent Outburst), Thunder Clap +5, Shield Charge +20,
  Demoralizing Shout +20 (Booming Voice), Ravager +20 a tick, damage taken +3,
  Frothing Berserker +5 on 16% of Revenges. Spent on Ignore Pain 35, Shield
  Block 30, Revenge 10, Execute 20. Ignore Pain has no effect but its cost; the
  APL's rage thresholds are kept.
- **Devastator** fires on every swing (including Skyfury repeats) and, with
  Thunder Clap, Revenge and Execute, resets Shield Slam 27.5% of the time (fitted:
  it reproduces SimC's 97 Slams; the data say 20-25%).
- **Violent Outburst**: 250 rage spent triggers it (`spent.rage/250` as a chance);
  the next Shield Slam does x3 and gains 50% more rage.
- **Colossus**: Shield Slam stacks Colossal Might (two on a crit, Precise Might);
  Demolish, a 30s three-hit channel, needs 5 and spends them. Thunder Clap
  applies Rend and a Phalanx the next Shield Slam spends for an extra hit.
  Execute and Demolish's third hit apply Deep Wounds (Decimator).
- **Mastery** is Critical Block, which adds attack power: x(1 + mastery%) on every
  AP-scaled source [db2 76857 #5]. The spec applies it per source
  (`mastery = true`, `[player].mastery` = 0.2359) and keeps AP as the buffed
  strength x1.0815; `[gear] mastery_ap` tells `tools/validate_stats.py` how to
  split SimC's AP (which already carries it) into the two.
- **Gear**: `[gear]` and `[gear.kit]` as for the other melee specs; the class set
  (2pc Vengeful Shield, 4pc Bloody Rebuke) is in `[[gear.effects]]`. Bloody Rebuke
  comes to 4,793 DPS against SimC's 4,758.

## Known gaps

- **Revenge, Execute and Thunder Clap run 6-7% short** (and Demolish 4%, Rend 6%):
  the engine wins slightly fewer GCDs than SimC. Shield Slam, Devastator and the
  autos are within 1%.
- **Deep Wounds** has 11% more DPS than SimC: its rolling refresh counts more
  ticks per application than SimC's.
- **Violent Outburst** is a rage-spent chance, not a counter, and Shield Block's
  Heavy Repercussions extension is folded into a fixed 6s buff.
- **Strength** responds +1.8 points too strongly (+7.44% against +5.65%). About a
  quarter of SimC's damage does not follow the AP the probe adds; the cause is not
  found (the procs and Tough as Nails are the suspects).
- **Crit** is the observed 31% for all abilities (the sheet says 25.5%; Wild
  Strikes, Rune of Critical Power and Celeritous Conclusion lift it), not modelled
  as buffs. Into the Fray and Celeritous Conclusion haste are a flat +6%.
- **Items**: azsim sums 1,845 strength from the items against the profile's 2,042
  (about 197 is missing from the item tables), absorbed into `base_primary`.
  The potion (Liquid Luster) and the 2pc's +15% Revenge trigger rate are not
  modelled; Zul'jin's Guillotine is ~30% hot per hit on this character.
- Single target only; no Wrecking Throw, Champion's Spear or Leap (under one a
  fight in the reference). Survivability is not simulated.
