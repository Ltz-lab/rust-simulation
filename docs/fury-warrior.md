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
| azsim | 179,423 (±155) |
| difference | **+6.97%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| slayers_strike | 18,187 | 17,864 | -1.8% | 57.4 | 56.7 |
| crushing_blow_mh | 12,641 | 17,599 | +39.2% | 46.4 | 64.8 |
| auto_attack_mh | 13,256 | 14,330 | +8.1% | 242.0 | 262.1 |
| raging_blow_mh | 11,359 | 11,389 | +0.3% | 73.7 | 74.0 |
| bladestorm_mh | 10,342 | 10,483 | +1.4% | 62.7 | 63.9 |
| reap_the_storm_execute | 10,080 | 10,295 | +2.1% | 34.5 | 35.3 |
| execute_mainhand | 9,814 | 8,900 | -9.3% | 48.1 | 43.9 |
| rampage4 | 9,297 | 8,745 | -5.9% | 125.7 | 118.9 |
| crushing_blow_oh | 6,174 | 8,614 | +39.5% | 46.4 | 64.8 |
| rampage3 | 8,056 | 7,612 | -5.5% | 125.7 | 118.9 |
| rampage2 | 7,298 | 6,901 | -5.4% | 125.9 | 119.2 |
| auto_attack_oh | 6,263 | 6,976 | +11.4% | 234.1 | 261.3 |
| rampage1 | 6,006 | 5,670 | -5.6% | 126.1 | 119.3 |
| raging_blow_oh | 5,537 | 5,539 | +0.0% | 73.7 | 74.0 |
| bladestorm_oh | 5,051 | 5,139 | +1.7% | 62.7 | 63.9 |
| execute_offhand | 4,783 | 4,372 | -8.6% | 48.1 | 43.9 |
| deep_wounds | 3,869 | 3,856 | -0.3% | 200.1 | 200.3 |
| bloodbath_dot | 3,202 | 3,599 | +12.4% | 55.7 | 62.7 |
| odyns_fury_mh | 2,465 | 2,523 | +2.4% | 37.2 | 38.2 |
| bloodthirst | 1,425 | 2,305 | +61.7% | 8.6 | 13.9 |
| rend_dot | 714 | 1,896 | +165.4% | 31.7 | 84.3 |
| gushing_wound | 1,022 | 1,626 | +59.0% | 38.1 | 60.8 |
| venomfang | 1,448 | 1,566 | +8.1% | 188.1 | 203.8 |
| bloodbath | 43 | 1,087 | +2439.7% | 0.2 | 5.1 |
| whirlwind (4 sources) | 412 | 1,811 | +339% | 30.6 | 134.9 |
| storm_bolt | 94 | 0 | -100.0% | 7.8 | 0.0 |

The Slayer kit matches closely: Slayer's Strike (-1.8%), Raging Blow (+0.3%),
Bladestorm (+1.4%), Reap the Storm (+2.1%), Odyn's Fury (+2.4%), Deep Wounds
(-0.3%). Rampage (-5.5%) and Execute (-9%) come out a little low, and the
filler — Crushing Blow, Whirlwind, Bloodthirst, Bloodbath — a lot high; see
below.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.17% | +9.56% | +1.39 pts |
| +500 haste | +10.05% | +9.64% | -0.41 pts |
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
- `--gear` reproduces every item level and stat rating (see
  `tools/check_gear.py`) and haste/mastery/versatility exactly; crit reads
  +3 points because SimC's sheet (28.2%) excludes the ~5 points of critical
  chance the abilities actually have, and attack power differs by the
  trinkets' primary-stat procs, which aren't modelled.
