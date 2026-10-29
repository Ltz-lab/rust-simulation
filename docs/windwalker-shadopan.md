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
| azsim | 167,124 (±109) |
| difference | **-1.26%** |

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.50% | +7.80% | -0.70 pts |
| +500 haste | +7.12% | +8.02% | +0.90 pts |
| +500 mastery | +8.08% | +8.29% | +0.22 pts |
| +500 vers | +8.67% | +8.79% | +0.12 pts |
| +200 agility | +5.86% | +5.97% | +0.11 pts |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| fists_of_fury | 34,017 | 36,822 | +8.2% | 140.3 | 152.0 |
| rising_sun_kick_damage | 21,660 | 20,203 | -6.7% | 45.5 | 42.4 |
| tiger_palm | 15,656 | 15,093 | -3.6% | 68.6 | 66.2 |
| dual_threat | 14,125 | 15,117 | +7.0% | 47.5 | 50.9 |
| rushing_wind_kick_damage | 13,190 | 11,146 | -15.5% | 14.3 | 12.1 |
| flurry_strike_shado_over_the_battlefield | 10,761 | 10,902 | +1.3% | 747.7 | 757.7 |
| melee_main_hand | 9,522 | 10,118 | +6.3% | 159.8 | 169.9 |
| blackout_kick | 9,101 | 6,517 | -28.4% | 48.7 | 34.9 |
| flurry_strike | 6,896 | 7,275 | +5.5% | 616.3 | 650.6 |
| jadefire_stomp | 6,536 | 7,082 | +8.4% | 27.9 | 30.3 |
| spinning_crane_kick_tick | 6,148 | 6,003 | -2.4% | 71.7 | 70.0 |
| touch_of_karma | 4,220 | 4,502 | +6.7% | 20.7 | 22.1 |
| whirling_dragon_punch_aoe | 2,640 | 2,573 | -2.5% | 26.0 | 25.3 |
| glory_of_the_dawn_rising_sun_kick_damage | 2,504 | 2,273 | -9.2% | 15.1 | 13.7 |
| touch_of_death | 2,280 | 2,279 | -0.1% | 1.0 | 1.0 |
| zenith_stomp_trigger | 2,264 | 2,030 | -10.3% | 5.3 | 4.7 |
| teachings_of_the_monastery | 1,598 | 1,535 | -3.9% | 67.9 | 65.3 |
| whirling_dragon_punch_singletarget | 1,266 | 1,236 | -2.4% | 8.7 | 8.5 |
| rune_of_unleashed_fire | 1,169 | 1,170 | +0.1% | 49.5 | 49.6 |
| expel_harm_damage | 1,066 | 967 | -9.2% | 20.1 | 18.3 |
| flurry_strike_wisdom_of_the_wall | 969 | 734 | -24.3% | 78.8 | 59.7 |
| glory_of_the_dawn_rushing_wind_kick_damage | 783 | 644 | -17.8% | 4.7 | 3.9 |
| flurry_strike_stand_ready | 451 | 406 | -10.0% | 52.6 | 47.4 |
| rune_of_unleashed_fire_lingering | 365 | 423 | +15.9% | 247.1 | 286.5 |
| **total** | **169,265** | **167,124** | **-1.26%** | | |

The energy side lands: Tiger Palm 66.2 a fight against SimC's 68.6 (55 energy
each out of ~3,900 regenerated), Teachings 65.3 against 67.9, and the Shado-Pan
layer (flurry strikes, Dual Threat, auto attacks) is within a few percent.

## What's modelled

- **Resources.** Energy 150 (100 + Ascension 20 + Inner Peace 30) at 10/s
  hasted (12.94/s), chi 6 (5 + Ascension) starting at 2 (Combat Wisdom).
  Tiger Palm pays 55 energy for 2 chi; Zenith's window takes 1 off every chi
  cost, Blackout Kick's included.
- **The GCD is a flat 1.0s**, not hasted: SimC's log has every global exactly
  1.00s apart at 29.4% haste, which changes what haste can buy.
