# Roadmap

## Not yet built

Trinkets and tier sets, multi-target, movement, healers and tanks, a real Warcraft Logs ingestion pipeline. The DB2 effect layer covers passive and buff spell modifiers; scripted auras, target debuffs and talent rank curves (TraitDefinitionEffectPoints) are next.

## Next steps

1. Check the fitted and `[obs]` mechanics against SimC's class modules,
   starting with the haste over-response (Ret's Expurgation and Hammer of
   Wrath chain, Fire's Fire Blast recharge), and tag what comes from there
   `[simc]`.
2. Extend the DB2 effect layer: target debuffs (aura 271) on the debuffs the
   spec applies, talent rank curves, and the coefficient mismatches it flags;
   then move hand-set buff values (`damage_pct` etc.) over to derived ones.
3. Warcraft Logs ingestion: the same `observe → calibrate → validate` loop on
   real logs instead of SimC output.
4. A rotation optimizer (search over priority lists with the engine as the judge).
5. Top Gear: sim many gear sets per run (the item data loads once), with
   paired seeds so the differences between sets carry little noise.
6. More specs, ideally generated as drafts by an AI from game data, tooltips
   and SimC's class modules, then verified by the validation loop above.
