# Unholy Death Knight (Rider of the Apocalypse)

Midnight Season 2, core kit: `specs/unholy_rider.toml`, calibrated by
`calibration/unholy_rider.toml`, derived modifiers in `derived/unholy_rider.toml`. The
reference is `reference/unholy_core_observations.json`, built by
`tools/make_reference.sh` from
`profiles/MID2/MID2_Death_Knight_Unholy.simc` (talents, default APL, hero
tree Rider of the Apocalypse).

| | SimC | azsim | diff |
|---|---:|---:|---:|
| DPS (10,000 iterations, seed 1) | 148,582 | 149,950 | +0.92% |

Stat probes (`tools/validate_stats.py`, same calibration):

| probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit rating | +11.60% | +11.18% | -0.42 pts |
| +500 haste rating | +9.06% | +7.77% | -1.29 pts |
| +500 mastery rating | +9.01% | +9.72% | +0.71 pts |
| +500 versatility rating | +8.79% | +8.86% | +0.07 pts |
| +200 strength | +6.11% | +6.13% | +0.03 pts |

## What is modelled

- **Attack power and mastery.** SimC's damage uses the sheet attack power
  plus a flat 1000 base attack power: +200 strength adds 220 sheet AP but
  only 6.1% damage. The player's `attack_power` is 3588 (2588 sheet + 1000),
  which reproduces the strength probe. Dreadblade mastery reaches nearly
  everything, pets and physical autos included; the mastery probe's per
  source gains are a little below the full `(1+mastery)` for the player's
  own abilities, so those carry `mastery_scale` 0.85-0.93 from the probe.
- **Runes and runic power.** Six runes regenerating at `0.225/s` base,
  hasted and doubled by Runic Corruption (60% chance per 30 RP, 3s); Rune
  Mastery's 15%-per-rune 8s +6% regen. Runic power from auto attacks (Runic
  Attenuation), Festering Strike (20), Scourge Strike / Soul Reaper /
  Putrefy / Outbreak (10) and Army of the Dead (50); Death Coil spends 30.
- **Midnight unholy.** Festering Strike costs two runes and leaves the free
  Festering Scythe follow-up (which also applies the 35%-faster-plague
  debuff); both grant Lesser Ghoul stacks. Scourge Strike spends a stack to
  summon a ghoul and erupts both plagues at 35% of a tick with mastery
  double-dipped. Soul Reaper spends up to three stacks. Putrefy summons a
  ghoul that strikes and explodes; Lord of the Dead reanimates it as a Magus,
  and three magi are consumed into a Lord of the Dead. Putrid Echoes repeats
  the ghoul at two charges, and Harbinger of Doom takes 2.5s off Putrefy's
  recharge per Scourge Strike.
- **Diseases and procs.** Dread and Virulent Plague tick every 1.4625s
  (Ebon Fever -25% and Festering Scythe -35% baked in, no haste — SimC's
  diseases are not hasted), permanently maintained by Mograine's Death and
  Decay extensions. Dread Plague ticks roll Sudden Doom (~30% effective:
  free +35% Death Coil) and the Forbidden Knowledge reanimation strike
  (20%). Infected Claws, the Pestilence runeforge, Venomfang, Rune of
  Unleashed Fire and the gear/embellishment procs are clocks or on-hit rolls.
- **Cooldowns.** Army of the Dead (90s: eight ghouls, a Magus, an
  Abomination and all four riders for 20s, plus Forbidden Knowledge) and
  Dark Transformation (45s, off GCD: 200% ghoul damage, Commander of the
  Dead, a Magus, Whitemane for 6s, Soul Reaper reset, 0.5s per RP spent).
  Riders' Champion summons a random rider that is not already out on a
  hasted clock standing in for SimC's 20% chance per rune spent.
- **Pets.** Raise Dead ghoul (sweeping claws only during Dark
  Transformation), lesser ghouls from Scourge Strike / Soul Reaper /
  Doomed Bidding (every Death Coil outside Army), Army ghouls (Death Orders
  on stacks spent during Army), the magi, the Lord of the Dead, Raise
  Abomination and the four horsemen. Pets inherit player crit (their crits
  are ~3.3x, from Grave Mastery, so pet abilities carry
  `crit_damage_bonus = 1.28`), mastery and haste.
- **Rider of the Apocalypse** abilities: Whitemane's Death Coil and Let
  Terror Reign, Mograine's Heart Strike and Death and Decay (one 11-tick
  aura per rider summon), Nazgrim's Scourge Strike (physical + shadow),
  Trollbane's Obliterate and the Icy Fury procs, and the Undeath DoT.

