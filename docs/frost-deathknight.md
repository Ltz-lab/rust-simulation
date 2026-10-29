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
| reference | 162,879 |
| azsim | 158,453 (±88) |
| difference | **-2.72%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| howling_blast | 19,925 | 22,712 | +14.0% | 57.7 | 65.6 |
| obliterate_frost | 13,179 | 11,316 | -14.1% | 113.9 | 97.8 |
| frost_strike | 10,901 | 11,970 | +9.8% | 97.0 | 106.4 |
| auto_attack_mh | 11,375 | 10,258 | -9.8% | 226.5 | 204.1 |
| obliterate | 9,950 | 8,543 | -14.1% | 113.9 | 97.8 |
| frostreaper | 8,891 | 7,447 | -16.2% | 34.6 | 29.0 |
| breath_of_sindragosa_damage | 7,232 | 6,224 | -13.9% | 99.4 | 85.5 |
| reapers_mark_explosion | 7,107 | 6,871 | -3.3% | 7.0 | 6.8 |
| frost_fever | 6,825 | 6,991 | +2.4% | 173.4 | 177.6 |
| exterminate | 6,725 | 6,540 | -2.8% | 20.9 | 20.4 |
| frost_strike_offhand | 5,454 | 5,993 | +9.9% | 97.0 | 106.4 |
| cryogenic_chamber | 4,955 | 5,748 | +16.0% | 6.1 | 7.0 |
| remorseless_winter_damage | 5,719 | 4,580 | -19.9% | 193.6 | 154.9 |
| glacial_advance_arctic_assault_obliterate | 5,037 | 5,659 | +12.3% | 105.2 | 118.1 |
| auto_attack_oh | 5,535 | 5,126 | -7.4% | 220.2 | 203.9 |
| icy_death_torrent | 5,120 | 4,682 | -8.6% | 64.3 | 58.8 |
| obliterate_offhand_frost | 5,091 | 4,371 | -14.2% | 113.9 | 97.8 |
| obliterate_offhand | 3,827 | 3,286 | -14.1% | 113.9 | 97.8 |
| empower_rune_weapon | 3,814 | 3,492 | -8.4% | 26.1 | 24.0 |
| hyperpyrexia | 2,972 | 3,431 | +15.5% | 257.4 | 297.0 |
| frostwyrms_fury | 2,894 | 2,879 | -0.5% | 3.7 | 3.7 |
| exterminate_second_hit | 2,761 | 2,681 | -2.9% | 20.9 | 20.3 |
| wave_of_souls | 2,172 | 2,157 | -0.7% | 14.1 | 14.0 |
| chosen_of_frostbrood | 1,493 | 1,469 | -1.6% | 3.7 | 3.6 |
| venomfang | 1,112 | 1,131 | +1.6% | 169.6 | 172.4 |
| rune_of_unleashed_fire | 1,014 | 1,016 | +0.2% | 50.3 | 50.4 |
| reapers_mark | 1,000 | 994 | -0.7% | 7.1 | 7.0 |
| venomfang_burst | 481 | 481 | -0.1% | 39.5 | 39.5 |
| rune_of_unleashed_fire_lingering | 304 | 304 | +0.1% | 241.3 | 241.4 |

## What is modelled

- **Runes and runic power.** Six runes, each 10s/haste; at most three
  regenerate at once [simc `MAX_REGENERATING_RUNES`], so the pool refills at
  0.3 runes/s x haste. Obliterate 2, Howling Blast 1, Reaper's Mark 2; Runic
  Empowerment (1.8% per RP spent), Murderous Efficiency (20% per Killing
  Machine stack), Obliteration (20% during Pillar) and Breath of Sindragosa
  (2 on expiry) give them back. Runic power: Obliterate 20, Howling Blast 10
  (20 with Rime), Reaper's Mark 20, Empower Rune Weapon 40, Runic Attenuation
  3 (RPPM 10.8), Rage of the Frozen Champion 8 per Rime Howling Blast, Frost
  Fever 4 on a 30% tick; Frost Strike spends 35, Breath 60.
