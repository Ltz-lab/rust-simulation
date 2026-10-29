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
| azsim | 163,103 (±127) |
| difference | **-0.06%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| raptor_swipe | 27,674 | 27,820 | +0.5% | 56.3 | 56.6 |
| raptor_strike | 23,208 | 23,303 | +0.4% | 56.8 | 57.1 |
| wildfire_bomb_damage | 21,760 | 21,694 | -0.3% | 43.7 | 43.7 |
| duck_strike_as_one | 14,748 | 14,629 | -0.8% | 216.5 | 214.9 |
| lunar_storm | 10,795 | 10,684 | -1.0% | 133.2 | 131.9 |
| auto_attack_mh | 10,212 | 10,215 | +0.0% | 122.7 | 122.7 |
| duck_kill_command | 9,953 | 9,881 | -0.7% | 76.5 | 76.0 |
| wildfire_bomb_bleed | 8,224 | 8,362 | +1.7% | 220.8 | 224.6 |
| moonlight_chakram_damage | 7,849 | 7,918 | +0.9% | 25.4 | 25.6 |
| sanctified_armaments | 7,494 | 7,525 | +0.4% | 282.1 | 283.5 |
| boomstick_damage | 6,777 | 6,733 | -0.6% | 27.6 | 27.5 |
| duck_sic_em | 2,886 | 2,587 | -10.3% | 303.6 | 272.4 |
| duck_claw | 1,895 | 1,943 | +2.5% | 95.3 | 97.8 |
| takedown_damage | 1,788 | 1,797 | +0.5% | 3.7 | 3.7 |
| duck_melee | 1,630 | 1,624 | -0.3% | 193.1 | 192.6 |
| duck_takedown | 1,547 | 1,556 | +0.6% | 3.7 | 3.7 |
| venomfang | 1,100 | 1,176 | +6.9% | 158.6 | 169.7 |
| rune_of_unleashed_fire | 1,033 | 1,033 | -0.0% | 50.6 | 50.6 |
| void_eruption | 764 | 761 | -0.5% | 29.0 | 28.9 |
| acidbrood_scourge | 759 | 752 | -0.9% | 21.6 | 21.5 |
| venomfang_burst | 476 | 473 | -0.6% | 35.4 | 35.2 |
| twilight_requiem | 325 | 328 | +0.9% | 3.6 | 3.6 |
| rune_of_unleashed_fire_lingering | 311 | 309 | -0.7% | 243.7 | 242.0 |

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +11.34% | +10.04% | -1.29 pts |
| +500 haste | +9.24% | +7.98% | -1.26 pts |
| +500 mastery | +10.30% | +9.03% | -1.27 pts |
| +500 vers | +8.76% | +8.82% | +0.06 pts |
| +200 agility | +6.12% | +7.83% | +1.71 pts |

The engine reproduces the reference's stat priority order except agility,
which it values a little too highly (pure attack power here, where SimC's
melee abilities carry a fixed weapon-damage share: the probe lifts attack
power 8.5% for a 6.1% DPS gain), and haste, which reads 1.3 points low, so
versatility edges above it.

## What is modelled

- **Focus** (125, 5 a hasted second). Kill Command pays 20 and is free; Raptor
  Strike/Swipe 30, Wildfire Bomb 10, Boomstick 50. Takedown pays 50.
- **Tip of the Spear** — 3 stacks, 10s, +24% direct damage; Kill Command grants
  two (Primal Surge), Takedown three (Twin Fangs). Raptor Strike, Raptor Swipe,
  Wildfire Bomb, Boomstick, Takedown and Moonlight Chakram each consume one
  stack, and each consumption provokes the pet's Strike as One (twice on a
  Raptor Swipe, and each strike fires the pet's Sic 'Em on a crit) and rolls
  for Sentinel's Mark.
- **Sentinel** — Sentinel's Mark (+90% damage taken: 40 from the debuff plus
  50 from Arcane Talons, 16s) on 20% of Tip consumptions (Sentinel's 10 plus
  Moon's Blessing's 10), 30% while Takedown is up (Lunar Calling +10), and
  forced by Takedown; a Wildfire Bomb on a marked target consumes it for four
  Lunar Storm missiles 150ms apart; every Mark takes 6s off Wildfire Bomb
  (Moon's Blessing). Moonlight Chakram after Takedown, the bounces 640ms
  apart, with Twilight Requiem on the last one. Sanctified Armaments banks 15%
  of every Raptor Strike/Swipe hit into a 6s arcane DoT.
- **Mongoose Fury** — an asynchronous aura: every Raptor Strike/Swipe and
  every Boomstick tick adds a stack with its own 8s timer (+10% Raptor Strike
  damage a stack, 20 stacks), and every stack gained rolls Wallop (+50% to the
  next Raptor Strike, 20%).
- **Wildfire Bomb** — 2 charges (Guerrilla Tactics), 18s hasted recharge, a
  fire explosion and Shrapnel Bomb's 6-tick physical bleed, and 5 focus a
  second for 3s back.
- **Boomstick** — 45s (60s less Quick Reload), a hasted 4-tick channel that
  stacks Mongoose Fury (Mongoose Rounds) and suppresses the swings that come
  due while it runs (`interrupt_auto_attack`).
- **Takedown** — 90s, 12 AP plus the pet's 6 AP, +20% direct, periodic and pet
  damage for 8s.
- **The pet** — 2s melee, Claw (3s, 25 focus, 100 max, 7.85 focus a second),
  Kill Command (3 AP), Takedown (6 AP), Strike as One (1.8 AP) and Sic 'Em
  (0.08 AP a tick for 5s, hasted, on a Strike as One crit).
- Gear and group procs as hasted RPPM where SimC hastens them: Venomfang (a 6s
  bleed and its burst), Rune of Unleashed Fire (unhasted) with its 8s Rune of
  Lingering bleed, Void Eruption, Acidbrood Scourge, and Skyfury on the auto
  attack. Every core-rune activation also grants Rune of Burning Haste (44.2
  haste rating a stack, 10s, 20 stacks, overlapping).

## Known gaps

- **Sic 'Em is 10% light** (272 ticks against 304): the bleed's pandemic
  refresh does not keep as much uptime as SimC's rolling periodic.
- **Lunar Storm is 1% light**: a few marks are refreshed before a Wildfire
  Bomb can spend them.
- **Venomfang ticks 7% hot**; its asynchronous aura ticks slightly faster
  than the reference's.
- **Agility reads 1.7 points high**: ability damage here is pure attack
  power, where SimC's melee abilities carry a fixed weapon-damage share.
- **Mastery** is modelled as a 1.766x coefficient on the reported 31.82%:
  the reference lifts every hit 10.4% for +500 mastery rating, which a plain
  `1 + mastery` cannot produce (it would need a factor of 0.88).
- The tier set, trinkets and consumables are not modelled (core kit only), and
  there is no `[gear]` block yet.
