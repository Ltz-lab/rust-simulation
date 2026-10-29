# Development

## Layout


```
src/
  main.rs      CLI, parallel iterations (rayon), reporting, JSON output
  sim.rs       the engine: event queue, cooldowns/charges, buffs, DoTs,
               triggers, procs, resources, pets. No class-specific code.
  spec.rs      the spec file format (serde) and calibration overlay
               (derived multipliers, residuals, buff patches)
  gear.rs      gear -> stats: parses /simc gear lines, item level from bonus
               IDs, stat budgets from the game's item scaling tables
  expr.rs      rotation condition language (holy_power>=3&!buff.x.up ...)
specs/
  retribution_herald.toml   Ret spec; every number tagged [db2] or [obs]
  fire_mage.toml            Fire spec, same tagging
  destruction_hellcaller.toml  Destruction Warlock spec, same tagging
  examples/summoner.toml    made-up spec showing pets and summons
data/
  gamedata.json.gz          item/bonus/gem/enchant + scaling tables (built by tools/gamedata.py)
profiles/
  MID2_Paladin_Retribution.simc   gear of SimC's MID2 sample Ret profile
  MID2_Warlock_Destruction.simc   gear and talents of SimC's MID2 sample Destruction profile
derived/
  retribution_herald.toml   multipliers and buff modifiers read from game data (tools/effects.py)
  fire_mage.toml
  destruction_hellcaller.toml
calibration/
  retribution_herald.toml   derived multipliers + fitted residuals and crit offsets
  fire_mage.toml
  destruction_hellcaller.toml
reference/                  observations used for calibration and validation
tools/
  spelldb.py         look up spells in raw DB2 tables
  db2.py             shared DB2 CSV loading
  talents.py         decode a talent loadout string against the DB2 trait tables
  effects.py         derive per-source and per-buff damage modifiers from talents and auras
  observe.py         turn a SimC (stand-in for logs) report into observations (pets included)
  calibrate.py       fit residual per-hit multipliers on top of derived ones; never touches hit counts
  compare.py         side-by-side per-source comparison
  validate_stats.py  held-out stat-sensitivity test
  paired_noise.py    paired vs independent noise of stat comparisons
  make_reference.sh  regenerate reference/ for a spec from a SimC profile + build
  gamedata.py        build data/gamedata.json.gz from wago.tools DB2/game tables
  check_gear.py      gear -> stats check against SimC's per-item and buffed stats
  wcl.py             Warcraft Logs API -> observations from real fights (docs/warcraft-logs.md)
```


## Usage