- **Killing Machine** (2 stacks, 10s): auto-attack crits (30%), Empower Rune
  Weapon, Exterminate, Reaper's Mark and every Frost Strike / Howling Blast
  during Pillar of Frost (Obliteration). It guarantees Obliterate crits and,
  spent, stacks Killing Streak (+1.5% haste a stack, asynchronous expiry),
  Bonegrinder (crit, then +10% frost) and Murderous Efficiency, and extends
  Breath.
- **Rime** (62% [obs] on Frost Strike): Howling Blast is free, +275% damage
  (derived from the buff's spell modifiers) and 20 RP; Bind in Darkness adds
  60% on top, Rage of the Frozen Champion 8 RP, Frostbound Will 6s off Empower
  Rune Weapon, and Breath 0.8s.
- **Pillar of Frost** (+20% strength, 12s, 45s) starts a free Remorseless
  Winter (Frozen Dominion): 12s of 1s ticks, +0.5s a rune spent (Gathering
  Storm, which also stacks +10% damage), applying Everfrost (+6% frost damage
  taken a stack). Its +450% Remorseless Winter modifier comes from the derived
  layer. Enduring Strength follows Pillar's expiry.
- **Reaper's Mark** (2 runes, 45s, 12s): every frost/shadowfrost hit adds 2
  stacks (4 on crit), the debuff explodes for 0.544 AP a stack, arms
  Exterminate (12.88 AP + 5.72 AP after 0.5s, one a stack), and the cast also
  fires Wave of Souls (2.54 AP; the return wave always crits), a Killing
  Machine and Bonegrinder stacks.
- **Frostwyrm's Fury** (90s) and the Chosen of Frostbrood recall (~52% of it,
  when Razorice is at 5 stacks), the haste buff and the Pillar extension.
- **Frost Fever** (24s, 2.4s [obs] hasted ticks, 30% RP a tick, 15% Brittle),
  Razorice (5 stacks from Glacial Advance's Arctic Assault), Icy Death Torrent
  (50% per auto crit), Frostreaper (RPPM 5 on Obliterate, spent by Frost
  Strike), Hyperpyrexia (fed at 35% x 30% into a banked 4s DoT) and Cryogenic
  Chamber (15% of Rime Howling Blasts, released by Remorseless Winter).
- **Gear**: Venomfang (DoT + burst), Rune of Unleashed Fire with its Lingering
  DoT, Unholy Strength, Rune Mastery, Rune of Burning Haste (folded into the
  base haste), Venomcursed crit (folded into the crit chance).

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +12.63% | +10.11% | -2.52 pts |
| +500 haste | +8.51% | +8.79% | +0.28 pts |
| +500 mastery | +9.58% | +8.87% | -0.71 pts |
| +500 vers | +8.92% | +8.99% | +0.08 pts |
| +200 strength | +6.95% | +8.35% | +1.40 pts |

## Known gaps

- **Obliterate count**: 97.8 against 113.9. The reference casts about nine
  Obliterates without Killing Machine (when runic power is under 35); the
  engine's runic power sits higher, so Frost Strike takes those GCDs and
  Obliterate (and with it the rune spend) is a sixth lower. Remorseless
  Winter's length (-20% ticks) and Breath's extensions (-14%) follow from the
  same lower rune spend.
- **Hyperpyrexia** is fed continuously at 35% x 30% into one banked DoT
  instead of 35% of hits each starting their own 4s burn: total damage matches
  (+15% because the DoT is always ticking), the 257 ticks against 297 are not
  the same shape.
- **Auto attacks** land 204 against 226 hits: the reference's logged swing
  cadence is the same, but its reported executes per fight are higher.
- **Crit probe** is 2.5 points low: the crit baseline includes the
  time-averaged Venomcursed and Brittle procs, so the marginal value of +500
  rating is smaller than the reference's snapshot-based one.
- **Strength probe** is 1.4 points high: attack power is 2741 (the buffed
  snapshot) with the strength procs as buffs; the reference's in-fight average
  is higher, so +233 AP is a bigger share here.
- Killing Machine is consumed with the Obliterate it empowers; the small
  delay (30-50ms) SimC uses doesn't matter on one target.
- No gear lines (`[gear]`), so `--gear` isn't supported for this spec yet.
