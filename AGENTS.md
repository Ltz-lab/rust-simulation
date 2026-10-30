# Working on azsim (for AI agents)

azsim is a data-driven WoW combat simulator in Rust. The engine has no class
code: a spec is a TOML file, and its numbers come from game data (DB2),
observed behaviour (combat logs, with SimC output as a stand-in) and SimC's
source. Read `README.md`, then the docs this file points to.

## Picking up work

Open tasks are GitHub issues labelled **`agent-ready`**. To claim one:

1. Comment "claiming" on the issue and add the `in-progress` label.
2. Work on a branch: `git checkout -b agent/<issue-number>-<slug> origin/main`.
3. When it's done and verified (see below), rebase on `origin/main`, rerun the
   checks, and push straight to `main`. This project doesn't use PR review
   yet. Close the issue with a comment giving the commit hash and the result
   numbers the issue asks for.

If you get stuck or a check can't pass, comment on the issue with what you
found, remove `in-progress`, and push your branch (don't merge it).

## Rules

- **No class-specific engine code.** If a spec needs a mechanic the engine
  lacks, add it generically in `src/` (see how pets, banked DoTs and
  `overlap` buffs are done) and document it in `docs/spec-format.md`.
- **Existing specs must not move.** Every spec in `specs/` must give
  bit-for-bit identical DPS for the same seed before and after an engine
  change. Check it explicitly:
  `./target/release/azsim specs/<s>.toml -n 2000 --seed 1 --calib calibration/<s>.toml`
  for every spec, before and after.
- **Tag every number** in a spec: `[db2]` (game tables), `[obs]` (observed
  in a log or SimC report), `[simc]` (read from SimC's class module or APL).
- **Calibration fits damage per hit only**, never hit counts. Cast and hit
  counts must come out of the engine's mechanics and rotation.
- Paired-RNG streams are keyed by name (`docs/paired-comparisons.md`). Don't
  make them positional.
- Other agents push to `main` in parallel. Keep engine changes small and
  additive, and rebase carefully, keeping everyone's features.

## Building a new spec (the recipe)

Examples to copy: `specs/retribution_herald.toml` (melee: `[gear]`,
`[gear.kit]`, trinkets and tier), `specs/fire_mage.toml` and
`specs/destruction_hellcaller.toml` (casters), and `docs/<spec>.md` for each.

1. **Reference.** `reference/<prefix>_core_observations.json` plus five stat
   probes. Many are already committed; if yours isn't, generate it with
   `tools/make_reference.sh <prefix> <MID2 profile.simc> <primary-stat> <simc-binary>`.
   That needs a SimC build; see the Dockerfile in Ltz-lab/wow-sims.
   Real-log observations come from `tools/wcl.py` (`docs/warcraft-logs.md`).
2. **Spec.** `specs/<spec>_<herotree>.toml`, using the hero tree of the
   profile in `profiles/`. Model resources, cooldowns, procs and the
   rotation from the APL. Look spells up with `tools/spelldb.py`.
   `tools/draft_spec.py` writes a first draft (abilities and buffs from DB2,
   rotation translated from the profile's APL, TODOs for the rest); see
   `docs/draft-spec.md`. Mark a draft as WIP and don't register it until it runs.
3. **Derived modifiers.** `tools/effects.py` → `derived/<spec>.toml`
   (`docs/talents-and-auras.md`).
4. **Calibrate.** `tools/calibrate.py ... --derived ...` →
   `calibration/<spec>.toml`. Compare with `tools/compare.py`. Iterate on the
   spec until hit counts per source match.
5. **Validate.** `tools/validate_stats.py` with the five probes. Targets: DPS
   within ~1% of the reference, each stat probe within ~1 point.
6. **Gear.** Add `[gear]` and `[gear.kit]` (`docs/gear.md`). If you have
   SimC, check with `tools/check_gear.py`.
7. **Register.** Add the spec to `SPECS` in `src/api.rs`, write
   `docs/<spec>.md` (results, validation, known gaps), and add a row to the
   README status table.
8. **Checks.** `cargo test --release`, `pytest python/tests` (after
   `maturin develop --release` in a venv), plus the identical-results check
   above.

While iterating, run `-n 2000`; use `-n 10000` for final numbers.

## Environment

- Rust stable; Python 3.11+ for `tools/` (needs `tomllib`).
- DB2 tables: `python3 tools/db2.py --build 12.1.5.70575` (into `target/db2`).
- Item data is committed (`data/gamedata.json.gz`).
- The web app that uses azsim is Ltz-lab/wow-sims (`gearsim/azsim_engine.py`
  maps class and spec to azsim spec names). After a spec lands, add its row
  there.

Commit messages: one summary line, then the details: what changed, the DPS
against the reference, and the worst stat probe.
