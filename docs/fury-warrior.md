# Fury Warrior (Slayer)

`specs/fury_slayer.toml`, built like the others: coefficients, costs,
cooldowns, durations and talent values from DB2, SimC's warrior module
(`sc_warrior.cpp`) and its default APL where a mechanic is scripted, and the
rest observed in SimC's log. Fury is the second rage spec, on the Slayer hero
tree of the MID2 sample profile. Every mechanic below is generic engine data,
not warrior code.

**Status: calibrated (round 1).** The rotation is a line-for-line paraphrase
of the profile's `actions.slayer`, the rage economy is measured from SimC's
own log, and every damage source is calibrated per hit. The total is 3.8% low
because the engine wins slightly fewer GCDs for Rampage and Execute than SimC
does and fills the difference with Whirlwind (see
[Known gaps](#known-gaps)).

## Results

Reference: SimC's Midnight S2 sample Fury profile (Slayer), core kit only (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 167,734 |
| azsim | 161,427 (±118) |
| difference | **-3.76%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| slayers_strike | 18,187 | 17,835 | -1.9% | 57.4 | 56.3 |
| auto_attack_mh | 13,256 | 13,079 | -1.3% | 242.0 | 239.2 |
| crushing_blow_mh | 12,641 | 12,007 | -5.0% | 46.4 | 44.1 |
| raging_blow_mh | 11,359 | 12,359 | +8.8% | 73.7 | 80.3 |
| bladestorm_mh | 10,342 | 9,389 | -9.2% | 62.7 | 56.9 |
| reap_the_storm_execute | 10,080 | 9,757 | -3.2% | 34.5 | 33.4 |
| execute_mainhand | 9,814 | 8,311 | -15.3% | 48.1 | 40.8 |
| rampage4 | 9,297 | 8,674 | -6.7% | 125.7 | 117.8 |
| rampage3 | 8,056 | 7,535 | -6.5% | 125.7 | 117.8 |
| rampage2 | 7,298 | 6,839 | -6.3% | 125.9 | 118.0 |
| bloodbath_bladestorm_unhinged | 6,402 | 5,731 | -10.5% | 28.8 | 25.8 |
| auto_attack_oh | 6,263 | 6,376 | +1.8% | 234.1 | 238.5 |
| crushing_blow_oh | 6,174 | 5,881 | -4.7% | 46.4 | 44.1 |
| raging_blow_oh | 5,537 | 6,017 | +8.7% | 73.7 | 80.3 |
| rampage1 | 6,006 | 5,621 | -6.4% | 126.1 | 118.1 |
| bladestorm_oh | 5,051 | 4,583 | -9.3% | 62.7 | 56.9 |
| execute_offhand | 4,783 | 4,049 | -15.3% | 48.1 | 40.8 |
| deep_wounds | 3,869 | 3,727 | -3.7% | 200.1 | 193.6 |
| bloodbath_dot | 3,202 | 2,856 | -10.8% | 55.7 | 49.8 |
| odyns_fury_mh | 2,465 | 2,479 | +0.6% | 37.2 | 37.4 |
| bloodthirst | 1,425 | 1,519 | +6.6% | 8.6 | 9.2 |
| venomfang | 1,448 | 1,476 | +1.9% | 188.1 | 192.0 |
| rune_of_unleashed_fire | 1,066 | 1,063 | -0.3% | 50.6 | 50.5 |
| gushing_wound | 1,022 | 966 | -5.5% | 38.1 | 36.1 |
| rend_dot | 714 | 916 | +28.2% | 31.7 | 40.6 |
| venomfang_burst | 628 | 613 | -2.4% | 46.3 | 45.2 |
| odyns_fury_oh | 382 | 386 | +1.1% | 12.5 | 12.5 |
| whirlwind_mh_others | 183 | 347 | +89.5% | 10.2 | 19.3 |
| rune_of_unleashed_fire_lingering | 319 | 318 | -0.1% | 241.9 | 241.8 |
| bloodthirst_bladestorm_unhinged | 123 | 184 | +49.3% | 0.7 | 1.1 |
| whirlwind_mh_first | 94 | 176 | +87.7% | 5.1 | 9.7 |
| whirlwind_oh_others | 89 | 168 | +89.0% | 10.2 | 19.3 |
| storm_bolt | 94 | 85 | -9.5% | 7.8 | 7.1 |
| whirlwind_oh_first | 46 | 86 | +87.6% | 5.1 | 9.7 |
| bloodbath | 43 | 55 | +28.8% | 0.2 | 0.2 |
| charge_impact | 3 | 3 | +1.4% | 1.0 | 1.0 |
| **total** | **167,734** | **161,427** | **-3.76%** | | |

What matches is the shape of the spec: auto attacks, Rampage's four hits,
Crushing Blow, Reap the Storm, Odyn's Fury, Deep Wounds, Gushing Wound and the
gear procs all land within a few percent of the reference's hit counts, and
Slayer's Strike is within 2%. What is left is a GCD-allocation gap: the engine
wins 7 fewer Rampages and 7 fewer Executes a fight and spends those GCDs on
Whirlwind (+4, and with it Rend) and Raging Blow (+6, since a Raging Blow that
does not happen during Recklessness shows up as Raging Blow rather than
Crushing Blow). Because damage per hit is fitted per source, those counts show
up directly as the 3.8% total; see [Known gaps](#known-gaps).

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.17% | +9.36% | +1.19 pts |
| +500 haste | +10.05% | +10.77% | +0.72 pts |
| +500 mastery | +8.91% | +9.99% | +1.08 pts |
| +500 vers | +8.53% | +8.59% | +0.06 pts |
| +200 strength | +5.44% | +6.56% | +1.11 pts |

The reference's priority is haste > mastery > vers > crit > strength; azsim's
is haste > mastery > crit > vers > strength. Crit and strength over-respond
because mastery is modelled as always on (see below) and because the filler
casts scale with crit; everything is within ~1.2 points, the range the other
melee specs sit in.

## What is modelled

- **Two two-handers.** Every attack is a constant weapon damage
  (`damage.amount`: 606.2 main hand, 268.5 off hand after the off-hand
  penalty) plus a DB2 attack-power coefficient; off-hand hits deal half, so
  their coefficient and amount are halved. Auto attacks use the 3.6s/6 power
  modifier and carry the +144% auto-attack damage of the Fury Warrior spec
  aura [db2 137050 #4] as `mult`, which `tools/effects.py` doesn't read.
- **Rage.** Auto attacks (7.6 / 3.8), Execute (+25), Bloodthirst and
  Bloodbath (+8), Raging Blow (+12) and its Crushing Blow (+18), Bladestorm
  strikes (+5) and Cold Steel, Hot Blood (+4) all scale with Recklessness's
  +50% rage from damage, so each reads x1.5 while Recklessness is up
  [db2 1719 #0]; Odyn's Fury (+20), Whirlwind (+3) and Recklessness itself
  (+50, Reckless Abandon) are flat grants. Execute costs nothing (Improved
  Execute [db2 316402]) and Rampage spends 80, or 50 while Reckless
  (Rampaging Berserker [db2 1269309] over the base 80). Every 20 rage spent
  takes a second off Recklessness (Anger Management [db2 152278]), which is
  what brings the 90s cooldown back every ~40s; Frothing Berserker refunds
  10% of a Rampage 20% of the time.
- **Enrage** (4s, +15% haste) from Bloodthirst/Bloodbath (45% with Fresh
  Meat), every Rampage and Odyn's Fury; 99% uptime. Unshackled Fury's mastery
  applies to every source.
- **Recklessness** (18s: 12s x Rampaging Berserker's +50% duration) grants
  +20% critical chance to the spells it lists, swaps Raging Blow for Crushing
  Blow and Bloodthirst for Bloodbath, and gives +50% rage from auto attacks
  and from Raging/Crushing Blow.
- **Slayer:** special attacks have a 15% chance to fire Slayer's Strike
  (0.5s ICD). SimC rolls it as a pseudo-random distribution
  (`p * attempts_since_proc`, `sc_warrior.cpp`); a flat 13.65% reproduces its
  57.4 procs a fight. Slayer's Strike stacks Executioner (+3% Execute damage,
  crit and crit damage, 5 stacks, each on its own 12s timer). Every 3rd one
  grants Sudden Death. Sudden Death Executes fire Reap the Storm and stack
  Imminent Demise, whose extra Bladestorm strikes are squeezed into the same
  4s (Unrelenting Onslaught also takes 5s off Bladestorm per Executioner
  stack and stacks Overwhelmed). Bladestorm itself is a 4s DoT with a strike
  every 1s, half of them auto-casting Bloodthirst (Unhinged), and it ends
  with Brutal Finish (+50% Rampage) and Violent Euphoria (+15% haste). Like
  SimC's, it blocks every other attack while it runs (`warrior_attack_t::
  ready`), which is why Storm Bolt — the one ability flagged
  `usable_while_channeling` through Unrelenting Onslaught — is cast only
  inside it.
- **Bleeds:** Deep Wounds (rolling, 6s/1s), Gushing Wound (Cold Steel, Hot
  Blood on Bloodthirst crits; re-applied fresh, not extended), Bloodbath's
  bleed (extends), Rend (Storm of Blood on Whirlwind; hasted ticks, as SimC's
  `rend_dot_t`), Odyn's Fury's 4s bleed. Bloodborne's +20% to bleeds.
- **Frenzy and Berserk** are asynchronous stack buffs in SimC (each stack
  expires on its own timer), so Rampage's haste and strength stacks settle
  around five, not ten; that alone is worth ~8% of GCD throughput.
- **Gear procs:** Venomfang, Rune of Unleashed Fire, Skyfury's extra swings,
  and the time-averaged Surge of Adrenaline (+30% auto speed and damage for
  6s after a Raging Blow reset, folded in at its observed 57.5% uptime).

## Known gaps

- **GCD allocation, the 3.8%.** Hit by hit the abilities are right; the
  rotation wins ~7 fewer Rampages and ~7 fewer Executes a fight than SimC.
  The rage books balance (both spend ~7,670 rage on Rampage a fight), so the
  difference is *when* the GCDs land: SimC packs 63% of its Rampages into
  Recklessness windows where they cost 50 instead of 80, and the engine's
  flat priority does not wait for rage or for the next Recklessness. The
  spare GCDs go to Whirlwind (9.7 against 5.1, which also doubles Rend's
  ticks) and, outside Recklessness, to Raging Blow (80.3 against 73.7, with
  Crushing Blow -2.3). Closing it needs either a "wait for rage" notion in
  the rotation or the shared Raging Blow / Bloodthirst cooldown categories.
- **Bladestorm casts, -9%.** Its 90s cooldown is cut by Unrelenting Onslaught
  (5s per Executioner stack per Sudden Death Execute); with 2 fewer
  Executioner stacks on average than SimC the engine gets 7.3 casts a fight
  instead of 7.9, which shows up in Bladestorm, its Unhinged Bloodbaths and
  Bloodbath's bleed.
- **Mastery is always on.** Unshackled Fury only applies while Enrage is up
  (99.1%); it's modelled as the abilities' `mastery` flag, which over-weights
  mastery slightly (the +500 mastery probe reads +1.08).
- **Hack and Slash / Bloodthirst's cooldown.** Raging Blow and Crushing Blow
  share one charge pool (as in SimC), and both its reset sources hand back
  every charge like `cooldown_t::reset(true)`; Bloodthirst and Bloodbath do
  *not* share their 4.5s category cooldown, which the engine has no concept
  of, so Bloodbath can come up a fraction more often than it should.
- `--gear` reproduces every item level and stat rating, attack power (3,226
  vs SimC's 3,225) and haste/mastery/versatility exactly, and lands within
  0.5% of the hand-entered stats (`tools/check_gear.py --dps`); crit reads
  +5.4 points because SimC's sheet (24.96%) excludes the ~5 points of
  critical chance the abilities actually have (Cruel Strikes and an
  unmodelled ~3%).
