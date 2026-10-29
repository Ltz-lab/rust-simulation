# Talents and auras from game data

Calibration fits one residual multiplier per damage source. Before this layer,
that multiplier had to stand in for every talent, spec aura and buff that
changes a spell's damage, so it ran to 2-4× on some sources. Most of those
modifiers are plain data, and `tools/effects.py` now reads them:

1. **Talents** come from the loadout string (the in-game export, SimC's
   `talents=`), decoded against the DB2 trait tables by `tools/talents.py`:
   the class tree's nodes in id order, selected/purchased/ranks/choice bits,
   hero talents filtered to the chosen hero tree.
2. **Passives** in effect are the talents, the spec's own spells and its
   mastery, keeping those with the passive attribute.
3. **Spell modifiers** on those passives that scale damage: aura 107/108
   (flat/percent, matched by spell family and class mask), 218/219 (matched by
   spell label), on the direct-damage, periodic-damage and effect-points ops,
   plus aura 79 (percent damage done to a school).
4. Each damage source gets the product of the percent modifiers that hit the
   spell that dealt it. The spell id comes from the observations, as a combat
   log records it; the spec's `spell_id` is only a fallback.
5. **Buffs the spec models** get their own modifiers from their buff spell,
   applied only while they're up: Avenging Wrath's +20% to every Ret spender,
   Heat Shimmer's +100% Scorch damage. Buffs whose damage the spec already
   sets by hand are left alone.

`calibrate.py --derived` writes these into the calibration file (`derived` per
source, `[buffs.*]` patches) and fits the residual `mult` on top. How much of
the old multiplier is now explained (damage-weighted mean of |log residual|,
as a factor):

| spec | residual before | residual after | damage with residual beyond ±25% |
|---|---:|---:|---|
| Retribution | ×2.10 | ×1.38 | 95% → 60% |
| Fire | ×1.46 | ×1.18 | 70% → 10% |
| Destruction | ×1.71 | ×1.18 | 75% → 4% |

DPS and the stat probes are unchanged within 0.15 points, except Ret haste,
which moves from +0.74 to +0.86: with Avenging Wrath's +20% damage now applied
inside its window rather than averaged over the fight, the window weighs more,
and the Hammer of Wrath chain inside it is where the haste over-response sits.

The tool also flags coefficients in the spec that disagree with the game data
for the spell that dealt the damage. Wake of Ashes used 1.743 (the old
artifact spell 205273), not the 3.2186 of the logged spell 255937. It now uses
255937's coefficient, and its residual dropped from 1.996 to 1.081. Per-hit
damage is the same either way (3.479 AP), so DPS and the stat probes didn't
move.

What stays in the residual, by source: scripted effects the data only names
(aura 4 dummies: Blessing of Dawn's +30% spender bonus, Light Within), target
debuffs (Judgment's +20% from spenders, aura 271), buffs and procs the spec
doesn't model.

```sh
python3 tools/db2.py --build 12.1.5.70575      # fetch the DB2 tables into target/db2
python3 tools/talents.py target/db2 CYEAAAAAAAAAAAAAAAAAAAAAAAAAAAANbbzMz...
python3 tools/effects.py target/db2 specs/retribution_herald.toml reference/ret_core_observations.json \
    profiles/MID2_Paladin_Retribution.simc --out derived/retribution_herald.toml --explain final_verdict
python3 tools/calibrate.py specs/retribution_herald.toml reference/ret_core_observations.json \
    calibration/retribution_herald.toml --derived derived/retribution_herald.toml
```
