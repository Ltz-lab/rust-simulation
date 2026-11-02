# Frost Death Knight (Deathbringer)

`specs/frost_deathbringer.toml`, built the same way as the others:
coefficients, costs, cooldowns, durations and talent values from DB2, SimC's
death knight module (`sc_death_knight.cpp`) and the default APL in the profile,
and the rest observed in SimC's log. Frost is the first spec on regenerating
runes and runic power, and the first Deathbringer: Reaper's Mark, Exterminate
and the Frostwyrm's Fury recall are all generic data.

## Results

Reference: SimC's Midnight S2 sample Frost Death Knight profile (Deathbringer),
core kit only (no trinkets, tier set, consumables or Bloodlust), 300s
Patchwerk, 5,000 iterations.

| | DPS |
|---|---:|
| reference | 162,970 |
| azsim | 162,532 (±106) |
| difference | **-0.27%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| howling_blast | 20,007 | 20,101 | +0.5% | 57.9 | 58.2 |
| obliterate_frost | 13,187 | 13,150 | -0.3% | 113.9 | 113.7 |
| auto_attack_mh | 11,369 | 11,538 | +1.5% | 226.1 | 229.5 |
| frost_strike | 10,910 | 10,947 | +0.3% | 97.0 | 97.3 |
| obliterate | 9,959 | 9,932 | -0.3% | 113.9 | 113.7 |
| frostreaper | 8,846 | 8,211 | -7.2% | 34.5 | 32.0 |
| breath_of_sindragosa_damage | 7,231 | 7,662 | +6.0% | 99.4 | 105.3 |
| reapers_mark_explosion | 7,107 | 6,856 | -3.5% | 7.0 | 6.8 |
| frost_fever | 6,828 | 6,716 | -1.6% | 173.4 | 170.7 |
| exterminate | 6,729 | 6,549 | -2.7% | 20.9 | 20.4 |
| auto_attack_oh | 5,536 | 5,756 | +4.0% | 220.3 | 229.1 |
| remorseless_winter_damage | 5,728 | 5,655 | -1.3% | 193.8 | 191.4 |
| frost_strike_offhand | 5,454 | 5,472 | +0.3% | 97.0 | 97.3 |
| icy_death_torrent | 5,108 | 5,257 | +2.9% | 64.3 | 66.2 |
| obliterate_offhand_frost | 5,094 | 5,079 | -0.3% | 113.9 | 113.7 |
| glacial_advance_arctic_assault_obliterate | 5,036 | 4,980 | -1.1% | 105.3 | 104.1 |
| cryogenic_chamber | 4,983 | 4,950 | -0.7% | 6.1 | 6.0 |
| obliterate_offhand | 3,830 | 3,821 | -0.2% | 113.9 | 113.7 |
| empower_rune_weapon | 3,810 | 3,794 | -0.4% | 26.2 | 26.1 |
| hyperpyrexia | 2,972 | 2,944 | -1.0% | 257.5 | 254.8 |
| frostwyrms_fury | 2,902 | 2,891 | -0.4% | 3.7 | 3.7 |
| exterminate_second_hit | 2,768 | 2,691 | -2.8% | 20.9 | 20.3 |
| wave_of_souls | 2,168 | 2,149 | -0.9% | 14.1 | 14.0 |
| chosen_of_frostbrood | 1,502 | 1,478 | -1.6% | 3.7 | 3.6 |
| venomfang | 1,105 | 1,127 | +2.0% | 169.3 | 172.6 |
| rune_of_unleashed_fire | 1,015 | 1,016 | +0.2% | 50.4 | 50.5 |
| reapers_mark | 1,001 | 992 | -0.9% | 7.1 | 7.0 |
| venomfang_burst | 480 | 482 | +0.5% | 39.3 | 39.6 |
| rune_of_unleashed_fire_lingering | 304 | 304 | +0.1% | 241.5 | 241.8 |

Every large source is within 3% of the reference's hit count, and Obliterate
(113.7 against 113.9) and Frost Strike (97.3 against 97.0) land on it.

## What is modelled

