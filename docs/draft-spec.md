# Drafting a spec from a profile and the game data

`tools/draft_spec.py` writes a first-draft spec (step 2 of the recipe in
`AGENTS.md`). It is a starting point for a person or an agent, not a spec that
runs: no `[player]` stats, no calibration, no engine check.

```sh
python3 tools/db2.py --build 12.1.5.70575                    # DB2 tables into target/db2
python3 tools/draft_spec.py profiles/MID2_Paladin_Retribution.simc \
    --db2 target/db2 --out /tmp/ret_draft.toml --diff specs/retribution_herald.toml
python3 tools/draft_spec.py profiles/MID2_Warrior_Fury.simc --apl-only \
    --talent slayers_dominance=1 --talent lightning_strikes=0     # rotation only, no DB2
python3 tools/test_draft_spec.py                              # tests
```

## What it does

**Rotation (needs only the profile).** The `actions=` lines are flattened:
`call_action_list` inlines the called list under the call's condition,
`run_action_list` does too and guards the entries after it with the negated
condition. Each ability entry's `if=` becomes a `[[rotation]]` `if`:

- `buff.X.up/down/remains/stack/react`, `dot.X.ticking/up/down/remains/stack`,
  `cooldown.X.ready/up/down/remains/charges/charges_fractional/duration/full_recharge_time`
  (an unqualified `full_recharge_time` becomes the entry's own cooldown),
  `gcd`, `gcd.max`, `time`, `fight_remains`, `target.time_to_die`,
  `target.health.pct`, `prev_gcd.1.X`, resources and `X.deficit` (as
  `X.max-X`), `%` as division.
- Folded to constants: `active_enemies` and `spell_targets.*` (the engine
  simulates one target, so the AoE lists drop out and are counted in the header),
  `fight_style.*` (patchwerk), `hero_tree.X`, and `talent.X` when the loadout is
  known (decoded with `--db2`, or `--talent name=0|1` by hand).
- `variable.X` is inlined when it is set once, unconditionally.
- Anything else (`trinket.*`, `race.*`, `equipped.*`, `debuff.*`, `refreshable`,
  `%%`, `<?`, `>?`, `@`, `raid_event.*`, setif variables, unknown talents) makes
  the entry a commented-out `# [[rotation]]` block with `# TODO untranslated
  (reason); [simc] list L: if=<original>`. The tool never drops part of a
  condition: a shortened condition would fire more often than the APL's.
- Actions a spec doesn't have (auto attack, racials, potions, items,
  interrupts, stances, movement) are skipped and listed in the header.

**Abilities and buffs (needs `--db2`).** Spells are found by name
(`snake_case` SimC name against `SpellName`), preferring the spec's own spells
and then spells that do something; other candidate ids are listed in a comment.
From the tables: cooldown, charges and recharge, GCD (when not 1.5s), cast
time, cost and gain by power type (rage, runic power etc. are stored in tenths),
the damage coefficient (`ap_coeff` or `sp_coeff`) and school of the first
damage effect, a periodic effect as a TODO note, and for buffs the duration,
stacks and every aura effect as a TODO comment (the modifier layer is
`tools/effects.py`). Each number is tagged `[db2 <spell id>]`, rotation
entries `[simc]`. A profile without `actions=` (Ret, Fire, and the other
profiles listed under "Gaps") gets the spec's active spells
(`SpecializationSpells` plus the loadout's talents) as its ability list and a
TODO in place of the rotation.

Not derivable, and left as TODO: `cooldown_hasted`, `mastery = true`, DoT
shape, proc chances, on_cast effects and triggers, `[player]` and `[gear]`,
`free_with`/`requires_buff`, pets. Those are the work of steps 3-6.

`--diff specs/<hand>.toml` prints a structural comparison (ability sets,
cooldown/charges/cast time/cost/gain/coefficient per common ability, buff sets,
rotation action order).

The DB2 reads are all in `Db2Source`; a different source (a cached JSON, SimC's
spell dump) can replace it without touching the APL half.

## Status: what was and was not verified

- The APL translator is tested on every `profiles/*.simc` (flattening, every
  emitted condition re-parses in the engine's grammar). Fury (Slayer) with the
  hero-tree talents set by hand gives 21 live rotation entries and no TODO;
  its order and conditions match the `slayer` list. The hand-written
  `specs/fury_slayer.toml` additionally has `target.health.pct<35` on Execute
  (the game's execute range, which the APL leaves to the spell itself) and
  `!dot.bladestorm_dot.up` guards (the engine models Bladestorm as a channel).
  Without a loadout, 481 of 535 non-pruned entries across all profiles are
  TODO, nearly all `talent.*` or `hero_tree.*`: the numbers need the DB2
  tables to improve.
- **The DB2 layer ran only on a synthetic table set** (`tools/test_draft_spec.py`
  writes made-up spells in the wago.tools column layout). wago.tools was not
  reachable from the session that wrote the tool (the egress proxy answered
  403), so `tools/db2.py --build` could not be run, the talent decoding and
  name matching were never run on real tables, and the power-type scaling (tenths for
  rage, runic power, astral power, insanity, fury, pain) is from the SimC
  convention, not checked against rows. Treat the first real run as the test.
- **The diff versus the hand-written Ret and Fire specs has not been produced.**
  It needs the DB2 tables, and neither `MID2_Paladin_Retribution.simc` nor
  `MID2_Mage_Fire.simc` has `actions=` lines, so there is no APL to translate
  for them either. Run the commands above once the tables are built and put the
  diff here.
- **No new spec was started.** Every profile in `profiles/` already has a
  registered spec in `specs/`. The first uncovered profile (a new season, or another
  hero tree) is the one to draft.

## Gaps

Profiles with no `actions=` lines: Balance, Arcane, Fire, Frost Mage, Ret,
Shadow, Subtlety, Elemental, the three Warlocks and Arms. Their rotation has to
come from SimC's `ActionPriorityLists/default/*.simc`.

## First real-table run

`tools/draft_spec.py --db2 target/db2` was run on real tables for the Blood
(Deathbringer) and Protection (Colossus) profiles, the first uncovered ones; both
specs are in `specs/` (`docs/blood-deathknight.md`, `docs/protection-warrior.md`).
The real-table layer works: talents decode, the hero tree is picked, 21 and 14
rotation entries come out, and costs, cooldowns and coefficients are right for
unambiguous spells. What a person still had to do: pick the right spell among
same-named ones (Protection's `execute` came back with Rogue-like energy cost, and
`shield_slam` a different school), read class mechanics out of the APL's
conditions (Bone Shield, Dancing Rune Weapon, Violent Outburst), set the resource
tables and everything under `[player]`, model pets and procs, and calibrate.
