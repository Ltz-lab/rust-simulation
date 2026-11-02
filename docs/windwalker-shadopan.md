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

## Gear

`[gear]`, `[gear.kit]` and `[gear.omnium]` let the spec take `--gear profile.simc`
(the layer is in [gear.md](gear.md)). Measured from SimC on
`profiles/MID2_Monk_Windwalker.simc` without trinkets and tier set:

| constant | value | how |
|---|---:|---|
| base agility | 622 | SimC `gear_agility=0` gives 679 = 622 x 1.092 |
| agility multiplier | 1.092 | `gear_agility` +1000 gives +1092 |
| attack power multiplier | 1.05 | 2555 agility -> 2682 AP (Battle Shout; 2555 AP with `optimal_raid=0`) |
| crit bonus | +14% | 27.48% at 620 rating = 13.48% + 14% |
| haste multiplier | x1.05 | 29.44% at 1,024 rating (23.27%) |
| mastery | 10 points x 2.32875 | 83.13% unbuffed, 87.78% buffed at 1,274 rating (2 points from Skyfury) |
| versatility bonus | +3% | Mark of the Wild |
| weapon attack power | 6.8809 per DPS | 1,072 at 155.80 two-hander DPS |
| spec id | 269 | ChrSpecialization Windwalker |

Kit (SimC buffed stats with and without each consumable): Harandar Celebration
+54 agility, Void-Touched augment rune +27, Flask of the Blood Knights +165
haste, Thalassian Phoenix Oil +16 crit and +16 haste (the two-hander takes one),
together +82 agility (+86 attack power), +181 haste, +16 crit. Bloodlust at the
pull. Potion of Recklessness (+1,725 mastery, the highest rating, and -232
versatility, the lowest) is used at the pull, as the APL does for Flurry Strikes
without Xuen (`potion,if=talent.flurry_strikes&chi>2&time<5`). Omnium traits
(20,000 iterations, core kit): 169,345 with all five, 164,187 with none (+3.14%);
Unleashed Fire 1.01651, Self-Mending 1.00015, Lingering 1.00120, Critical Power
1.01150, Overload 1.00179.

Against SimC (`tools/check_gear.py`, no trinkets), every item level and rating
matches, and buffed stats match to rounding (with kit: attack power 2,768, crit
27.83%, haste 33.76%, mastery 87.78%, versatility 5.28%). DPS, 300s Patchwerk,
SimC 5,000 iterations and azsim 5,000, seed 42, kit off: 169,360 vs 167,114
(-1.33%, the spec's own calibration gap; hand-entered stats give 167,122).
With the kit on (SimC 20,000 iterations, azsim 10,000, seed 1): SimC 192,139
vs azsim 187,757 (-2.28%). Split out, consumables only 178,138 vs 176,775
(-0.77%), consumables plus Bloodlust 184,678 vs 182,984 (-0.92%), consumables
plus potion 183,055 vs 180,831 (-1.21%). The extra 0.95 points over the core-kit gap sit in how
the rotation responds to Bloodlust and the potion together (SimC times Zenith
off them; the spec casts it on cooldown). Mastery above about 1,500 rating also
reads lower in azsim than SimC (2,999 rating: 153.97% vs 156.30%; 4,000: 179.50%
vs 185.09%), which touches only the potion window.

Both trinkets (Voracious Heart of Ula'tek, Zul'jin's Guillotine Technique) are
in the effect library.

## Tier set (Midnight season 2, set 2061)

`[[gear.effects]]` at the end of the spec, read from SimC's monk module
(`fists_of_fury_t`, `rising_sun_kick_t::base_damage_t`,
`spinning_crane_kick_t::tick_t`, `buff.mid2_ww_4pc`) and DB2 (spells 1296621,
1296624, 1297033, 451297):

- **2pc (1296621):** every Fists of Fury cast also lands one extra tick at 50% of
  a tick's damage. It reports under `fists_of_fury`, so it takes the tick's
  calibration. The extra tick lands before any Momentum Boost stack exists and
  adds one itself, so the five real ticks get +10% more each (a 1.3 / 1.2 lift
  on the calibrated average ramp, +8.33%); both are in the fragment.
- **4pc (1296624):** Unbroken Rhythm, a stack (20s, 6 at most) for each Fists of
  Fury tick, the five of the channel and the 2pc's extra one. Each stack is +10%
  on Rising Sun Kick and Rushing Wind Kick and +20% on Spinning Crane Kick's ticks;
  either kick spends every stack. A new generic `[[patch]]` form
  (`dot = { on_tick = [..] }`, `channel = { on_finish = [..] }`) hangs the stack on
  the Fists of Fury tick and the spend on the crane kick's end.

Full profile against the same profile with the set off, SimC and azsim (SimC
`set_bonus=midnight_season_2_{2,4}pc=0` and `bite_of_zuljan_2pc=0`, Patchwerk 300s,
seed 1, target error 0.1%; azsim `--set gear.set_bonuses=0`, 20,000 iterations, seed 1):

| | SimC DPS | azsim DPS | SimC gain | azsim gain |
|---|---:|---:|---:|---:|
| tier off | 235,325 | 225,173 | | |
| 2pc only | 243,622 | 233,793 | +3.53% | +3.83% |
| 2pc + 4pc (full profile) | 257,957 | 247,817 | **+9.62%** | **+10.06%** |

The full profile is -3.9% against SimC, the tier-off one -4.3% (the spec's own gap
plus the trinkets and kit). Per source, DPS the set adds (SimC 4,000 iterations, azsim
20,000): Fists of Fury plus the 2pc's extra tick +8,094 (4,559 of it the extra tick) vs
+8,607, Spinning Crane Kick +4,777 vs +4,592, Rising Sun Kick +5,916 vs +8,032 and Rushing
Wind Kick +3,502 vs +1,378. The three kicks sum to about the same (14,195 vs 14,002), but
the split differs: this spec casts Rushing Wind Kick ahead of Fists of Fury as soon as its
buff is up, so it rarely sees stacks, where SimC's list casts it after them.

- The rotation is unchanged. SimC's list prefers a Dance of Chi-Ji crane kick while
  Unbroken Rhythm is up; this spec already casts that one before Rising Sun Kick.
- SimC snapshots a crane kick's stacks at the cast; here its ticks read the buff as
  they fire and the end of the channel spends it (nothing else can add or spend stacks
  inside a channel).
- Rising Sun Kick's Glory of the Dawn strike is not boosted: SimC's kick spends the
  buff before it fires.
