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

Twenty-four specs so far, all Midnight Season 2, compared against SimC's sample
profiles (core kit, 300s Patchwerk):

| spec | reference DPS | azsim DPS | diff | worst stat probe | speed vs SimC |
|---|---:|---:|---:|---|---:|
| [Retribution Paladin (Herald)](docs/retribution.md) | 131,766 | 132,884 | +0.85% | haste +0.86 pts | ~16× |
| [Fire Mage (Sunfury)](docs/fire-mage.md) | 147,374 | 148,454 | +0.73% | haste +2.19 pts | ~15× |
| [Destruction Warlock (Hellcaller)](docs/destruction-warlock.md) | 157,019 | 155,825 | -0.76% | mastery +0.48 pts | ~7× |
| [Arcane Mage (Spellslinger)](docs/arcane-mage.md) | 133,260 | 133,980 | +0.54% | mastery +0.51 pts | — |
| [Affliction Warlock (Hellcaller)](docs/affliction-warlock.md) | 157,898 | 157,573 | -0.21% | haste -0.60 pts | — |
| [Frost Mage (Spellslinger)](docs/frost-mage.md) | 149,542 | 151,256 | +1.15% | mastery +0.71 pts | — |
| [Subtlety Rogue (Deathstalker)](docs/subtlety-rogue.md) | 179,254 | 181,350 | +1.17% | crit -0.40 pts | ~4× |
| [Elemental Shaman (Farseer)](docs/elemental-shaman.md) | 161,888 | 163,606 | +1.06% | haste +0.80 pts | — |
| [Demonology Warlock (Diabolist)](docs/demonology-warlock.md) | 160,301 | 158,236 | -1.29% | haste +0.74 pts | — |
| [Balance Druid (Elune’s Chosen)](docs/balance-druid.md) | 150,502 | 148,765 | -1.15% | mastery +0.36 pts | — |
| [Arms Warrior (Slayer)](docs/arms-warrior.md) | 164,332 | 163,522 | -0.49% | crit +0.49 pts | ~6× |
| [Shadow Priest (Voidweaver)](docs/shadow-priest.md) | 143,444 | 142,586 | -0.60% | haste -0.29 pts | — |
| [Enhancement Shaman (Stormbringer)](docs/enhancement-shaman.md) | 148,565 | 142,652 | -4.0% (known gap) | crit -1.30 pts | — |
| [Feral Druid (Wildstalker)](docs/feral-druid.md) | 169,230 | 169,363 | +0.08% | haste +0.17 pts | — |
| [Devourer Demon Hunter (Void-Scarred)](docs/devourer-demon-hunter.md) | 164,261 | 164,308 | +0.03% | haste -1.13 pts | — |
| [Havoc Demon Hunter (Fel-Scarred)](docs/havoc-demon-hunter.md) | 159,675 | 158,729 | -0.59% | haste -3.06 pts | ~10× |
| [Survival Hunter (Sentinel)](docs/survival-hunter.md) | 163,198 | 163,103 | -0.06% | agility +1.71 pts | — |
| [Assassination Rogue (Deathstalker)](docs/assassination-rogue.md) | 148,195 | 148,533 | +0.23% | haste +0.66 pts | — |
| [Frost Death Knight (Deathbringer)](docs/frost-deathknight.md) | 162,970 | 162,532 | -0.27% | crit -2.02 pts | — |
| [Marksmanship Hunter (Sentinel)](docs/marksmanship-hunter.md) | 155,473 | 156,903 | +0.92% | crit -1.89 pts | — |
| [Unholy Death Knight (Rider of the Apocalypse)](docs/unholy.md) | 148,582 | 149,950 | +0.92% | haste -1.29 pts | — |
| [Windwalker Monk (Shado-Pan)](docs/windwalker-shadopan.md) | 169,265 | 167,124 | -1.26% | crit -0.70 pts | — |
| [Fury Warrior (Slayer)](docs/fury-warrior.md) | 167,734 | 161,427 | -3.76% (known gap) | crit +1.19 pts | ~8× |
| [Outlaw Rogue (Trickster)](docs/outlaw-rogue.md) | 162,375 | 158,651 | -2.29% (known gap) | haste +1.45 pts | — |

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
  [Destruction Warlock](docs/destruction-warlock.md), [Affliction Warlock](docs/affliction-warlock.md),
  [Subtlety Rogue](docs/subtlety-rogue.md), [Elemental Shaman](docs/elemental-shaman.md),
  [Demonology Warlock](docs/demonology-warlock.md), [Balance Druid](docs/balance-druid.md),
  [Arms Warrior](docs/arms-warrior.md), [Shadow Priest](docs/shadow-priest.md),
  [Feral Druid](docs/feral-druid.md), [Devourer Demon Hunter](docs/devourer-demon-hunter.md),
  [Havoc Demon Hunter](docs/havoc-demon-hunter.md),
  [Survival Hunter](docs/survival-hunter.md),
  [Assassination Rogue](docs/assassination-rogue.md),
  [Frost Death Knight](docs/frost-deathknight.md),
  [Marksmanship Hunter](docs/marksmanship-hunter.md),
  [Unholy Death Knight](docs/unholy.md),
  [Windwalker Monk](docs/windwalker-shadopan.md),
  [Fury Warrior](docs/fury-warrior.md),
  [Enhancement Shaman](docs/enhancement-shaman.md),
  [Outlaw Rogue](docs/outlaw-rogue.md) (results, validation, known gaps)
  [Unholy Death Knight](docs/unholy.md)
  [Outlaw Rogue](docs/outlaw-rogue.md)
  (results, validation, known gaps)
- [Spec format](docs/spec-format.md): how a spec file is written
- [Talents and auras from game data](docs/talents-and-auras.md): the DB2 effect layer
- [Gear → stats](docs/gear.md): computing stats from /simc gear lines
- [Paired comparisons](docs/paired-comparisons.md): low-noise gear and stat comparisons
- [Batch mode and Python](docs/batch-and-python.md)
- [Development](docs/development.md): repo layout and every tool's usage
- [Roadmap](docs/roadmap.md)
