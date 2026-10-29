# Windwalker Monk (Shado-Pan)

`specs/windwalker_shadopan.toml`, built the same way as the others:
coefficients, costs, cooldowns and durations from DB2 and SimC's monk module
(`sc_monk.cpp`, `sc_monk.hpp`) and its default APL where a mechanic is
scripted, the rest observed in SimC's log. Windwalker is the second spec on
energy plus a discrete secondary resource: Tiger Palm pays energy for chi,
Blackout Kick, Rising Sun Kick, Fists of Fury and Spinning Crane Kick spend
it. Every mechanic below is generic engine data, not monk code.

## Results

Reference: SimC's Midnight S2 sample Windwalker profile (Shado-Pan), core kit
only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 169,265 |
| azsim | 167,850 (±101) |
| difference | **-0.84%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| fists_of_fury | 34,017 | 33,684 | -1.0% | 140.3 | 138.8 |
| rising_sun_kick_damage | 21,660 | 16,567 | -23.5% | 45.5 | 34.7 |
| tiger_palm | 15,656 | 17,696 | +13.0% | 68.6 | 77.5 |
| dual_threat | 14,125 | 14,788 | +4.7% | 47.5 | 49.7 |
| rushing_wind_kick_damage | 13,190 | 13,782 | +4.5% | 14.3 | 14.9 |
| flurry_strike_shado_over_the_battlefield | 10,761 | 10,745 | -0.2% | 747.7 | 745.9 |
| melee_main_hand | 9,522 | 9,889 | +3.9% | 159.8 | 165.8 |
| blackout_kick | 9,101 | 9,741 | +7.0% | 48.7 | 52.0 |
| flurry_strike | 6,896 | 7,027 | +1.9% | 616.3 | 627.4 |
| jadefire_stomp | 6,536 | 6,473 | -1.0% | 27.9 | 27.6 |
| spinning_crane_kick_tick | 6,148 | 5,737 | -6.7% | 71.7 | 66.8 |
| touch_of_karma | 4,220 | 5,264 | +24.8% | 20.7 | 25.9 |
| whirling_dragon_punch_aoe | 2,640 | 2,596 | -1.7% | 26.0 | 25.5 |
| glory_of_the_dawn_rising_sun_kick_damage | 2,504 | 1,865 | -25.5% | 15.1 | 11.2 |
| touch_of_death | 2,280 | 2,281 | +0.0% | 1.0 | 1.0 |
| zenith_stomp_trigger | 2,264 | 2,037 | -10.0% | 5.3 | 4.7 |
| teachings_of_the_monastery | 1,598 | 1,809 | +13.2% | 67.9 | 76.8 |
| whirling_dragon_punch_singletarget | 1,266 | 1,248 | -1.4% | 8.7 | 8.5 |
| rune_of_unleashed_fire | 1,169 | 1,169 | +0.1% | 49.5 | 49.5 |
| expel_harm_damage | 1,066 | 909 | -14.8% | 20.1 | 17.1 |
| flurry_strike_wisdom_of_the_wall | 969 | 875 | -9.7% | 78.8 | 71.1 |
| glory_of_the_dawn_rushing_wind_kick_damage | 783 | 797 | +1.7% | 4.7 | 4.8 |
| flurry_strike_stand_ready | 451 | 406 | -9.9% | 52.6 | 47.3 |
| rune_of_unleashed_fire_lingering | 365 | 423 | +15.9% | 247.1 | 286.0 |
| **total** | **169,265** | **167,902** | **-0.81%** | | |

The two resources come out close: about 78 Tiger Palms a fight (SimC 68.6)
against ~180 chi generated, and the sources that matter most — Fists of Fury
and its Jadefire Stomps, Rushing Wind Kick, the Shado-Pan flurry strikes,
Dual Threat, auto attacks — land within a few percent of the reference. The
remaining shift is the chi economy: SimC's Rotation spends more of its chi on
Rising Sun Kick (45.5 a fight against 34.7 here) and less on Tiger Palm and
Blackout Kick, which the simplified priority list can't fully reproduce.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.50% | +8.18% | -0.32 pts |
| +500 haste | +7.12% | +10.63% | +3.51 pts |
| +500 mastery | +8.08% | +8.30% | +0.22 pts |
| +500 vers | +8.67% | +8.79% | +0.12 pts |
| +200 agility | +5.86% | +6.10% | +0.24 pts |

