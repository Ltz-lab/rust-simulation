# Survival Hunter (Sentinel)

`specs/survival_sentinel.toml`, built the same way as the others: coefficients,
costs, cooldowns, durations and talent values from DB2, SimC's hunter module
(`sc_hunter.cpp`) and default APL where a mechanic is scripted, and the rest
observed in SimC's log. Survival is the second spec on a pooling resource
(focus, 125) and the first with a pet that is not a summon: the duck is out
for the whole fight, its Claw runs on a 3s cooldown and its focus, and the
hunter's own actions fire the pet's Kill Command, Takedown, Strike as One and
Sic 'Em. Every mechanic below is generic engine data, not hunter code.

## Results

Reference: SimC's Midnight S2 sample Survival profile (Sentinel), core kit
only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 163,198 |
| azsim | 158,655 (±126) |
| difference | **-2.78%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| raptor_swipe | 27,674 | 27,384 | -1.0% | 56.3 | 55.7 |
| raptor_strike | 23,208 | 22,990 | -0.9% | 56.8 | 56.2 |
| wildfire_bomb_damage | 21,760 | 20,259 | -6.9% | 43.7 | 40.8 |
| duck_strike_as_one | 14,748 | 14,367 | -2.6% | 216.5 | 210.9 |
| lunar_storm | 10,795 | 8,709 | -19.3% | 133.2 | 107.7 |
| auto_attack_mh | 10,212 | 10,523 | +3.0% | 122.7 | 126.5 |
| duck_kill_command | 9,953 | 9,722 | -2.3% | 76.5 | 74.7 |
| wildfire_bomb_bleed | 8,224 | 8,224 | +0.0% | 220.8 | 220.8 |
| moonlight_chakram_damage | 7,849 | 7,870 | +0.3% | 25.4 | 25.5 |
| sanctified_armaments | 7,494 | 7,530 | +0.5% | 282.1 | 283.7 |
| boomstick_damage | 6,777 | 6,779 | +0.0% | 27.6 | 27.6 |
| duck_sic_em | 2,886 | 2,675 | -7.3% | 303.6 | 281.6 |
| duck_claw | 1,895 | 1,942 | +2.5% | 95.3 | 97.8 |
| takedown_damage | 1,788 | 1,791 | +0.2% | 3.7 | 3.7 |
| duck_melee | 1,630 | 1,594 | -2.2% | 193.1 | 189.2 |
| duck_takedown | 1,547 | 1,541 | -0.4% | 3.7 | 3.7 |
| venomfang | 1,100 | 1,177 | +7.0% | 158.6 | 170.0 |
| rune_of_unleashed_fire | 1,033 | 1,033 | -0.0% | 50.6 | 50.6 |
| void_eruption | 764 | 762 | -0.4% | 29.0 | 28.9 |
| acidbrood_scourge | 759 | 754 | -0.7% | 21.6 | 21.5 |
| venomfang_burst | 476 | 474 | -0.5% | 35.4 | 35.3 |
| twilight_requiem | 325 | 327 | +0.6% | 3.6 | 3.6 |
| rune_of_unleashed_fire_lingering | 311 | 270 | -13.1% | 243.7 | 211.5 |

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +11.34% | +10.06% | -1.27 pts |
| +500 haste | +9.24% | +7.10% | -2.13 pts |
| +500 mastery | +10.30% | +9.03% | -1.27 pts |
| +500 vers | +8.76% | +8.82% | +0.06 pts |
| +200 agility | +6.12% | +7.80% | +1.68 pts |

The engine reproduces the reference's stat priority order except agility,
which it values a little too highly: auto attacks and the pet are fully
attack-power-driven here, where SimC's melee swings are mostly weapon damage
and the pet's own attack power only partly tracks the hunter's.

## What is modelled

- **Focus** (125, 5 a hasted second). Kill Command pays 20 and is free; Raptor
  Strike/Swipe 30, Wildfire Bomb 10, Boomstick 50. Takedown pays 50.
- **Tip of the Spear** — 3 stacks, 10s, +24% direct damage; Kill Command grants
  two (Primal Surge), Takedown three (Twin Fangs). Raptor Strike, Raptor Swipe,
  Wildfire Bomb, Boomstick, Takedown and Moonlight Chakram each consume one
  stack, and each consumption provokes the pet's Strike as One (twice on a
  Raptor Swipe) and rolls for Sentinel's Mark.
- **Sentinel** — Sentinel's Mark (+90% damage taken: 40 from the debuff plus
  50 from Arcane Talons, 16s) applied on 10% of Tip consumptions, 20% while
  Takedown is up, and forced by Takedown; a Wildfire Bomb on a marked target
  consumes it for four Lunar Storm missiles 150ms apart; every Mark takes 6s
  off Wildfire Bomb (Moon's Blessing). Moonlight Chakram after Takedown, the
  bounces 640ms apart, with Twilight Requiem on the last one. Sanctified
  Armaments banks 15% of every Raptor Strike/Swipe hit into a 6s arcane DoT.
- **Mongoose Fury** (+10% Raptor Strike damage a stack, 20 stacks, 8s) from
  every Raptor Strike/Swipe and every Boomstick tick (Mongoose Rounds), with
  Wallop (+50% to the next Raptor Strike, 20% chance).
- **Wildfire Bomb** — 2 charges (Guerrilla Tactics), 18s hasted recharge, a
  fire explosion and Shrapnel Bomb's 6-tick physical bleed, and 5 focus a
  second for 3s back.
- **Boomstick** — 45s (60s less Quick Reload), a hasted 4-tick channel.
- **Takedown** — 90s, 12 AP plus the pet's 6 AP, +20% direct, periodic and pet
  damage for 8s.
- **The pet** — 2s melee, Claw (3s, 25 focus, 100 max, 7.85 focus a second),
  Kill Command (3 AP), Takedown (6 AP), Strike as One (1.8 AP) and Sic 'Em
  (0.08 AP a tick for 5s on a Strike as One crit).
- Gear and group procs as Poisson rates: Venomfang (a 6s bleed and its burst),
  Rune of Unleashed Fire (and the Rune of Lingering bleed), Void Eruption,
  Acidbrood Scourge, and Skyfury on the auto attack.

## Known gaps

- **Lunar Storm is 19% light.** Marks are applied at the reference's rate
  (about 36 a fight) but only 27 of them are spent on a Wildfire Bomb: this
  rotation casts Wildfire Bomb on its recharge a little more often than SimC
  does, so fewer charges are left when a Mark arrives.
- **Wildfire Bomb is 7% light** for the same reason (40.8 casts against 43.7).
- **Haste reads 2.1 points low.** SimC's pet regains focus faster with haste
  and takes the +25% Wild Hunt Claw more often; here Claw is a single
  ability.
- **Mastery** is modelled as a 1.766x coefficient on the reported 31.82%:
  the reference lifts every hit 10.4% for +500 mastery rating, which a plain
  `1 + mastery` cannot produce (it would need a factor of 0.88).
- The tier set, trinkets and consumables are not modelled (core kit only), and
  there is no `[gear]` block yet.
