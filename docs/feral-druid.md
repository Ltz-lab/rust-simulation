# Feral Druid (Wildstalker)

Single target, Midnight Season 2, core kit. Profile:
`profiles/MID2_Druid_Feral.simc` (hero tree Wildstalker). Spec:
`specs/feral_wildstalker.toml`. Reference: `reference/feral_*.json`
(generated with `tools/make_reference.sh feral profiles/MID2_Druid_Feral.simc agility`).

Energy (140, 12.65/s hasted) and combo points (5). Builders: Rake, Shred,
Lunar Inspiration (Moonfire in cat form). Finishers: Rip (bleed length grows
with combo points) and Ferocious Bite (all combo points, up to 25 extra
energy). Tiger's Fury (+50 energy, +21% damage, bleeds +31% for their full
duration) drives Berserk (15s, +15% damage, +1 combo point per builder, one
combo point every 1.5s) and Feral Frenzy / Convoke the Spirits.

Wildstalker: Rip and Rake ticks grow Bloodseeker Vines (a stacking bleed that
ticks every 1.6s), each stack bursts as Bursting Growth on expiry, and every
vine tick has a 30% chance to proc Rampancy. Tiger's Fury gives the next two
builders an Unseen Slash and Ferocious Bite has a 15% chance per combo point
to do the same; Convoke's Ferocious Bites proc it too. Sudden Ambush, Frantic
Momentum, Savage Fury, Tiger's Tenacity, Panther's Guile and Omen of Clarity
(a 15.2% chance per auto-attack swing) are modelled from SimC's source.

## Results

| | value |
|---|---:|
| reference DPS (SimC, 300s core kit) | 169,230 |
| azsim DPS, 10,000 iterations, seed 1 | 169,363 |
| difference | +0.08% |

`derived/feral_wildstalker.toml` comes from `tools/effects.py` (Feral Druid's
+70% spec aura, Killer Instinct, Lacerating Claws, Dreadful Bleeding,
Sabertooth, Taste for Blood, Focused Frenzy, Soul of the Forest, Circle of
Life and Death, the Wildstalker label modifiers).
`calibration/feral_wildstalker.toml` fits the per-hit residual and crit offset
on top; hit counts are the engine's.

Per-source counts (per fight, 10,000 iterations):

| source | reference | azsim |
|---|---:|---:|
| Ferocious Bite | 59.1 | 57.4 |
| Rip (ticks) | 249.2 | 251.4 |
| Unseen Slash (hits) | 208.8 | 201.1 |
| Rake (casts + ticks) | 28.0 + 164.9 | 25.0 + 168.4 |
| auto attacks | 461.9 | 491.4 |
| Shred | 102.9 | 111.7 |
| Feral Frenzy (hits) | 105.6 | 107.1 |
| Bloodseeker Vines (ticks) | 182.5 | 167.2 |
| Lunar Inspiration (casts + ticks) | 19.1 + 225.9 | 20.2 + 241.8 |
| Bursting Growth | 135.6 | 131.7 |
| Convoke: Bite / Shred / Wrath | 10.8 / 17.9 / 9.9 | 10.3 / 17.6 / 10.1 |
| Rampancy | 54.7 | 50.1 |

Stat probes (held out, same calibration):

| probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.85% | +8.85% | -0.00 pts |
| +500 haste | +9.00% | +9.18% | +0.17 pts |
| +500 mastery | +8.12% | +8.12% | -0.00 pts |
| +500 versatility | +8.57% | +8.57% | +0.01 pts |
| +200 agility | +5.94% | +5.95% | +0.01 pts |

Reference and azsim priority orders agree: haste > crit > vers > mastery >
agility.

## Gear

`[gear]` turns /simc gear lines into these stats; against SimC without
trinkets (`tools/check_gear.py`), all 13 items' item levels and every rating
match, and so do attack power (2,577), crit (26.89%), haste (26.57%),
mastery (74.74%) and versatility (7.98%). DPS from `--gear` is 169,396
against 169,379 from the hand-entered stats (+0.01%, same seed; SimC
169,196). Weapon attack power comes from the main hand's DPS (1,112 at
155.8), fitted to the agility probe as for Subtlety rather than SimC's plain
6 x DPS.

`[gear.kit]` (gear lines only) adds the sample profile's raid-night
consumables as flat stats — Harandar Celebration, Void-Touched augment rune,
Flask of the Magisters and Thalassian Phoenix Oil — plus Bloodlust and the
Liquid Luster potion. The potion's Lustrous Gleam stacks (220.82 versatility
rating each, ~3 average over its 30s) are folded into one versatility buff.
`--set gear.kit.enabled=0` switches it off for core-kit comparisons.

## Known gaps

- **Overflowing Power** (Berserk lets combo points go over the cap, restored
  by finishers) is not modelled; the combo point cap is 5 always.
- **Coiled to Spring** (combo point overflow: next Ferocious Bite +10%) is
  not modelled — the engine has no "gain overflowed" hook.
- **Twin Sprouts** (30% chance to add a second Bloodseeker Vines stack) is
  folded into the vines' grow rate (13.5%/10% per Rake/Rip tick from Thriving
  Growth, x1.2 Green Thumb, x1.3 Twin Sprouts) rather than applied as a
  second stack, so the vines' stack count and Bursting Growth line up but a
  doubled application shares one timer in SimC and not here.
- **Bloodseeker Vines** is a per-stack DoT here; SimC refreshes one dot and
  keeps the stack timers separate, which keeps its bleed up 95.8% of the
  fight. With a 7.8s effective duration this model gets 167 ticks against
  182.5 and 8% less vines damage, which the calibration does not hide (the
  rows run -8.5%).
- **Gear stat procs**: Rune of Burning Haste, Arcanoweave Insight, Might of
  the Void and the Venomcursed ring are not modelled, so the in-fight haste
  from Savage Fury and Frantic Momentum is more than SimC applies to its
  auto-attacks in practice: `cat_melee` 491 against 462 and Shred 112 against
  103. `[gear]` carries the plain buffed snapshot, so those procs stay in the
  calibration residual there too.
- **Liquid Luster** (the sample profile's potion) is one flat versatility
  buff, 12.27% for 30s: the stacking Gleam ramp (up to 5 stacks, 15s) is
  averaged rather than stacked.
- **Convoke the Spirits** is a 32-slot weighted approximation of SimC's cast
  pool (7 Bite, 12 Shred, 6 Wrath, 2 Moonfire-or-Wrath, 4 heals, a 40%
  Feral Frenzy). Rake from convoke (0.3 hits) is not modelled.
- **Shadowmeld** is cast on cooldown-ish (3 times a fight, matching the
  reference) but this model's Rake refresh already covers those casts, so it
  does not add the 3 extra Rakes the reference's APL gets (25 against 28).
- **Unseen Slash from a Ferocious Bite** is not scaled by the combo points and
  excess energy of the Bite that proc'd it (SimC scales it by `cp/5 x
  energy`); the average lands in the residual.
- `cat_melee` and `feral_frenzy` need large residuals because the observation
  rows carry spell id 0 / the parent spell 274837, so `effects.py` cannot see
  Cat Form's +40% auto-attack, Feral Druid's +560% auto-attack or Focused
  Frenzy's +15%.
