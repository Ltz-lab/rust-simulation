# Elemental Shaman (Farseer)

MID2 Farseer single-target core kit, using `profiles/MID2_Shaman_Elemental.simc` and the committed `reference/ele_*.json` SimC observations. Run with `specs/elemental_farseer.toml` and `calibration/elemental_farseer.toml`. The reference excludes trinkets, consumables, set bonuses and Bloodlust; `[gear.kit]` adds consumables and Bloodlust when gear is supplied.

At 10,000 iterations, seed 1, azsim gives **163,606 DPS** against SimC **161,888** (+1.06%). Calibration fits damage per hit, not counts. For example, per fight ancestor Lava Burst hits are 293.9 vs 282.6, Lightning Bolt 145.7 vs 140.9, Lava Burst 62.2 vs 65.4, and Elemental Blast 22.0 vs 22.0. The model includes Maelstrom, Flame Shock and Lava Surge, Stormkeeper, Ascendance, mastery overloads, elemental summons and individually reacting Farseer ancestors. Gear stats against the core profile without trinkets match SimC's ratings and buffed stats exactly; gear effects use the shared effect library.

Stat probes (20,000 iterations, paired seed 42; +500 rating, +200 intellect):

| probe | SimC gain | azsim gain | difference |
|---|---:|---:|---:|
| crit | +8.59% | +8.36% | -0.23 pts |
| haste | +9.64% | +10.44% | +0.80 pts |
| mastery | +7.79% | +7.11% | -0.68 pts |
| versatility | +8.67% | +8.79% | +0.12 pts |
| intellect | +6.24% | +6.43% | +0.19 pts |

## Known gaps

Lava Surge uses an observed constant per-tick chance rather than SimC's increasing chance after successive failures; the timing of its reset during a hard cast is approximated. Storm elemental Wind Gust hits are 36.7 vs 34.1 and Lava Burst overloads 77.1 vs 82.9 per fight. Proc averages in the player stats are fixed to the reference profile; different gear can change them. Shaman tier effects and any trinkets absent from the shared library are not modelled; use gear effect coverage reporting to identify unsupported effects. The full raid kit has not been calibrated to a full-kit SimC run.