- **Runes and runic power.** Six runes, each 10s/haste with at most three
  regenerating at once [simc `MAX_REGENERATING_RUNES`]. The engine refills the
  pool at a flat 0.36/s: SimC's Rune Regeneration grant is 97.3 a fight
  (0.324/s), but it spends them evenly while this engine caps runes in Pillar
  windows where Obliterate is held, so 0.36/s is what reproduces SimC's 113.6
  Obliterates. Obliterate 2, Howling Blast 1, Reaper's Mark 2; Runic
  Empowerment (1.8% per RP spent), Murderous Efficiency (20% per Killing
  Machine stack, rolled a stack at a time), Obliteration (20% during Pillar)
  and Breath of Sindragosa (2 on expiry) give them back. Runic power (max
  100): Obliterate 20 (10 while Exterminate is armed: its -50% cost modifier
  hits the negative RP cost too), Reaper's Mark 20, Empower Rune Weapon 40,
  Runic Attenuation 3 (RPPM 10.8 hasted), Rage of the Frozen Champion 8 per
  Rime Howling Blast and Frost Fever ticks (30%, 4); Frost Strike spends 35,
  Breath 60 and rolls its 1.08 Runic Empowerment. Howling Blast gives no
  runic power of its own: its -10 RP cost is not paid out (SimC's log).
- **Killing Machine** (2 stacks, 10s): auto-attack crits, Empower Rune
  Weapon, Exterminate, Reaper's Mark and every Frost Strike / Howling Blast
  during Pillar of Frost (Obliteration). SimC's 30% accumulated RNG procs on
  ~49.8% of auto crits (30% x attempts since the last proc), so the spec rolls
  50%. Spent, it stacks Killing Streak (+1.5% haste a stack, asynchronous
  expiry), Bonegrinder (crit, then +10% frost at 5 stacks), Murderous
  Efficiency, Dark Talons (25% for 3 Icy Talons stacks and +18% attack speed
  for 6s) and extends Breath by 0.8s a stack. Obliterate consumes every stack
  at once (Killing Streak).
- **Rime** (60% per Frost Strike [db2 59057 + 377076]): Howling Blast is free,
  +275% damage (derived from the buff's spell modifiers) and 8 RP; Bind in
  Darkness adds 60% on top, Frostbound Will 6s off Empower Rune Weapon, and
  Breath 0.8s. A Rime Howling Blast still fires the rune-spend hooks (Gathering
  Storm stack and extension, Rune Mastery) because SimC spends its base cost
  even when it is free.
- **Icy Talons** (6% attack speed a stack, 5 stacks: 3 + Smothering Offense's
  2) from every RP spend; Frost Fever, Killing Streak, Chosen of Frostbrood's
  +15% haste and the time-averaged Rune of Burning Haste complete the haste.
- **Pillar of Frost** (+20% strength, 12s, 45s) starts a free Remorseless
  Winter (Frozen Dominion): 12s of 1s ticks, +0.5s a rune spent (Gathering
  Storm, which also stacks +10% damage up to 10), applying Everfrost (+6% frost
  damage taken a stack). Frostwyrm's Fury extends it 4s and the Chosen of
  Frostbrood recall 2s (SimC's log). Its +450% Remorseless Winter modifier
  comes from the derived layer. Enduring Strength follows Pillar's expiry.
- **Reaper's Mark** (2 runes, 45s, 12s): every frost/shadowfrost hit adds 2
  stacks (4 on crit), the debuff explodes for 0.544 AP a stack, arms
  Exterminate (12.88 AP + 5.72 AP after 0.5s, one a stack: the next
  Obliterates cost 1 rune and pull one scythe, and Arctic Assault fires one
  Glacial Advance at Killing-Machine-stack strength), and the cast
  also fires Wave of Souls (2.54 AP; the return wave always crits), a Killing
  Machine and Bonegrinder stacks.
- **Frostwyrm's Fury** (90s) and the Chosen of Frostbrood recall (~52% of it,
  when Razorice is at 5 stacks), the haste buff (both casts) and the Pillar
  extensions.
- **Frost Fever** (24s, 2.55s [obs] hasted ticks, 30% RP a tick, 15% Brittle),
  Razorice (5 stacks from Arctic Assault), Icy Death Torrent (50% per auto
  crit), Frostreaper (RPPM 5 hasted on Obliterate, spent by Frost Strike),
  Hyperpyrexia (35% of each Frost Strike / Arctic Assault / Breath hit banks
  30% of it into a rolling 4s DoT, so its uptime and 257 ticks come out of
  the proc rate) and Cryogenic Chamber (15% of Rime Howling Blasts, released
  by the next Remorseless Winter).
- **Gear**: Venomfang (DoT + burst), Rune of Unleashed Fire with its Lingering
  DoT, Unholy Strength, Rune Mastery, Rune of Burning Haste (folded into the
  base haste), Venomcursed crit (folded into the crit chance), Skyfury repeats
  (13% of hits: SimC's 0.1s ICD and synced dual-wield swings halve the 20%),
  and the 16.4% dual-wield miss on both hands.

Engine additions made for this spec, all generic: `feed` entries take a
`chance` (Hyperpyrexia) with its own random stream, and `spent.<resource>` is
reset to zero when a cast is free (Obliterate's RP gain is `10 x spent.runes`:
20 normally, 10 under Exterminate, 0 when Empower Rune Weapon pays).

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +12.56% | +10.54% | -2.02 pts |
| +500 haste | +8.32% | +7.68% | -0.64 pts |
| +500 mastery | +9.57% | +8.67% | -0.91 pts |
| +500 vers | +8.77% | +8.99% | +0.23 pts |
| +200 strength | +6.85% | +8.34% | +1.48 pts |

Versatility, haste and mastery are within a point. The reference's crit gain
beats pure crit scaling (+11.98% for the whole bar): every extra point of crit
also buys Killing Machine procs from auto crits, which the reference gets more
of. Strength runs the other way because every coefficient here is attack
power, while the reference's weapon damage does not scale with AP.

## Known gaps

- **Frostreaper** procs 32.0 against 34.5 (-7%): the per-hit real-PPM roll
  gives a slightly lower hasted rate than SimC's (~1.28 against 1.38 effective
  haste), so its damage share is 7% low.
- **Breath of Sindragosa** runs 105 ticks against 99: the engine's
  extension per Killing Machine stack and Rime lands a couple of ticks late
  per breath. Its DPS is +6% because the hit count carries it.
- **Auto attacks** land 229.5 against 226.1 and the off hand 229.1 against
  220.3: the swing cadence matches, but the reference reports a higher off-hand
  miss rate (17.0% against 16.4%).
- **Crit probe** is 2 points low and the **strength probe** 1.5 high, for the
  reasons above; the stat priority keeps crit first and swaps mastery and
  versatility.
- **Hyperpyrexia** banks into one rolling DoT (like SimC's residual action)
  but at 35% per hit instead of per damaging event; 254.8 ticks against 257.5.
- Killing Machine is consumed with the Obliterate it empowers; SimC's 30-50ms
  delay doesn't matter on one target.
- No gear lines (`[gear]`), so `--gear` isn't supported for this spec yet.

## Gear

`[gear]`, `[gear.kit]` and `[gear.omnium]` let `--gear profiles/MID2_Death_Knight_Frost.simc`
build the character from its items (docs/gear.md). Constants, all measured on SimC's buffed stats for the
sample profile (human, no trinkets, tier set off):

| constant | value | measured as |
|---|---:|---|
| `base_primary` | 760.7 | 2611 buffed strength / 1.05 less the items' 1726 |
| `primary_mult` | 1.05 | plate specialization |
| `attack_power_mult` | 1.05 | 2611 strength -> 2741 attack power (one-handers: no flat weapon attack power) |
| `crit_bonus` | 0.05 | 19.37% at 661 rating |
| `haste_mult` | 1.04 | 26.41% at 948 rating |
| `rating_mult` | 1.02 each | Human: 648/929/1355/107 item rating -> 661/948/1382/109 |
| `mastery_points`, `mastery_coeff` | 10, 2.0 | 80.08% at 1382 rating |
| `versatility_bonus` | 0.03 | Mark of the Wild |
| `spec_id` | 251 | ChrSpecialization |
| `[gear.procs] haste` | 0.0798 | Rune of Burning Haste's average already in `[player].haste` (0.3439 - 0.2641) |

Kit, from SimC's buffed stats with each consumable alone: food and augment rune +83 strength (79.1 before
the multiplier), Flask of the Shattered Sun +168 crit, Thalassian Phoenix Oil +33 crit and +32 haste on the two
weapons, Bloodlust at the pull. Potion of Recklessness (rank 2) gives +1759.5 mastery and -236.6 versatility
for 30s (SimC's stat timeline; 1725 and -232 with Human's 2%); azsim uses it with Pillar of Frost
(`variable.cooldown_check`). Omnium (SimC ablate, 20,000 iterations, core kit: 163,016 with all traits,
158,100 with none, +3.11%): Unleashed Fire 1.01600, Burning Haste 1.01193, Overload 1.00176, Lingering
1.00105, Lynxlike Reflexes 1.00008.

Against SimC on the sample profile without trinkets (`tools/check_gear.py`, passing `--spec` and `--calib`
because `spec=frost` is also the Frost Mage's):

| stat | azsim | SimC |
|---|---:|---:|
| attack power | 2741 | 2741 |
| crit | 19.37% | 19.37% |
| haste | 26.40% | 26.41% |
| mastery | 80.08% | 80.08% |
| versatility | 5.02% | 5.02% |

Every item level matches; the rating rows differ by Human's 2% because `check_gear.py` prints the item sum
before `rating_mult`. DPS (azsim 10,000 iterations seed 1 / 5,000 seed 42 in check_gear, SimC 20,000 / 5,000):

| run | SimC | azsim |
|---|---:|---:|
| core kit (no consumables, Bloodlust) | 163,086 | 162,593 (-0.30%) |
| full kit (consumables, Bloodlust, potion) | 191,744 | 189,426 (-1.21%) |

The tier set (set 2055) is modelled, see below. The two trinkets
(Voracious Heart of Ula'tek, Zul'jin's Guillotine Technique) and the Bite of Zul'jan 2pc are in the effect library.

## Tier set (Midnight season 2, set 2055)

`[[gear.effects]]` in the spec, applied with gear lines; `--set gear.set_bonuses=0` turns it off. Tier gain is the
sample profile with the set against the same profile without it (SimC: `set_bonus=midnight_season_2_{2,4}pc=0`;
SimC about 400 iterations to 0.3% error, azsim 20,000 iterations, seed 1):

| bonus | what it does | modelled as | SimC | azsim |
|---|---|---|---:|---:|
| 2pc (1296652) | each Remorseless Winter tick: a stack of Freezing Tempest (5s, +1% auto-attack speed and +2% Icy Death Torrent a stack); a new Remorseless Winter cast removes it | buff with `attack_speed_pct` and `ability_pct`; a stack on every tick of the Remorseless Winter DoT (new `dot =` patch), consumed first in the cast (new `prepend =` patch) | +5.31% | +4.49% |
| 4pc (1296653) | Remorseless Winter ticks 20% sooner (1s to 0.8s); Freezing Tempest lasts 5s longer | `tick_rate_pct` of 25 on the DoT, buff duration 10s | +2.75% (on the 2pc) | +3.41% |
| both | | | **+8.21%** | **+8.06%** |

Full profile: SimC 260,861, azsim 257,531 (-1.28%); set off: SimC 241,074, azsim 238,328 (-1.14%). Per fight with the
set (SimC / azsim): 347 / 292 main-hand swings and 108 / 107 Icy Death Torrents against 289 / 243 and 91 / 89 without
it; Remorseless Winter ticks 261 / 252 against 208 / 200. The swing counts differ by the same 16% with the set off,
which is the gear-profile gap and not the set's. The two bonuses split differently (azsim gets less from the 2pc,
more from the 4pc; I did not find why) and sum to within 0.15 points.

The removal of Freezing Tempest by the next Remorseless Winter cast matters. SimC's log has a first window of 49
ticks (38s) whose buff is removed at the next cast, 7.6s after the window instead of after the full 10s tail.
Without that removal azsim carried the stacks into the next window (42.5 stacks per window over 40 seeds' logs, SimC
about 37) and gave +9.25%; with it, 39.8 stacks and +8.06%.

Not modelled: nothing of the bonuses themselves. Freezing Tempest's stacks are one timer (SimC's too), and the first tick
of each window comes at the cast, as in SimC's log.

