# Drafting a spec from a profile and the game data

`tools/draft_spec.py` writes a first-draft spec (step 2 of the recipe in
`AGENTS.md`). It is a starting point for a person or an agent, not a spec that
runs: no `[player]` stats, no calibration, no engine check.

```sh
python3 tools/db2.py --build 12.1.5.70575                    # DB2 tables into target/db2
python3 tools/draft_spec.py profiles/MID2_Paladin_Retribution.simc \
    --db2 target/db2 --simc $SIMC_DIR --bare --out /tmp/ret_draft.toml --diff specs/retribution_herald.toml
python3 tools/draft_spec.py profiles/MID2_Warrior_Fury.simc --apl-only \
    --talent slayers_dominance=1 --talent lightning_strikes=0     # rotation only, no DB2
python3 tools/test_draft_spec.py                              # tests
```

`--simc DIR` (or `SIMC_DIR`) points at a SimC checkout: profiles with no `actions=`
lines (Ret, Fire and most casters; the profiles in `profiles/` are SimC's sample
profiles) take their rotation from `ActionPriorityLists/default/<class>_<spec>.simc`;
`--apl FILE` names one directly. `--bare` assumes no trinkets or equipped items
(`equipped.*` and `trinket.*` fold to 0), as in the core-kit references.
`--fight-length` (default 300) is `expected_combat_length`.

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
- **The diff versus the hand-written Ret and Fire specs** is below, run on the
  built tables (build 12.1.5.70575) and SimC's default APLs.
- **No new spec was started.** Every profile in `profiles/` already has a
  registered spec in `specs/`. The first uncovered profile (a new season, or another
  hero tree) is the one to draft.

## Gaps

Profiles with no `actions=` lines: Balance, Arcane, Fire, Frost Mage, Ret,
Shadow, Subtlety, Elemental, the three Warlocks and Arms. Their rotation comes
from SimC's `ActionPriorityLists/default/*.simc` with `--simc`.

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

## Draft versus the hand-written Ret and Fire specs

`draft_spec.py ... --diff` on both, before and after the fixes this comparison
revealed. "Before" is the tool as first merged, with the real tables (it had
only ever run on synthetic ones) and no APL for either profile.

| | Ret before | Ret after | Fire before | Fire after |
|---|---|---|---|---|
| rotation entries (live / TODO) | 0 / 0 | 19 / 0 | 0 / 0 | 0 / 66 |
| hand-spec abilities whose spell the draft picked (of those with a `spell_id`) | 8 of 28 | 20 of 28 | 4 of 14 | 13 of 14 |
| abilities named alike in both | 7 | 9 | 5 | 4 |
| per-field differences listed (cooldown, charges, cast time, cost, gain, coefficient, school) | 14 | 11 | 12 | 4 |

What the diff showed, and what was changed:

1. **Wrong spell picked among same-named ones.** Ret's `avenging_wrath` came back
   as 384376, `final_verdict` as 1251812, `blade_of_justice` as 404358, so
   coefficients and costs were those of procs and overrides. The ranking now puts
   castable (non-passive) spells that do something first, then ones the loadout's
   talents grant, then the spec's own spells, then the oldest id. Of the Ret hand
   spec's 28 spell ids, 20 now match (the rest are hit sub-spells that never
   appear in an APL: Seething Flames, Light Within, Divine Storm's Tempest
   copies); Fire 13 of 14 (Meteor's impact).
2. **Cooldowns of one-charge spells were dropped.** Blade of Justice, Wake of
   Ashes and Fire Blast keep their recovery in the charge category
   (`MaxCharges` 1, `ChargeRecoveryTime`), with `RecoveryTime` only a lockout
   between casts (Fire Blast showed 0.5s). Now read.
3. **Multi-school spells** (Wake of Ashes, school mask 6) came out `holy` with a
   TODO; the mask now maps to SimC's names (`holyfire`, `shadowflame`, ...).
4. **No rotation for profiles without `actions=`**: `--simc` / `--apl` (above).
5. **APL idioms that made whole entries TODO**: `raid_event.*` (nothing exists on
   one Patchwerk target; `.in` never comes), `variable ... op=reset,default=` (a
   variable that is only reset is its default; Ret's `pre_es_wake` blocked
   Execution Sentence, Wake of Ashes and Divine Toll), `%%` (modulo) on constants
   (Fire's `expected_combat_length%%60`), `equipped.*`/`trinket.*` with `--bare`
   (blocked Avenging Wrath), `expected_combat_length`.
6. **Action names the APL spells differently from the spell**: `templars_verdict`
   is Final Verdict when the loadout has it; actions of talents the loadout
   doesn't have (a bare `hammer_of_light`, `templar_strike`) are left out
   instead of drafted. The header now lists untranslated entries by reason.

What is still different, and why it is not a tool bug:

- **Ret.** Cooldowns/charges that talents change (Avenging Wrath 120 vs hand 60,
  Judgment 11s/1 charge vs 9.5s/2, Hammer of Wrath charges, Blade of Justice
  2 Holy Power vs 1) are talent effects the tool doesn't apply yet; the hand
  values are tagged `[obs]`. Judgment's coefficient is spell-power (1.00739) in
  the data while the hand spec has `ap_coeff 1.256` (`sp x 1.247`, the AP/SP
  conversion). Divine Storm: the draft is the hard cast with a 3 Holy Power cost,
  the hand spec's `divine_storm` is Empyrean Legacy's free one. The hand spec's
  other 30 abilities are triggered sub-hits, DoT appliers and gear/procs that
  are not in any APL. The 11 hand rotation entries are a paraphrase; the draft's
  19 follow the APL, and its Divine Storm and Final Verdict conditions match the
  hand spec's (`buff.divine_arbiter_verdict.up&!buff.empyrean_legacy.up&...`).
  Missing from the draft's buffs: the 9 the hand spec has from gear, consumables
  and mechanics (`dawnlight`, `divine_resonance`, Bloodlust ...).
- **Fire.** All 66 rotation entries stay TODO. The APL's conditions use
  `action.<spell>.executing` / `.execute_remains` / `.cast_time` / `.in_flight`
  / `.last_used`, `scorch_execute.active` and `cooldown_react`, which the
  engine's condition language has no counterpart for (it casts spells
  atomically and fires Fire Blast between casts). Folding them would change
  what the rotation does, so they are listed rather than guessed; this is the
  concrete list of engine features a Fire-like caster rotation needs. Buffs the
  draft lacks are the hand spec's mechanics (Combustion's value, execute,
  Molten Fury ...). Fire Blast's 3 charges, 11s and Meteor's 30s are talent
  effects, as above.

Also still open from the issue: "one new spec started from a draft" was met
earlier by Blood and Protection (`docs/blood-deathknight.md`,
`docs/protection-warrior.md`); every profile in `profiles/` now has a spec.
