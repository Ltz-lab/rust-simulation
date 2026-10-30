# Roadmap

## Not yet built

Effects of gear other than trinkets and set bonuses (embellished rings such
as Loa Worshiper's Band, armor and weapon procs, proc enchants), multi-target, movement, healers and tanks, a real Warcraft Logs ingestion pipeline. The DB2 effect layer covers passive and buff spell modifiers; scripted auras, target debuffs and talent rank curves (TraitDefinitionEffectPoints) are next.

Trinket effects and set bonuses are in (see [gear.md](gear.md#item-effects-and-set-bonuses)):
an effect library of spec fragments keyed by spell id, numbers from DB2 at
the item's level. It covers six Midnight trinket effects, Bite of Zul'jan and
Ret's MID2 tier set; every other trinket or set bonus is reported as
unmodelled, and the app sends swaps touching one to SimC.

## Next steps

1. Grow the effect library: the season's remaining trinkets and the other
   specs' tier sets, then the non-trinket item effects (rings, armor,
   weapons, enchants) with the same templates, so Top Gear values them.

2. Check the fitted and `[obs]` mechanics against SimC's class modules,
   starting with the haste over-response (Ret's Expurgation and Hammer of
   Wrath chain, Fire's Fire Blast recharge), and tag what comes from there
   `[simc]`.
3. Extend the DB2 effect layer: run the new target-debuff (aura 271) and rank-curve
   code against built DB2 tables, check the numbers, regenerate `derived/` and
   calibration, then move hand-set buff values (`damage_pct` etc.) over to derived ones.
4. Warcraft Logs ingestion: the same `observe → calibrate → validate` loop on
   real logs instead of SimC output.
5. A rotation optimizer (search over priority lists with the engine as the judge).
6. Top Gear: sim many gear sets per run (the item data loads once), with
   paired seeds so the differences between sets carry little noise.
7. More specs, ideally generated as drafts by an AI from game data, tooltips
   and SimC's class modules, then verified by the validation loop above.
