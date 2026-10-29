# Fury Warrior (Slayer)

`specs/fury_slayer.toml`, built like the others: coefficients, costs,
cooldowns, durations and talent values from DB2, SimC's warrior module
(`sc_warrior.cpp`) and its default APL where a mechanic is scripted, and the
rest observed in SimC's log. Fury is the second rage spec, on the Slayer hero
tree of the MID2 sample profile. Every mechanic below is generic engine data,
not warrior code.

**Status: draft.** The rotation and the main damage sources are in place and
calibrated; the total is ~7% high because the engine casts more filler than
SimC does (see [Known gaps](#known-gaps)).

## Results

Reference: SimC's Midnight S2 sample Fury profile (Slayer), core kit only (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 167,734 |
| azsim | 179,730 (±149) |
| difference | **+7.15%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| slayers_strike | 18,187 | 17,969 | -1.2% | 57.4 | 57.1 |
| crushing_blow_mh | 12,641 | 17,557 | +38.9% | 46.4 | 64.6 |
| auto_attack_mh | 13,256 | 14,376 | +8.5% | 242.0 | 262.9 |
| raging_blow_mh | 11,359 | 11,063 | -2.6% | 73.7 | 72.0 |
| bladestorm_mh | 10,342 | 10,487 | +1.4% | 62.7 | 63.8 |
| reap_the_storm_execute | 10,080 | 10,324 | +2.4% | 34.5 | 35.4 |
| execute_mainhand | 9,814 | 8,787 | -10.5% | 48.1 | 43.4 |
| rampage4 | 9,297 | 9,463 | +1.8% | 125.7 | 128.7 |
| crushing_blow_oh | 6,174 | 8,590 | +39.1% | 46.4 | 64.6 |
| rampage3 | 8,056 | 8,233 | +2.2% | 125.7 | 128.7 |
| rampage2 | 7,298 | 7,474 | +2.4% | 125.9 | 129.0 |
| auto_attack_oh | 6,263 | 6,998 | +11.7% | 234.1 | 262.1 |
| bloodbath_bladestorm_unhinged | 6,402 | 6,014 | -6.1% | 28.8 | 27.0 |
| rampage1 | 6,006 | 6,147 | +2.3% | 126.1 | 129.1 |
| raging_blow_oh | 5,537 | 5,383 | -2.8% | 73.7 | 72.0 |
| bladestorm_oh | 5,051 | 5,137 | +1.7% | 62.7 | 63.8 |
| execute_offhand | 4,783 | 4,323 | -9.6% | 48.1 | 43.4 |
| deep_wounds | 3,869 | 3,843 | -0.7% | 200.1 | 199.8 |
| bloodbath_dot | 3,202 | 3,300 | +3.1% | 55.7 | 57.6 |
| odyns_fury_mh | 2,465 | 2,487 | +0.9% | 37.2 | 37.7 |
| bloodthirst | 1,425 | 2,170 | +52.3% | 8.6 | 13.1 |
| rend_dot | 714 | 1,713 | +139.8% | 31.7 | 75.9 |
| venomfang | 1,448 | 1,571 | +8.4% | 188.1 | 204.3 |
| gushing_wound | 1,022 | 1,505 | +47.2% | 38.1 | 56.3 |
| rune_of_unleashed_fire | 1,066 | 1,063 | -0.3% | 50.6 | 50.5 |
| whirlwind_mh_others | 183 | 693 | +278.9% | 10.2 | 38.7 |
| venomfang_burst | 628 | 669 | +6.5% | 46.3 | 49.3 |
| bloodbath | 43 | 652 | +1422.0% | 0.2 | 3.0 |
| odyns_fury_oh | 382 | 390 | +2.1% | 12.5 | 12.6 |
| whirlwind_mh_first | 94 | 355 | +278.3% | 5.1 | 19.4 |
| whirlwind_oh_others | 89 | 338 | +279.4% | 10.2 | 38.7 |
| rune_of_unleashed_fire_lingering | 319 | 318 | -0.1% | 241.9 | 241.8 |
| bloodthirst_bladestorm_unhinged | 123 | 201 | +63.3% | 0.7 | 1.2 |
| whirlwind_oh_first | 46 | 174 | +278.0% | 5.1 | 19.4 |
| storm_bolt | 94 | 0 | -100.0% | 7.8 | 0.0 |
| charge_impact | 3 | 0 | -100.0% | 1.0 | 0.0 |
| **total** | **167,734** | **179,730** | **+7.15%** | | |

The Slayer kit matches closely: Slayer's Strike (-1.2%), Raging Blow (-2.6%),
Bladestorm (+1.4%), Reap the Storm (+2.4%), Odyn's Fury (+0.9%), Deep Wounds
(-0.7%), Rampage (+2%). Execute comes out 10% low and the filler — Crushing
Blow (+39%), Whirlwind (x3), Bloodthirst (+52%), Bloodbath — a lot high; see
below.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.17% | +9.45% | +1.27 pts |
| +500 haste | +10.05% | +9.62% | -0.43 pts |
| +500 mastery | +8.91% | +10.00% | +1.09 pts |
| +500 vers | +8.53% | +8.59% | +0.06 pts |
| +200 strength | +5.44% | +6.60% | +1.16 pts |

The reference's priority is haste > mastery > vers > crit > strength; azsim's
is mastery > haste > crit > vers > strength. Crit and strength over-respond
because mastery is modelled as always on (see below) and because the filler
casts scale with crit.

## What is modelled

- **Two two-handers.** Every attack is a constant weapon damage
  (`damage.amount`: 606.2 main hand, 268.5 off hand after the off-hand
  penalty) plus a DB2 attack-power coefficient; off-hand hits deal half, so
  their coefficient and amount are halved. Auto attacks use the 3.6s/6 power
  modifier and carry the +144% auto-attack damage of the Fury Warrior spec
  aura [db2 137050 #4] as `mult`, which `tools/effects.py` doesn't read.
- **Rage.** Auto attacks (7.6 / 3.8, x1.5 in Recklessness), Bloodthirst and
  Bloodbath (+8), Raging Blow (+12, 18 in Recklessness), Execute (+20, it
  costs nothing with Improved Execute [db2 316402]), Odyn's Fury (+20);
  Rampage spends 80 less the 20 the Berserk buff refunds, Whirlwind 20. Every
  20 rage spent takes a second off Recklessness (Anger Management [db2
  152278]), which is what brings the 90s cooldown back every ~40s.
- **Enrage** (4s, +15% haste) from Bloodthirst/Bloodbath (45% with Fresh
  Meat), every Rampage and Odyn's Fury; 99% uptime. Unshackled Fury's mastery
  applies to every source.
- **Recklessness** (18s: 12s x Rampaging Berserker's +50% duration) grants
  +20% critical chance to the spells it lists, swaps Raging Blow for Crushing
  Blow and Bloodthirst for Bloodbath, and gives +50% rage from auto attacks
  and from Raging/Crushing Blow.
- **Slayer:** special attacks have a 15% chance (as a 12.5% flat roll,
  matching SimC's pseudo-random distribution's 57.4 procs a fight) to fire
  Slayer's Strike (0.5s ICD), which stacks Executioner (+3% Execute damage,
  crit and crit damage, 5 stacks). Every 3rd one grants Sudden Death. Sudden
  Death Executes fire Reap the Storm and stack Imminent Demise, whose extra
  Bladestorm strikes are squeezed into the same 4s (Unrelenting Onslaught
  also takes 5s off Bladestorm per Executioner stack and stacks Overwhelmed).
  Bladestorm itself is a 4s DoT with a strike every 1s, half of them
  auto-casting Bloodthirst (Unhinged), and it ends with Brutal Finish (+50%
  Rampage) and Violent Euphoria (+15% haste).
- **Bleeds:** Deep Wounds (rolling, 6s/1s), Gushing Wound (Cold Steel, Hot
  Blood on Bloodthirst crits), Bloodbath's bleed, Rend (Storm of Blood on
  Whirlwind), Odyn's Fury's 4s bleed. Bloodborne's +20% to bleeds.
- **Gear procs:** Venomfang, Rune of Unleashed Fire, Skyfury's extra swings,
  and the time-averaged Surge of Adrenaline (+30% auto speed and damage for
  6s after a Raging Blow reset, folded in at its observed 57.5% uptime).

## Known gaps

- **Filler casts.** Crushing Blow, Whirlwind, Bloodthirst and Bloodbath all
  run well above the reference: the engine's rotation never idles, so every
  GCD that SimC spends waiting (or on a priority the APL expresses as a
  condition the engine can't see) goes to the last usable action. Fixing it
  needs either a "wait for rage" notion in the rotation or the shared
  Raging Blow / Bloodthirst cooldown categories; that is the 7%.
- **Storm Bolt** (0.06% of damage, cast inside Bladestorm) isn't modelled.
- **Mastery is always on.** Unshackled Fury only applies while Enrage is up
  (99.1%); it's modelled as the abilities' `mastery` flag, which over-weights
  mastery slightly (the +500 mastery probe reads +1.09).
- **Charge** deals 3 DPS in the reference and isn't modelled as damage.
- The Raging Blow / Crushing Blow charge pool is one ability here (as in
  SimC), with the resets restarting the recharge like `cooldown_t::reset`;
  Bloodthirst and Bloodbath do *not* share their cooldown (SimC's do), which
  is part of why Bloodbath runs high.
- `--gear` reproduces every item level and stat rating, attack power (3,226
  vs SimC's 3,225) and haste/mastery/versatility exactly, and lands within
  0.5% of the hand-entered stats (`tools/check_gear.py --dps`); crit reads
  +5.4 points because SimC's sheet (24.96%) excludes the ~5 points of
  critical chance the abilities actually have (Cruel Strikes and an
  unmodelled ~3%).
