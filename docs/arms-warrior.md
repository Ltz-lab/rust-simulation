# Arms Warrior (Slayer)

`specs/arms_slayer.toml`, built the same way as the others: coefficients,
costs, cooldowns, durations and talent values from DB2, SimC's warrior module
(`sc_warrior.cpp`) and default APL where a mechanic is scripted, and the rest
observed in SimC's log. Arms is the first spec on rage: auto attacks, Colossus
Smash and Rend ticks fill it, and every 20 spent takes a second off Avatar and
Colossus Smash. Every mechanic below is generic engine data, not warrior code.

## Results

Reference: SimC's Midnight S2 sample Arms profile (Slayer), core kit only (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 164,332 |
| azsim | 163,522 (±138) |
| difference | **-0.49%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| execute_damage | 24,948 | 25,117 | +0.7% | 69.5 | 70.1 |
| slayers_strike | 18,996 | 19,135 | +0.7% | 66.2 | 66.6 |
| mortal_strike | 18,112 | 18,154 | +0.2% | 51.6 | 51.7 |
| bladestorm_mh | 15,250 | 14,220 | -6.8% | 100.9 | 94.2 |
| mortal_strike_bladestorm_unhinged | 15,222 | 14,836 | -2.5% | 48.3 | 47.0 |
| deep_wounds | 14,374 | 14,382 | +0.1% | 295.4 | 295.7 |
| auto_attack_mh | 14,166 | 14,073 | -0.7% | 158.2 | 157.3 |
| reap_the_storm_execute | 12,267 | 12,323 | +0.5% | 44.4 | 44.7 |
| overpower | 9,858 | 9,880 | +0.2% | 41.2 | 41.3 |
| heroic_strike | 7,346 | 7,496 | +2.0% | 26.4 | 26.9 |
| fatality | 3,081 | 3,064 | -0.5% | 28.0 | 27.9 |
| rend_dot | 2,847 | 2,870 | +0.8% | 135.4 | 136.3 |
| slam | 2,213 | 2,370 | +7.1% | 17.2 | 18.4 |
| cleave | 1,403 | 1,397 | -0.4% | 13.7 | 13.7 |
| venomfang | 1,234 | 1,233 | -0.1% | 164.5 | 164.6 |
| rune_of_unleashed_fire | 1,032 | 1,033 | +0.1% | 50.4 | 50.5 |
| colossus_smash | 972 | 974 | +0.1% | 11.1 | 11.2 |
| venomfang_burst | 535 | 537 | +0.4% | 37.2 | 37.4 |
| rune_of_unleashed_fire_lingering | 311 | 309 | -0.7% | 243.4 | 241.8 |
| wrecking_throw | 96 | 101 | +5.5% | 0.6 | 0.6 |
| storm_bolt | 100 | 0 | -100.0% | 8.9 | 0.0 |
| charge_impact | 3 | 0 | -100.0% | 1.0 | 0.0 |

Rage: 4,542 generated a fight (775 lost to the 130 cap) against SimC's 4,772
(861).

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +9.46% | +9.94% | +0.49 pts |
| +500 haste | +9.16% | +9.43% | +0.27 pts |
| +500 mastery | +9.27% | +9.32% | +0.06 pts |
| +500 vers | +8.76% | +8.75% | -0.01 pts |
| +200 strength | +5.47% | +5.47% | -0.00 pts |

Every probe is within 0.5 points. Haste and mastery are 0.11 points apart in
SimC and swap places here (crit > haste > mastery > vers > strength).

## What's modelled

- **Rage**: 24.3 a swing, 26.7 on a crit (SimC's rounded 3.6s × 1.75 × 3.5 ×
  1.1, ×1.1 Seasoned Soldier), Skyfury repeats (20%), Charge, Just Warming Up
  (+15 on Colossus Smash), Bloodsurge (30% of Rend ticks, 5), Frothing
  Berserker (20%, 10% of the cost back) and Improved Execute with Critical
  Thinking (30% of an Execute's rage back). 130 cap.
- **Mortal Strike**: 30 rage, 6s hasted. Executioner's Precision (+35% a
  stack from Executes), Fierce Followthrough (a crit gives the next +20%),
  Fatality's Fatal Mark (50%), Mortal Wounds (crits apply Deep Wounds).
