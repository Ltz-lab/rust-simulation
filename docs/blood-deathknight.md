# Blood Death Knight (Deathbringer)

`specs/blood_deathbringer.toml`, built the way the other specs are: coefficients,
costs and durations from DB2 (`tools/spelldb.py`), mechanics from SimC's death
knight module (`sc_death_knight.cpp`) and the default APL in the profile, the rest
observed in SimC's combat log. Blood is a tank, so only the damage it deals to a
Patchwerk boss is simulated: the boss still matters in two small ways (Bone
Shield loses stacks to its hits and Bloody Reflection fires on them), each its own
source below. Survivability, healing and absorbs are not
simulated.

Before this spec wow-sims had no Blood entry, so Blood characters always fell back
to SimC.

## Results

Reference: SimC's Midnight S2 sample Blood Death Knight (Deathbringer,
`profiles/MID2_Death_Knight_Blood_Deathbringer.simc`), core kit only (no trinkets,
tier set, consumables or Bloodlust), 300s Patchwerk, 5,000 iterations. The
embellishments are removed from the profile (Prismatic Focusing Iris alone is 2.9%
of SimC's damage): azsim does not read ring and wrist effects yet.

| | DPS |
|---|---:|
| SimC | 81,067 |
| azsim (10,000 iterations, ±48) | 80,665 |
| difference | **-0.50%** |

From gear lines (`--gear`, `tools/check_gear.py`, trinkets off): SimC 81,046, azsim
80,699 (-0.43%), within 0.01% of the hand-entered stats. Every item's level and
every rating matches SimC exactly (crit 960, haste 816, mastery 745, versatility
514), and so do crit, haste and versatility percent. With the raid-night kit
(consumables, Bloodlust and the potion; trinkets and set bonuses off): SimC
91,917, azsim 92,026 (+0.12%). The full profile (trinkets and class set on) is
SimC 120,730, azsim 118,660 (-1.7%, see Known gaps).

## Structure: hit counts per fight

| source | SimC DPS | azsim DPS | diff | SimC hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| death_strike | 18,951 | 19,018 | +0.4% | 69.6 | 69.9 |
| exterminate | 8,032 | 7,903 | -1.6% | 17.4 | 17.1 |
| auto_attack_mh | 7,997 | 7,904 | -1.2% | 172.1 | 170.3 |
| reapers_mark_explosion | 6,541 | 6,394 | -2.2% | 6.9 | 6.7 |
| heart_strike | 5,665 | 5,626 | -0.7% | 81.7 | 81.2 |
| blood_boil | 5,024 | 5,029 | +0.1% | 56.9 | 57.0 |
| exterminate_second_hit | 3,530 | 3,474 | -1.6% | 17.3 | 17.1 |
| blood_plague | 3,388 | 3,394 | +0.2% | 234.5 | 235.1 |
| wave_of_souls | 3,030 | 2,953 | -2.5% | 14.0 | 13.7 |
| death_and_decay_damage | 1,282 | 1,283 | +0.1% | 267.2 | 267.6 |
| reapers_mark | 1,232 | 1,202 | -2.5% | 7.0 | 6.8 |
| consumption | 1,140 | 1,125 | -1.3% | 6.3 | 6.2 |
| rune_of_unleashed_fire | 1,112 | 1,113 | +0.1% | 50.0 | 50.1 |
| marrowrend | 945 | 1,031 | +9.2% | 18.6 | 20.4 |
| blood_boil_boiling_point | 933 | 945 | +1.3% | 11.2 | 11.4 |
| bloody_reflection | 912 | 912 | 0.0% | 25.2 | 25.3 |
| venomfang | 783 | 783 | 0.0% | 165.3 | 165.4 |
| blood_mist | 708 | 707 | -0.1% | 33.3 | 33.3 |
| sanguinary_burst | 637 | 633 | -0.6% | 3.7 | 3.7 |
| the three Dancing Rune Weapon copies, all spells | 7,412 | 7,438 | +0.4% | | |

The ghoul, Bloodworms, Claw and the Rune of Unleashed Fire / Venomfang lingering
parts are within 1% too (all under 0.5% of the total each).

## Stat probes (held out)

| stat probe | SimC DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +9.52% | +8.65% | -0.87 pts |
| +500 haste | +7.16% | +6.33% | -0.83 pts |
| +500 mastery | +9.56% | +8.39% | -1.16 pts |
| +500 versatility | +8.80% | +8.23% | -0.57 pts |
| +200 strength | +6.06% | +6.08% | +0.03 pts |

Stat priority: SimC mastery > crit > versatility > haste > strength; azsim crit >
mastery > versatility > haste > strength (mastery and crit are 0.04 apart in SimC).

## What is modelled

- **Runes and runic power.** Six runes (max three recharging at once in SimC; here
  a flat hasted regeneration), runic power max 125. Heart Strike 1 rune and +17 RP
  (15 plus Heartbreaker's 2); Marrowrend 1 rune, +20 RP, +3 Bone Shield; Death's
  Caress 1 rune, +10 RP, +2 Bone Shield; Death and Decay 1 rune (free with Crimson
  Scourge), +10 RP; Reaper's Mark 2 rune, +20 RP, +3 Bone Shield; Runic
  Attenuation +3 (real PPM); Blood Mist +2 a hit. Death Strike spends 35 and is
  the biggest source in the rotation, so nearly every RP point is turned into one.
- **Rotation.** SimC's `deathbringer` list, with the cooldown lines (Vampiric Blood,
  Dancing Rune Weapon, the potion) in front.
- **Deathbringer.** Reaper's Mark (45s) explodes for 40 stacks of 0.544 AP a stack,
  on average 6.7s later (SimC ends the debuff at the 40-stack cap, 3-11s); the
  explosion arms two Exterminates and every Dancing Rune Weapon cast arms one.
  Marrowrend spends one and fires it 500ms later (12.88 AP, then 5.72 AP). Wave of
  Souls hits twice per Mark, the second always a crit.
- **Dancing Rune Weapon.** Three pets (Dancing Rune Weapon, Everlasting Bond, and
  Dance of Midnight copies at 12.5% a rune spent) cast copies of Death Strike,
  Heart Strike, Marrowrend, Blood Boil, Blood Plague and swings. They are echo
  buffs that copy a share of the player's own hit as final damage: Death Strike
  14.5%, Heart Strike 17.3%, Marrowrend 52%, Blood Boil 28.9%, Blood Plague 42%,
  swings 3.7%, each the pets' total divided by the player's source in SimC's
  report. Dance of Midnight also makes the next Heart Strike +150% and a rune
  cheaper (31% of Heart Strikes inside a window).
- **Bone Shield** gains stacks from the above, loses one to a boss hit outside a
  Dancing Rune Weapon window (a Poisson rate fitted to SimC's 29 losses a fight);
  Death's Caress and Marrowrend refresh it when it falls under 6 stacks or 3s.
- **Mastery** is Blood Shield, which also lifts attack power (the sheet's attack
  power carries x(1 + 0.5 x mastery), and the damage of every source responds 9.4%
  to +21.7 points of the sheet's mastery). The spec applies it per source
  (`mastery = true`, `[player].mastery` = 0.2858) over strength-derived attack
  power plus a flat 1,000 from the two-hander's DPS, as Unholy's does.
- **Gear.** `[gear]` and `[gear.kit]`, as the other melee specs: strength x1.05 x
  Battle Shout/Skyfury; `weapon_ap_per_dps` for the flat attack power; `[gear.procs]`
  adds the time-averaged haste the sheet misses (the GCD averages 1.124s in the log,
  33.4% against the sheet's 23.3%: Berserking, Parry Haste). `[gear] mastery_ap`
  tells `tools/validate_stats.py` how to split SimC's attack power into the two.
- **Smaller sources**: Blood Mist, Sanguinary Burst (after each Dancing Rune
  Weapon), Bloody Reflection (a rate; the boss is not simulated), Boiling Point,
  Consumption, the ghoul and the Bloodworms as flat procs, and the Omnium rune and
  Venomfang as in Frost.

## Known gaps

- **Marrowrend +9% and Death's Caress -32%** (about 0.1% of DPS each way): Bone
  Shield refresh picks Marrowrend over Death's Caress more often than SimC, whose
  `runes<4` test sees a fuller rune pool than the engine's. SimC regenerates 115
  runes a fight and the engine, spending each rune as it arrives, needs 0.40/s
  (×33.4% haste) to keep Reaper's Mark on its 45s cadence.
- **Haste probe -0.8 points, crit -0.9, mastery -1.2**: SimC gets a little more from
  each rating than the sheet-level changes explain (the stat feeds Blood Boil
  recharge, runes and the like in ways the engine only approximates). Within about
  one point, as the other specs.
- **Dancing Rune Weapon copies are average shares**, not windows: they follow the
  player's hits at a constant ratio instead of only inside the 8-12s windows.
  Together they match SimC to 0.4%, but a build that casts differently inside the
  windows would be off.
- **Reaper's Mark** explodes on a fixed 6.7s delay; SimC ends it at the stack cap
  (3-11s), driven by the player's shadowfrost hits (Death Strike counts only while
  Dark Talons is up). Its Exterminate arming is the same on average.
- **Blood Draw, Blood Shield, Frost Shield, healing and absorbs** are not
  simulated. Hemostasis (after Blood Boil) and Coagulopathy (after Death Strike)
  raise Death Strike's hit; they are folded into its derived and fitted
  multipliers (x3.49 derived, the rest fitted), not tracked as buffs.
- **Full profile**: SimC 120,730, azsim 118,660 (-1.7%). Blood's Midnight S2
  2pc/4pc (Blood Debt, Relentless Riders' Strength, Bloody Demise) is unmodelled,
  and the two trinkets run through the shared item library (not touched here).
  The potion is Draught of Rampant Abandon, a stacking stat buff that SimC does not
  describe in data; 870 strength for 30s is fitted to its +4.1% DPS.
- **Embellishments** (Prismatic Focusing Iris, Stabilizing Gemstone Bandolier) are
  not modelled; the reference profile has them removed.
- Single target only.