## Engine

No engine change in this calibration round; the draft's one generic fix
(`src/sim.rs`: a pet energy wait shorter than the 1 µs clock tick is floored
at 1 µs) is unchanged and every existing spec still gives identical DPS.

## Known gaps

- **Pet sources are hand-fitted.** SimC merges all guardian rows into the
  player's stats and observe.py keeps only the last row per action name, so
  pet damage cannot be calibrated by `tools/calibrate.py`. Each pet
  ability's coefficient is fitted by hand to SimC's per-hit non-crit average
  from the report (`avg_noncrit`), and counts come from the spec's mechanics.
- **Reference row collisions.** The player's own Death Coil (47541) and auto
  attack rows are overwritten by Whitemane's Death Coil and a horseman's
  auto attack in the reference, so they have no row to calibrate against.
  They are named `death_coil_player` and `auto_attack_player` and fitted the
  same way as the pet sources.
- **Cast counts.** Scourge Strike (102 vs 102), Festering Strike/Scythe
  (10/10 vs 15/14), Putrefy (63 vs 61), Ruptured Viscera (141 vs 144) and
  the ghoul autos match; Death Coil (71 vs 61), Necrotic Coil (32 vs 40,
  Forbidden Knowledge uptime) and the magi are 10-30% off. Death Order is
  +8%.
- **Haste scales ~1.3 points low.** SimC's pet and Lord of the Dead damage
  grow faster with haste than a fixed uptime allows (magi are consumed into
  Lords more often, and the Lord's consecutive summons overlap less), which
  the permanent-Lord stand-in cannot reproduce.
- No trinkets and (the reference is the core kit; the tier set is under Tier set below) and
  no multi-target APL.

## Tier set (Midnight season 2, set 2055)

`[[gear.effects]]` in the spec, applied with gear lines; `--set gear.set_bonuses=0` turns it off. Tier gain is the
sample profile with the set against the same profile without it (SimC: `set_bonus=midnight_season_2_{2,4}pc=0`;
SimC about 400 iterations to 0.3% error, azsim 20,000 iterations, seed 1):

| bonus | what it does | modelled as | SimC | azsim |
|---|---|---|---:|---:|
| 2pc (1296654) | Magus and Lord of the Dead cast Necrotic Bolt instead of Frostbolt, the Magus Withering Grasp instead of Shadow Bolt | the pets' rotations replaced (new `set = { rotation = .. }` on a pet patch); coefficients from the new spells' DB2 values, scaled like the spells they replace; Withering Grasp as one hit a cast (2.089 ticks) | +5.09% | +4.98% |
| 4pc (1296655) | Necrotic Bolt +130% and Withering Grasp +130% against enemies at or below 35% health | a health buff (`below_health = 35`) with `pet_pct` 130 on the Magus and the Lord | +5.73% (on the 2pc) | +5.62% |
| both | | | **+11.11%** | **+10.88%** |

Full profile: SimC 240,022, azsim 240,936 (+0.38%); set off: SimC 216,029, azsim 217,293 (+0.59%). The bolt
coefficients: Magus Necrotic Bolt 1.71594 x 1.8672 (the spec's Frostbolt 1.831113 over spell 317792's 0.980671,
which already holds Lord of the Dead's +25%) = 3.2040; Lord Necrotic Bolt 1.51406 x 2.1260 (the spec's 2.299522
over spell 1292107's 0.865297 x 1.25; the new spell has no +25% label) = 3.2189. SimC's non-crit hits against its
Frostbolt are 1.758x (Magus) and 1.397x (Lord); the coefficients give 1.750x and 1.400x. Per source, SimC / azsim
pDPS with the 2pc: Lord's bolt 30,152 / 22,380 (Frostbolt 21,562 / 15,994), Magus' bolt 4,804 / 5,394, Withering Grasp
5,405 / 3,003. The Lord's absolute damage is the existing gap in the spec's pets (-26% with the set off as well); the
gain it adds is +39.9% in both.

Withering Grasp is a rolling DoT in SimC (0.384306 a 1s tick, refreshed by each cast, outliving its Magus); azsim has
one DoT per pet ability shared by every instance, so the 2.089 ticks a cast SimC lands (220 on 105.3 casts) are dealt as
the cast's hit. The Magus' Frostbolt cooldown of 15 (one cast per Magus, as the spec has it) is kept for Necrotic Bolt:
SimC casts 44.6 of either with and without the set.

