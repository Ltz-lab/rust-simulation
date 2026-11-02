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

Lava Surge uses an observed constant per-tick chance rather than SimC's increasing chance after successive failures; the timing of its reset during a hard cast is approximated. Storm elemental Wind Gust hits are 36.7 vs 34.1 and Lava Burst overloads 77.1 vs 82.9 per fight. Proc averages in the player stats are fixed to the reference profile; different gear can change them. Trinkets absent from the shared library are not modelled; use gear effect coverage reporting to identify unsupported effects. The season 2 tier set is modelled (below). The full raid kit has not been calibrated to a full-kit SimC run.

## Tier set (Midnight season 2, set 2065)

`[[gear.effects]]` entries for 2pc 1296625 and 4pc 1296626 in `specs/elemental_farseer.toml`, applied with gear lines when the pieces are worn (`--set gear.set_bonuses=0` turns them off). Numbers from the set spells and Flowing Elements / Overcharge! (1300219, 1300222) in the game data; mechanics from `sc_shaman.cpp` (`mid2_ele_4pc_builder` / `_spender`) and the default Elemental APL.

- **2pc**: +25% direct damage to Elemental Blast (the cast, its overloads, and the Power of the Maelstrom one). The same bonus covers Earth Shock and Earthquake (and Earthquake's ticks); this spec casts neither on one target.
- **4pc**: Flowing Elements (2 stacks) arrives when Stormkeeper ends or Ascendance ends (4 if Stormkeeper ended inside Ascendance). It gives Lava Burst, Lightning Bolt and their overloads +25%; each Lava Burst or Lightning Bolt cast spends a stack and makes the next Elemental Blast free (Overcharge!).
- **Rotation**: the APL changes with the 4pc (`set_bonus.mid2_4pc`), so the fragment adds those lines: Stormkeeper whenever it is ready and not up, Ascendance on cooldown, Stormkeeper charges spent before Ascendance, the free Elemental Blast before the next builder, and Lava Burst while Flowing Elements is up. Without the set the rotation is unchanged.

Profile `MID2_Shaman_Elemental`, gear lines, seed 1, Patchwerk 300s, one target. SimC 0.05% error, azsim 12,000 iterations (0.08% at 95%). "Off" is SimC with `set_bonus=midnight_season_2_2pc=0/midnight_season_2_4pc=0/bite_of_zuljan_2pc=0` and azsim with `gear.set_bonuses=0`.

| | SimC | azsim |
|---|---:|---:|
| Full profile DPS | 250,358 | 252,527 (+0.9%) |
| Set bonuses off | 218,477 | 221,261 (+1.3%) |
| Tier gain | +14.59% | +14.13% |
| 2pc alone | +5.2% | +5.12% |
| 4pc on top of the 2pc | +9.2% | +8.57% |
| Elemental Blast casts a fight (full / off) | 41.5 / 23.7 | 41.7 / 23.9 |
| Lightning Bolt casts a fight (full / off) | 130.3 / 150.9 | 132.5 / 153.6 |

The 2pc-alone rows for SimC come from a 0.14% error run (about +-0.2 points). Not modelled: Earthquake and Earth Shock (2pc effect 2 and the spender side of the 4pc for them).