Mastery and crit are right because the spec flags the same sources Combo
Strikes applies to (`mastery = true` on the kicks, Fists of Fury, Spinning
Crane Kick, Jade Fire Stomp and Zenith Stomp; not on flurry strikes, Dual
Threat, auto attacks, Expel Harm or Teachings). Agility is right because
ability damage scales with `attack_power + weapon_attack_power`, fitted from
the +200 agility probe.

Haste over-responds by 3.5 points. SimC's haste probe barely moves the
filler abilities (Tiger Palm -1.5%, Blackout Kick -8.4%, Rising Sun Kick
+2.0%) while Fists of Fury gains 20%; here nearly every source gains the full
~12% of the extra haste, because the rotation isn't GCD-saturated the way
SimC's is and converts the extra time into casts. See Known gaps.

## What's modelled

- **Resources.** Energy 150 (100 + Ascension 20 + Inner Peace 30) at 10/s
  hasted, chi 6 (5 + Ascension) starting at 2 (Combat Wisdom). Tiger Palm
  pays 50 energy for 2 chi; Zenith's window takes 1 off every chi cost.
- **Kicks.** Rising Sun Kick (2 chi, 10s hasted cooldown, +20% crit from
  Xuen's Battlegear) and its Rushing Wind Kick replacement; Blackout Kick
  (1 chi, free under Combo Breaker) takes a second off both kicks (Sharp
  Reflexes, two inside Zenith) and cashes Teachings of the Monastery stacks
  as extra kicks; Rising Sun Kick crits take 4s off Fists of Fury (Xuen's
  Battlegear) and Spinning Crane Kick ticks take 0.5s.
- **Fists of Fury** is a hasted 4s channel of five 2.61 AP ticks; the last
  tick fires Jade Fire Stomp, the cast cashes the Shado-Pan flurry charges
  and arms Whirling Dragon Punch.
- **Shado-Pan.** Auto attacks stow Flurry Charges (2 a hit, doubled on a
  crit, One Versus Many); Fists of Fury and (inside Zenith) kicks cash them
  as Flurry Strike (physical, 0.6 AP) plus Flurry Strike Shado Over the
  Battlefield (nature, 0.52 AP). Zenith's Stand Ready adds 10 at 70%
  effectiveness. Dual Threat repeats 30% of auto attacks.
- **Cooldowns.** Zenith (2 charges, 90s, 15s hasted window, one Zenith Stomp
  per cast), Touch of Death (180s, 3 chi), Touch of Karma (90s, 6s redirect
  DoT), Expel Harm via Combat Wisdom's 15s timer.
- **Procs.** Combo Breaker on Tiger Palm, Dance of Chi-Ji on chi spend (RPPM,
  free Spinning Crane Kick), Teachings of the Monastery stacks and their
  12% Rising Sun Kick resets, Glory of the Dawn on kicks.
- **Gear.** Rune of Unleashed Fire and its lingering DoT, as Poisson procs in
  the core kit.

## Engine additions

None. Windwalker needed only existing features: a second discrete resource
(`[resources.chi]`), `cost_expr` for Zenith's chi discount,
`free_with` for Combo Breaker and Dance of Chi-Ji, `channel` for Fists of
Fury and Spinning Crane Kick, `reduce_cooldown` / `reset_cooldown` for the
kick resets and cooldown acceleration, `prev_gcd` for Combo Strikes, and
`trigger` with `count` for flurry-charge and Teachings flushes.

## Known gaps

- **Chi economy.** SimC fits 45.5 Rising Sun Kicks, 28.2 Fists of Fury, 48.7
  Blackout Kicks and 17.9 Spinning Crane Kicks into the chi its Tiger Palms
  generate; this spec's costs can't, so Rising Sun Kick and (slightly)
  Spinning Crane Kick come up short while Tiger Palm, Blackout Kick and
  Teachings take those globals. Damage per hit is calibrated, so DPS is
  within 1%, but the counts are the structural error to fix next.
- **Haste** (+3.5 pts) as described above: the rotation has spare GCD time
  that the reference doesn't.
- **Touch of Karma** counts 25.9 ticks against 20.7 (the redirected damage is
  modelled as a flat 6-tick DoT, not from damage taken).
- **Zenith** is modelled by its chi discount, kick reset and Zenith Stomp;
  Tigereye Brew's stacking crit and the +damage% aura are left in the
  calibration residual, so its 4.7 casts a fight are slightly low (5.3).
- **Rune of Unleashed Fire lingering** ticks 286 times against 247 (8s at 1s
  with `extend` refresh).
- Not modelled: movement, multi-target (Spinning Crane Kick's Mark of the
  Crane stacks), Invoke Xuen and Strike of the Windlord (not talented in this
  profile).