- **Execute** as two variants: free with Sudden Death at double damage, or
  20-40 rage at 2 × spent/40 below 35% health (Massacre). Fatality detonates
  Fatal Mark per stack. An Execute's own Slayer's Strike can grant Sudden
  Death, which that Execute then spends (SimC's order).
- **Sudden Death**: real PPM 1.7 hasted with a 3s ICD, Tactical Edge (a stack
  from Colossus Smash) and Imminent Demise (every third Slayer's Strike).
- **Overpower**: 2 charges, 12s; Tactician gives a charge back on 30% of rage
  spenders, its recharge carrying on.
- **Colossus Smash**: +30% damage from all warrior attacks for 10s. **Avatar**:
  +20% for 20s (derived), +20% autos. Anger Management takes spent/20 seconds
  off both.
- **Bladestorm** as a channel: a tick at the start and 6 over the hasted 6s,
  plus one per Imminent Demise stack fitted into the same time
  (`ticks_expr`, `tick_time_expr`). Unhinged casts Mortal Strike on the odd
  ticks; its end grants Violent Euphoria (+15% haste, 8s). SimC's Bladestorm
  isn't channeled and casts Storm Bolt alongside (0.06% of damage); here the
  channel takes the player.
- **Slayer**: Slayer's Strike (25% on special attacks, 0.5s ICD), Executioner
  (stacks with own timers), Reap the Storm and Unrelenting Onslaught (5s off
  Bladestorm and an Overwhelmed stack per Executioner stack on a Sudden Death
  Execute), Overwhelmed (+1% a stack).
- **Deep Wounds** from Execute and crits, rolling (SimC's rolling periodic),
  7.98s with Bloodletting; **Rend** from Cleave and, in the execute phase,
  Mortal Strike.
- **Master of Warfare**: 15% on Mortal Strike, Slam, Overpower and Execute
  turns the next Slam into Heroic Strike.
- **Wild Strikes**: auto crits give +20% attack speed for 10s (8s ICD).
- The rotation is SimC's slayer_st / slayer_execute lists.

## Engine additions

All generic, all documented in [spec-format.md](spec-format.md#rage-rolling-bleeds-cooldown-ties).
Every other spec gives identical DPS for the same seed.

- DoTs: `refresh = "rolling"`.
- Channels: `ticks_expr`, a tick count evaluated as the channel starts.
- `reduce_cooldown` takes `seconds_expr`.
- `[fight] ready_tolerance`: Mortal Strike's 6s hasted cooldown is exactly 4
  hasted GCDs, a tie the microsecond event clock lost at random (without it,
  -10% Mortal Strikes and the haste probe goes flat).
- `reset_cooldown` takes `keep_recharge`: a charge given back lets the
  recharge in progress carry on (SimC's `cooldown_t::reset`). Off by default,
  which keeps the old restart (Elemental's Lava Burst resets would move with
  it on).
- Fix: an attack-speed buff applied by the swing itself (Wild Strikes) no
  longer starts a second swing timer.

## Gear

`[gear]` matches SimC's ratings and stats exactly without trinkets
(`tools/check_gear.py`): all 13 items, 3,013 attack power, 33.24% crit, 27.62%
haste, 25.47% mastery, 5.28% versatility. DPS through `--gear` is within 0.01%
of the hand-entered stats (163,587 vs 163,609; SimC 164,371, 5,000
iterations). Strength takes plate specialization's ×1.05 and Battle Shout plus
another 3% on attack power (×1.0815); the base strength (936) includes what
Armored to the Teeth gives from armor. `[gear.kit]` adds SimC's default Arms
consumables (Flask of the Blood Knights, Harandar Celebration, Void-Touched
rune, Thalassian Phoenix Oil, Light's Potential in a Colossus Smash window)
and Bloodlust, as Ret's does. `[gear.procs]` carries Battle Stance's 3% crit
and the time-averaged stat procs of this gear set.

## Speed

10,000 iterations on 10 threads: azsim 0.44s, SimC 2.64s (~6×).

## Known gaps

- **Bladestorm (-7% hits, about -0.9% of DPS).** Two parts. SimC casts 9.9 a
  fight against 9.7 here: both cast it right after Colossus Smash (or in
  Avatar), and whether it's back in time depends on how Sudden Death
  Executes cluster before the window. And SimC's ticks re-haste mid-channel
  as Rune of Burning Haste stacks, which adds a short tenth tick to about a
  third of channels. Here the gear's haste procs are a constant average,
  because haste rising between a hasted cooldown's start and the GCDs after
  it breaks Mortal Strike's 4-GCD tie (SimC rescales hasted cooldowns as
  haste changes; this engine doesn't yet).
- **Fitted rather than derived**: the gear's stat procs (time-averaged over 9
  logged fights), the weapon's share of ability damage (strength probe), crit
  damage per ability (observed crit/non-crit ratios).
- **Not modelled**: Heroic Might (SimC adds 0.03% a stack: its value is
  scaled down twice), Charge's tiny hit, Storm Bolt (only cast alongside
  Bladestorm), the MID2 tier set (reported as `unmodelled` with gear lines),
  multi-target (Sweeping Strikes, Cleave's Reap the Storm).
