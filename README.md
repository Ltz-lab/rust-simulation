# azsim

A fast, data-driven combat simulator for World of Warcraft, written in Rust.

The goal is a faster, easier-to-extend alternative to SimulationCraft. The
engine knows nothing about any class: a spec is a data file (abilities, buffs,
procs, DoTs, rotation), and the numbers in it come from:

- **game data**: the client's spell tables (DB2), read straight from
  [wago.tools](https://wago.tools) exports with `tools/spelldb.py`;
- **observed behaviour**: combat logs, mined for cast cadence, proc rates, how
  DoTs refresh, what triggers what, and average damage per hit;
- **SimC's source**: its class modules and APLs are a valid reference for how
  a mechanic works.

Keeping mechanics in data rather than class code is the preferred design, but
it is a means, not a rule: whatever gets a spec right fastest is fair game.
Since we don't have a Warcraft Logs pipeline yet, SimC's output stands in for
real logs. Because SimC is GPL-3.0, azsim is licensed under GPL-3.0 too (see
[LICENSE](LICENSE)).

## Status

Three specs so far, all Midnight Season 2, compared against SimC's sample
profiles (core kit, 300s Patchwerk):

| spec | reference DPS | azsim DPS | diff | worst stat probe | speed vs SimC |
|---|---:|---:|---:|---|---:|
| [Retribution Paladin (Herald)](docs/retribution.md) | 131,766 | 132,884 | +0.85% | haste +0.86 pts | ~16× |
| [Fire Mage (Sunfury)](docs/fire-mage.md) | 147,374 | 148,454 | +0.73% | haste +2.19 pts | ~15× |
| [Destruction Warlock (Hellcaller)](docs/destruction-warlock.md) | 157,019 | 155,825 | -0.76% | mastery +0.48 pts | ~7× |

With gear lines, Ret also models SimC's default consumables and Bloodlust,
and any spec gets trinket effects and set bonuses from an
[effect library](docs/gear.md#trinkets-and-set-bonuses) read at the item's
level: on the app's Ret character azsim gives 227,081 against SimC's
like-for-like 225,566 (+0.67%,
[full kit vs SimC](docs/retribution.md#full-kit-a-real-character-against-simc)).
Not yet built: most gear effects beyond the library's, multi-target, movement,
healers and tanks, Warcraft Logs ingestion. See the [roadmap](docs/roadmap.md).

## Quick start

```sh
cargo build --release

# run the calibrated Ret spec
./target/release/azsim specs/retribution_herald.toml -n 10000 \
    --calib calibration/retribution_herald.toml --json out.json

# sim a character from their gear (the /simc addon export's slot lines)
./target/release/azsim specs/retribution_herald.toml --calib calibration/retribution_herald.toml \
    --gear profiles/MID2_Paladin_Retribution.simc
```

## Docs

- Specs: [Retribution Paladin](docs/retribution.md), [Fire Mage](docs/fire-mage.md),
  [Destruction Warlock](docs/destruction-warlock.md)
  (results, validation, known gaps)
- [Spec format](docs/spec-format.md): how a spec file is written
- [Talents and auras from game data](docs/talents-and-auras.md): the DB2 effect layer
- [Gear → stats](docs/gear.md): computing stats from /simc gear lines
- [Paired comparisons](docs/paired-comparisons.md): low-noise gear and stat comparisons
- [Batch mode and Python](docs/batch-and-python.md)
- [Development](docs/development.md): repo layout and every tool's usage
- [Roadmap](docs/roadmap.md)