```sh
cargo build --release

# run the calibrated Ret spec
./target/release/azsim specs/retribution_herald.toml -n 10000 \
    --calib calibration/retribution_herald.toml --json out.json

# one fight with a full event log
./target/release/azsim specs/retribution_herald.toml --log --set fight.variance=0

# override any numeric field (e.g. for stat weights)
./target/release/azsim specs/retribution_herald.toml --calib calibration/retribution_herald.toml \
    --set player.haste=0.25

# compare against the reference
python3 tools/compare.py reference/ret_core_observations.json out.json

# re-fit calibration, then run the held-out validation
python3 tools/calibrate.py specs/retribution_herald.toml reference/ret_core_observations.json calibration/retribution_herald.toml \
    --derived derived/retribution_herald.toml
python3 tools/validate_stats.py specs/retribution_herald.toml calibration/retribution_herald.toml \
    reference/ret_core_observations.json \
    "+500 crit=reference/ret_probe_crit_rating.json" "+500 haste=reference/ret_probe_haste_rating.json" \
    "+500 mastery=reference/ret_probe_mastery_rating.json" "+500 vers=reference/ret_probe_versatility_rating.json" \
    "+200 strength=reference/ret_probe_strength.json"

# how much noise does a same-seed comparison remove?
python3 tools/paired_noise.py specs/retribution_herald.toml calibration/retribution_herald.toml \
    reference/ret_core_observations.json \
    "+500 crit=reference/ret_probe_crit_rating.json" "+500 haste=reference/ret_probe_haste_rating.json" \
    "+500 mastery=reference/ret_probe_mastery_rating.json" "+500 vers=reference/ret_probe_versatility_rating.json"

# sim a character from their gear (the /simc addon export's slot lines)
./target/release/azsim specs/retribution_herald.toml --calib calibration/retribution_herald.toml \
    --gear profiles/MID2_Paladin_Retribution.simc
# ...or just print the per-item and final stats it computes
./target/release/azsim specs/retribution_herald.toml --gear profiles/MID2_Paladin_Retribution.simc --gear-stats

# check gear -> stats (and DPS) against SimC for the same gear
python3 tools/check_gear.py "/path/to/simc" profiles/MID2_Paladin_Retribution.simc --no-trinkets --dps

# rebuild the item data for a new game build (downloads ~70 MB of DB2 CSVs into target/)
python3 tools/gamedata.py --build 12.1.5.70575

# Fire: same loop
./target/release/azsim specs/fire_mage.toml -n 10000 --calib calibration/fire_mage.toml --json out.json
python3 tools/compare.py reference/fire_core_observations.json out.json
python3 tools/calibrate.py specs/fire_mage.toml reference/fire_core_observations.json calibration/fire_mage.toml \
    --derived derived/fire_mage.toml --rounds 6
python3 tools/validate_stats.py specs/fire_mage.toml calibration/fire_mage.toml \
    reference/fire_core_observations.json \
    "+500 crit=reference/fire_probe_crit_rating.json" "+500 haste=reference/fire_probe_haste_rating.json" \
    "+500 mastery=reference/fire_probe_mastery_rating.json" "+500 vers=reference/fire_probe_versatility_rating.json" \
    "+200 intellect=reference/fire_probe_intellect.json"

# Destruction: same loop
./target/release/azsim specs/destruction_hellcaller.toml -n 10000 --calib calibration/destruction_hellcaller.toml --json out.json
python3 tools/compare.py reference/destro_core_observations.json out.json
python3 tools/effects.py target/db2 specs/destruction_hellcaller.toml reference/destro_core_observations.json \
    profiles/MID2_Warlock_Destruction.simc --out derived/destruction_hellcaller.toml
python3 tools/calibrate.py specs/destruction_hellcaller.toml reference/destro_core_observations.json \
    calibration/destruction_hellcaller.toml --derived derived/destruction_hellcaller.toml --rounds 6
python3 tools/validate_stats.py specs/destruction_hellcaller.toml calibration/destruction_hellcaller.toml \
    reference/destro_core_observations.json \
    "+500 crit=reference/destro_probe_crit_rating.json" "+500 haste=reference/destro_probe_haste_rating.json" \
    "+500 mastery=reference/destro_probe_mastery_rating.json" "+500 vers=reference/destro_probe_versatility_rating.json" \
    "+200 intellect=reference/destro_probe_intellect.json"

# regenerate a reference: prefix, SimC profile, primary stat, SimC binary
tools/make_reference.sh fire $SIMC/profiles/MID2/MID2_Mage_Fire.simc intellect $SIMC/build/simc

# look up spells in the raw game data (CSV exports from wago.tools/db2/<Table>/csv)
# tables used: SpellName SpellEffect SpellMisc SpellCooldowns SpellCategories
#              SpellCategory SpellPower SpellAuraOptions SpellDuration
#              (optional) SpellCastTimes SpellProcsPerMinute
python3 tools/spelldb.py path/to/db2 "Blade of Justice" 383328

# real logs: top Demonology parses on a single-target boss -> observations (docs/warcraft-logs.md)
python3 tools/wcl.py survey --zone 53 --class Warlock --spec Demonology
python3 tools/wcl.py observe --encounter 3420 -n 25 --skip-aug --out reference/demo_wcl
```

`validate_stats.py` and `wcl.py` need Python 3.11+.
