# Feral Druid (Wildstalker) — draft

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

Wildstalker: Rip and Rake ticks grow Bloodseeker Vines (stacking bleed, 6s);
each stack bursts as Bursting Growth on expiry and has a 30% chance per tick
to proc Rampancy. Tiger's Fury gives the next two builders an Unseen Slash,
and Ferocious Bite has a 15% chance per combo point to do the same. Sudden
Ambush, Frantic Momentum, Savage Fury, Tiger's Tenacity and Omen of Clarity
(7 real PPM on auto attacks) are modelled from SimC's source.

## Results

| | value |
|---|---:|
| reference DPS (SimC, 300s core kit) | 169,338 |
| azsim DPS, 10,000 iterations, seed 1 | 171,069 |
| difference | +1.02% |

`derived/feral_wildstalker.toml` comes from `tools/effects.py` (Feral Druid's
+70% spec aura, Killer Instinct, Lacerating Claws, Dreadful Bleeding,
Sabertooth, Taste for Blood, Focused Frenzy, Soul of the Forest, Circle of
Life and Death, the Wildstalker label modifiers). `calibration/feral_wildstalker.toml`
fits the per-hit residual and crit offset on top; hit counts are the engine's.

Stat probes (held out, same calibration):

| probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.76% | +8.92% | +0.16 pts |
| +500 haste | +8.85% | +10.45% | +1.60 pts |
| +500 mastery | +8.01% | +8.32% | +0.31 pts |
| +500 versatility | +8.34% | +8.57% | +0.24 pts |
| +200 agility | +5.92% | +5.95% | +0.03 pts |

## Known gaps

- **Overflowing Power** (Berserk lets combo points go over the cap, up to 3,
  restored by finishers) is not modelled; the combo point cap is 5 always.
  This costs Ferocious Bite and Shred casts (53.5 and 93.3 a fight against
  59.1 and 102.9).
- **Coiled to Spring** (combo point overflow: next Ferocious Bite +10%) is
  not modelled — the engine has no "gain overflowed" hook.
- **Twin Sprouts** (30% chance to add a second Bloodseeker Vines stack) is
  folded into the vines residual rather than applied as a second stack.
- **Gear stat procs**: only Rune of Burning Haste's average haste is folded
  into `[player]`; Arcanoweave Insight, Might of the Void and the Venomcursed
  ring procs sit in the residual. There is no `[gear]` block yet, so `--gear`
  is not supported.
- **Convoke the Spirits** is a 16-tick weighted approximation of SimC's random
  cast pool (4 Ferocious Bite, 6 Shred, 1 Moonfire-or-Wrath, 3 Wrath, a rare
  Feral Frenzy, a heal). It casts 3 times a fight where the reference averages
  2.5, so the convoke rows run high; Rake from convoke (0.3 hits) is not
  modelled.
- **Unseen Slash's bleed** ticks more often than the reference (243 hits
  against 209): the dot stays up longer in this model.
- `cat_melee` and `feral_frenzy` need large residuals because the observation
  rows carry spell id 0 / the parent spell 274837, so `effects.py` cannot see
  Cat Form's +40% auto-attack, Feral Druid's +560% auto-attack or Focused
  Frenzy's +15%.