- **Kicks.** Rising Sun Kick (2 chi, 10s hasted cooldown, +20% crit from
  Xuen's Battlegear) and **Rushing Wind Kick as its own action** (own 10s
  hasted cooldown, no chi, only while its buff is up, which in SimC makes
  Rising Sun Kick uncastable). Blackout Kick (1 chi, free under Combo Breaker)
  cuts both kicks' cooldowns (Sharp Reflexes, doubled inside Zenith), cashes
  Teachings stacks as extra kicks, and rolls the 12% Teachings reset; Rising
  Sun Kick crits take 4s off Fists of Fury (Xuen's Battlegear), and every
  Glory of the Dawn strike inherits the same crit rule; Spinning Crane Kick
  ticks take 0.5s.
- **Fists of Fury** is a hasted 4s channel of five 2.61 AP ticks; the last
  tick fires Jade Fire Stomp, the cast cashes the Shado-Pan flurry charges
  and arms Whirling Dragon Punch.
- **Shado-Pan.** Auto attacks and Dual Threat store Flurry Charges (2 a hit
  for a 2H, doubled to 4 on a crit, One Versus Many); Fists of Fury cashes
  them as Flurry Strike (physical, 0.6 AP) plus Shado Over the Battlefield
  (nature, 0.52 AP). Wisdom of the Wall, on kicks and Spinning Crane Kicks
  inside Zenith, adds 3 strikes of each kind without spending charges.
  Zenith's Stand Ready adds 10 at 70% effectiveness. Dual Threat repeats
  ~30% of auto attacks.
- **Cooldowns.** Zenith (2 charges, 90s, 15s hasted window, one Zenith Stomp
  and 10 Stand Ready strikes per cast), Touch of Death (180s, 3 chi), Touch
  of Karma (90s, a 6-tick redirect DoT), Expel Harm from Combat Wisdom.
- **Procs.** Combo Breaker: 8% per Tiger Palm [db2 137384] and 100% per
  Spinning Crane Kick paid by Dance of Chi-Ji (Sequenced Strikes). Dance of
  Chi-Ji arms 17.9 free Spinning Crane Kicks a fight; Teachings of the
  Monastery stacks 1 per Tiger Palm, cashed by Blackout Kick; Glory of the
  Dawn procs off kicks at 1.1 x haste.
- **Gear.** Rune of Unleashed Fire and its lingering DoT, as Poisson procs in
  the core kit.

## Engine additions

None. Windwalker needed only existing features: a second discrete resource
(`[resources.chi]`), `cost_expr` for Zenith's chi discount, `free_with` for
Combo Breaker and Dance of Chi-Ji, `channel` for Fists of Fury and Spinning
Crane Kick, `reduce_cooldown` / `reset_cooldown` for the kick resets and
cooldown cuts, and `trigger` with `count` for the flurry and Teachings
flushes.

## Known gaps

- **Blackout Kick and Rushing Wind Kick are short** (34.9 vs 48.7 and 12.1 vs
  14.3) while **Fists of Fury, auto attacks and Dual Threat run hot** (+8%,
  +6%, +7%). SimC's APL fits about ten more free Blackout Kicks (mostly inside
  Zenith, where its chi discount makes them free) into the same globals; this
  priority list can only get them by taking chi and globals away from Fists of
  Fury, which costs more damage. The DPS total is close because the two errors
  partly cancel.
- **Kick cadence.** SimC lands a Rising Sun Kick every ~6.6s. A 10s hasted
  cooldown with Sharp Reflexes' 1s cut per Blackout Kick only gets one every
  ~7.2s here, because the fixed priority list leaves the kick ready while it
  channels Fists of Fury (the cooldown cut then lands on a finished cooldown).
  The spec therefore carries a 9s cooldown and a 4s cut (4x the tooltip, 8s in
  Zenith) so the observed cadence matches; with the raw 1s cut the kicks come
  up 8% short and DPS is -3.6%. This is the one number in the file that is
  fitted to the reference's cadence rather than read off the spell data.
- **Haste** still over-responds slightly (+0.90 pts): the rotation leaves a
  few seconds of GCDs unused per fight, so extra haste buys casts that the
  reference doesn't get.
- **Touch of Karma** ticks 22.1 times against 20.7 (the redirect is modelled
  as a 6-tick DoT at a fixed 16.26 AP a tick, not from damage taken). Touch of
  Death and both Rune of Unleashed Fire sources are flat damage, as the probes
  show (they don't move with agility).
- **Zenith** is modelled by its chi discount, kick reset, Zenith Stomp and
  Stand Ready; Tigereye Brew's stacking crit and the +damage% aura are left in
  the calibration residual, so its 4.7 casts a fight are slightly low (5.3).
- **Rune of Unleashed Fire lingering** ticks 286 times against 247 (8s at 1s
  with `extend` refresh).
- Not modelled: movement, multi-target (Spinning Crane Kick's Mark of the
  Crane stacks), Invoke Xuen and Strike of the Windlord (not talented in this
  profile).
