# azsim

A fast, data-driven combat simulator for World of Warcraft, written in Rust.

The engine knows nothing about any class. A spec is a data file (abilities,
buffs, procs, DoTs, rotation), and the numbers in it come from two places:

- **game data**: the client's spell tables (DB2), read straight from
  [wago.tools](https://wago.tools) exports with `tools/spelldb.py`;
- **observed behaviour**: combat logs, mined for cast cadence, proc rates, how
  DoTs refresh, what triggers what, and average damage per hit.

The long-term aim is to remove the dependency on hand-maintained class modules:
mechanics come from data, and logs at scale verify and calibrate them.

Two specs so far, both Midnight Season 2: **Retribution Paladin (Herald of the
Sun)** and **Fire Mage (Sunfury)**. Since we don't have a Warcraft Logs
pipeline yet, SimulationCraft's output stands in for real logs. It is used
strictly as a black box (run it, read its report and combat log), never its
source code.

| spec | reference DPS | azsim DPS | diff | worst stat probe | speed vs SimC |
|---|---:|---:|---:|---|---:|
| Retribution Paladin | 131,766 | 132,942 | +0.89% | haste +0.74 pts | ~16× |
| Fire Mage | 147,374 | 148,453 | +0.73% | haste +2.23 pts | ~15× |

## Retribution Paladin

### Results

Reference: SimC's Midnight S2 sample Ret profile, core kit only (no trinkets,
tier set or consumables), 300s Patchwerk, 5,000 iterations.

| | DPS |
|---|---:|
| reference | 131,766 |
| azsim | 132,942 (±58) |
| difference | **+0.89%** |

### Structure: hit counts per fight

Calibration only adjusts damage *per hit*, so hit counts have to come out of
the engine's own mechanics and rotation. They match closely:

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| final_verdict | 40,061 | 41,061 | +2.5% | 160.1 | 164.2 |
| dawnlight | 17,830 | 17,931 | +0.6% | 211.8 | 213.1 |
| crusading_strike | 13,078 | 13,396 | +2.4% | 230.7 | 236.5 |
| hammer_of_wrath | 6,540 | 6,528 | -0.2% | 33.4 | 33.3 |
| execution_sentence | 5,915 | 5,979 | +1.1% | 5.2 | 5.3 |
| blade_of_justice_walk_into_light | 5,066 | 5,055 | -0.2% | 38.7 | 38.7 |
| suns_avatar | 4,447 | 4,473 | +0.6% | 211.8 | 213.1 |
| light_within_boj | 4,377 | 4,168 | -4.8% | 16.0 | 15.3 |
| highlords_judgment | 4,125 | 4,139 | +0.4% | 27.9 | 28.0 |
| light_within_boj_wil | 3,934 | 3,946 | +0.3% | 11.4 | 11.4 |
| blade_of_justice | 3,690 | 3,507 | -4.9% | 30.8 | 29.3 |
| expurgation | 3,151 | 3,140 | -0.4% | 138.9 | 138.4 |
| judgment | 2,611 | 2,600 | -0.4% | 27.2 | 27.2 |
| hammer_of_wrath_divine_resonance | 2,049 | 2,065 | +0.8% | 15.8 | 15.9 |
| truths_wake | 1,963 | 1,935 | -1.4% | 76.1 | 75.1 |
| hammer_of_wrath_divine_toll | 1,731 | 1,738 | +0.4% | 5.3 | 5.4 |
| venomfang | 1,298 | 1,315 | +1.3% | 167.1 | 169.4 |
| wake_of_ashes | 1,229 | 1,233 | +0.3% | 10.3 | 10.3 |
| sun_sear | 1,069 | 1,172 | +9.7% | 179.0 | 196.5 |
| execution_sentence_init | 1,097 | 1,138 | +3.8% | 5.2 | 5.5 |
| divine_storm | 1,100 | 1,099 | -0.0% | 5.4 | 5.4 |
| consecration_tick_blade_of_justice | 1,070 | 955 | -10.7% | 409.8 | 365.8 |
| rune_of_unleashed_fire | 1,028 | 1,028 | -0.0% | 50.3 | 50.4 |
| seething_flames_1 | 863 | 868 | +0.5% | 10.3 | 10.3 |
| seething_flames_0 | 864 | 868 | +0.5% | 10.3 | 10.3 |
| venomfang_burst | 561 | 561 | -0.1% | 38.7 | 38.7 |
| hammer_of_wrath_second_sunrise | 442 | 434 | -1.9% | 6.7 | 6.6 |
| rune_of_unleashed_fire_lingering | 324 | 323 | -0.2% | 242.0 | 241.6 |
| divine_storm_tempest | 147 | 147 | -0.2% | 5.4 | 5.4 |
| divine_storm_second_sunrise | 81 | 84 | +3.0% | 1.1 | 1.1 |
| divine_storm_second_sunrise_tempest | 32 | 33 | +2.5% | 1.1 | 1.1 |

Holy Power economy also matches: 502.9 generated per fight vs 500.1 in the
reference (81 vs 95 lost to overcapping).

### Held-out validation: stat sensitivity

The real test, since none of these runs were used for fitting. Each probe adds
stats in the reference; we apply the same change in final stats to our spec,
keep the calibration fixed, and compare DPS gains.

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.50% | +8.24% | -0.25 pts |
| +500 haste | +9.00% | +9.74% | +0.74 pts |
| +500 mastery | +9.33% | +9.26% | -0.07 pts |
| +500 vers | +8.79% | +8.82% | +0.03 pts |
| +200 strength | +6.05% | +6.10% | +0.05 pts |

Mastery, versatility and strength are within 0.1 points, crit within 0.3.
**Haste over-responds by 0.7 points**, enough to swap haste and mastery at the
top of the stat priority. See *Known gaps*.

### Paired comparisons

Gear and stat decisions care about *differences* between two setups, not
absolute DPS. azsim draws every random number from its own deterministic
stream, keyed by (iteration seed, purpose): fight length, the crit roll of each
damage source, the damage range of each source, every chance roll site (each
proc or trigger effect in the spec) and every RPPM proc. So the 40th Final
Verdict crit roll is the same number in both runs of a pair, even when one run
got an extra proc earlier and the two fights have drifted apart. Run two setups
with the same `--seed` and they share all the luck they can.

`tools/paired_noise.py` measures it: 50 seeds × 500 iterations per setup, the
same stat changes as the held-out probes, and the SD of the DPS delta when both
setups use the same seed (paired) vs different seeds (independent). The
variance-reduction factor is how many times fewer iterations a paired
comparison needs for the same precision (each factor is good to about ±20%).

| probe | mean delta | before: indep / paired SD | before: reduction | after: indep / paired SD | after: reduction |
|---|---:|---:|---:|---:|---:|
| +500 crit | +10,973 | 223 / 129 | 3.0x | 180 / 47 | **14.8x** |
| +500 haste | +12,915 | 220 / 136 | 2.6x | 182 / 74 | **6.1x** |
| +500 mastery | +12,306 | 226 / 132 | 2.9x | 179 / 29 | **37.6x** |
| +500 vers | +11,723 | 239 / 12 | 411x | 175 / 12 | **226x** |

"Before" is the previous engine, where every roll came in order from one shared
RNG per iteration. Once two runs diverged, every later roll landed on a
different event.

In iterations: a paired +500 crit comparison at 500 iterations is as precise as
an unpaired one at about 7,400.

Versatility changes no decisions, so pairing removes nearly all of its noise
either way. What's left is its gain being proportional to DPS (8.8% of a run's
SD of 132 is 12). Crit and mastery only move roll thresholds, so per-purpose
streams keep almost every outcome aligned. Haste is the hard case: it shifts
every GCD and tick, so the rotation eventually takes a different branch and the
two runs diverge in what they cast. Even +0.2% haste leaves a paired SD of
about 36. The streams still keep each source's n-th roll shared, but cast-count
divergence is the floor. With all randomness turned off except fight length, a
haste change still has a paired SD of about 20, because its gain genuinely
depends on how many cooldowns fit into the fight.

Streams are keyed by *name* (`crit/final_verdict`, `roll/final_verdict/on_cast/1`,
`rppm/rune_of_unleashed_fire`, ...), not by position in the spec. So a gear set
that adds a trinket ability or proc leaves every other stream untouched:
inserting a firing, roll-consuming but damageless ability at the top of the
spec gives bit-identical DPS for the same seed. Each stream is a splitmix64
sequence, with no measurable cost in speed.

### Gear → stats

`--gear profile.simc` computes the character's stats from their gear (the
`/simc` addon export's slot lines) instead of taking final stats as input.
Item stats come from the game's item tables: each item's stat allocation, its
item level from bonus IDs (upgrade tracks, crafting quality), the stat budget
for that level and slot, combat-rating and stamina multipliers, plus gems and
enchants. Ratings turn into percent at the character's level with diminishing
returns. On top sit a few per-character constants in the spec's `[gear]`
table (base strength, passives, raid buffs), each observed from the
reference's reported stats.

Against SimC's MID2 sample Ret profile (`tools/check_gear.py`), all 15 items'
item levels and every rating match exactly, with and without trinkets:

| stat | azsim | reference | diff |
|---|---:|---:|---:|
| crit rating | 1,051 | 1,051 | 0 |
| haste rating | 816 | 816 | 0 |
| mastery rating | 1,062 | 1,062 | 0 |
| versatility rating | 107 | 107 | 0 |
| attack power | 2,696 | 2,696 | 0 |
| crit | 31.85% | 31.85% | 0 |
| haste | 20.92% | 20.92% | 0 |
| mastery | 50.07% | 50.07% | 0 |
| versatility | 4.98% | 4.98% | 0 |

DPS from `--gear` is 132,876 vs 132,902 from the hand-entered final stats
(-0.02%, same seed; SimC 131,771). Real Raidbots Top Gear exports
(Havoc DH, BM Hunter, Demonology Warlock) load too: Havoc matches every
rating exactly, Demonology is within 2 (one off-hand, see *Known gaps*).
Loading the item data costs about 30ms once per run.

### Speed

Same machine (2 cores), 10,000 iterations: azsim **1.6s**, SimC **25.3s**.
Treat this as indicative only, since azsim models less than SimC does.
Per-purpose random streams cost nothing measurable: on a 4-core container,
10,000 iterations take a median 0.73s with them and 0.73s with the old single
RNG (2.85s vs 2.82s single-threaded, medians of 8 runs).

### What the raw data alone gets you

With coefficients straight from the game tables and base mechanics only, the
spec reached **~35% of the reference DPS**. Adding the mechanics observed in
logs (crit damage, the weapon component of attack power, Execution Sentence
storing damage, and so on) brings the uncalibrated spec to **~50%**. The rest
is talent and aura multipliers encoded in effect/aura types that still need
interpreting. That gap is what the calibration overlay covers for now, and what
a proper effect-interpretation layer (or log-driven fitting at scale) has to close.

Several mechanics weren't visible in the base data at all and were found by
observing logs: Judgment's second charge, Dawnlight extending rather than
resetting, mastery doubling as the Highlord's Judgment proc chance, Avenging
Wrath shortening the GCD and speeding up Hammer of Wrath's recharge, Truth's
Wake extending on crit ticks, and DoTs critting harder than direct hits. The
stat probes caught three of these, which is exactly what held-out validation
is for.

## Fire Mage (Sunfury)

`specs/fire_mage.toml`, built the same way: coefficients, cast times, charges,
durations and talent values from DB2 (plus the `Spell` table's tooltip text for
what each talent does), the rest observed in SimC's log. Fire is a very
different shape from Ret: hardcasts, an off-GCD spell, crit chains, a rolling
DoT that banks damage, and a pet, so it was a real test of the "no class code"
rule. Every mechanic below is generic engine data, not mage code.

### Results

Reference: SimC's Midnight S2 sample Fire profile (Sunfury), core kit only (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000 iterations.
Pet damage (Arcane Phoenix) is included, as `arcane_phoenix_*` sources.

| | DPS |
|---|---:|
| reference | 147,374 |
| azsim | 148,453 (±138) |
| difference | **+0.73%** |

### Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| pyroblast | 60,712 | 62,610 | +3.1% | 145.5 | 149.9 |
| ignite | 33,949 | 33,951 | +0.0% | 299.5 | 299.7 |
| intensifying_flame | 9,600 | 9,601 | +0.0% | 299.5 | 299.7 |
| fire_blast | 7,802 | 7,716 | -1.1% | 98.0 | 97.0 |
| scorch | 7,603 | 7,296 | -4.0% | 30.9 | 29.7 |
| fireball | 5,953 | 5,488 | -7.8% | 48.9 | 45.2 |
| meteor_impact | 5,854 | 5,859 | +0.1% | 9.6 | 9.7 |
| meteorite_impact | 4,874 | 4,867 | -0.1% | 115.5 | 115.4 |
| meteor_burn | 3,327 | 3,320 | -0.2% | 76.8 | 76.8 |
| arcane_phoenix_pyroblast | 1,450 | 1,447 | -0.2% | 26.9 | 26.9 |
| rune_of_unleashed_fire | 1,378 | 1,376 | -0.1% | 50.2 | 50.3 |
| arcane_phoenix_arcane_barrage | 1,179 | 1,175 | -0.4% | 26.9 | 26.8 |
| arcane_phoenix_greater_pyroblast | 867 | 860 | -0.8% | 5.1 | 5.0 |
| arcane_phoenix_meteorite_bug_impact | 807 | 799 | -1.0% | 15.1 | 15.0 |
| acidbrood_scourge | 776 | 771 | -0.7% | 22.8 | 22.9 |
| arcane_phoenix_arcane_surge | 578 | 577 | -0.1% | 5.1 | 5.1 |
| rune_of_unleashed_fire_lingering | 363 | 426 | +17.5% | 243.5 | 287.1 |
| arcane_phoenix_meteorite_impact | 311 | 308 | -0.9% | 5.0 | 5.0 |
| frostbolt (Mirror Image) | 10 | 0 | -100.0% | 27.7 | 0.0 |

The crit economy matches: 122 Heating Ups and 118 Hot Streaks per fight against
125 and 115 in the reference, Fire Blast 97 vs 98, Combustion 14.9s vs 14.7s
long. Split by phase (Combustion, execute, filler), Pyroblast, Fire Blast,
Fireball and Scorch counts are within a few casts of the reference in each.

### Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +4.61% | +4.17% | -0.43 pts |
| +500 haste | +8.24% | +10.47% | +2.23 pts |
| +500 mastery | +6.53% | +6.44% | -0.09 pts |
| +500 vers | +7.96% | +7.92% | -0.04 pts |
| +200 intellect | +6.49% | +6.61% | +0.13 pts |

Mastery, versatility and intellect are within 0.15 points, crit within 0.45.
Mastery landing this close matters most: it runs entirely through Ignite, so
it checks the banked-DoT model directly. **Haste over-responds by 2.2 points**.
Both stat priorities still put haste first, then vers; mastery and intellect
swap at fourth and fifth. See *Known gaps*.

### Speed

Same machine (10 cores), 10,000 iterations: azsim **0.24s**, SimC **3.6s**.

### What the raw data alone gets you

With DB2 coefficients and the modelled mechanics but no calibration, the spec
reaches **62% of the reference** (90.8k). Ignite is the standout: a bank of
exactly mastery% of direct damage (×2.25 from Hot Streak Pyroblasts, per the
Hot Streak and Inflame tooltips) lands within 20% of the reference with no
fitting at all. The larger residuals sit on direct hits (Meteor ×2.6, Scorch
×2.5, Fire Blast ×2.2), where the Fire spec aura and talent multipliers live.

Mechanics read from the log rather than the tables: Hot Streak being spent as
Pyroblast is cast, so its own crit can start the next Heating Up; non-crits
breaking both Heating Up and the Fevered Incantation streak; the Combustion
pull delay (Firestarter) and Hyperthermia following the phoenix; the phoenix
casting once a second for Combustion's whole duration; and Fire Blast's
recharge budget. The tooltips list From the Ashes (every direct hit −0.5s),
Pyrocosm (every meteorite −0.5s), Fired Up (−2.5s) and Fiery Rush (+50% in
Combustion), but Fire Blast still came up 10% short inside Combustion until the
phoenix's meteorites were counted too.

### Engine features added for Fire

All generic, data-driven, and Ret's results are bit-for-bit unchanged:

- **spell power** and `sp_coeff` alongside attack power;
- **cast times** (hasted): the spell resolves at the end of the cast, the GCD
  runs from its start;
- **banked (rolling) DoTs**: `feed` a share of each hit (`pct` plus
  `per_mastery`) into a DoT that spreads its bank over the time left and resets
  its duration on each feed, keeping its tick cadence (Ignite, Intensifying Flame);
- **conditional effects** (`if` with the rotation condition language) and
  `on_noncrit` / `on_start` effect lists (crit chains, effects at cast time
  before the hit lands);
- **buffs**: overlapping stacks with their own timers, health-gated buffs
  (`below_health` / `above_health`, `target.health.pct` in conditions), per-ability
  crit, crit damage, mastery and power modifiers, `extend_buff`, `on_expire`;
- cooldown-rate buffs now also speed up a recharge already in progress;
- **`source`**: several ability variants (hardcast / Hot Streak / Hyperthermia
  Pyroblast) report under one result row and share one calibration entry.

## Batch mode and Python

Gear decisions (Top Gear, upgrades) sim one character against many gear
variants. Batch mode does that in one call: the spec and calibration are loaded
once, every variant runs the same iteration seeds (so they share their luck, see
*Paired comparisons*), and each variant reports its DPS plus the **paired delta
to the base with its own 95% error**. On the Ret sample, one ring upgrade:
DPS ±62, delta +996 ±4.4.

A variant is a list of /simc gear lines; each replaces the base's line for the
same slot, so a variant only needs the slots that change. `set` overrides numeric
spec fields, as `--set` does. With `target_error` (DPS) or `target_error_pct` (% of base DPS) it runs in chunks until every
variant's delta is that tight (or `max_iterations`).

```bash
# JSON in, JSON out ("-" reads stdin; --out writes a file instead of stdout)
./target/release/azsim batch request.json
```

```json
{"spec": "retribution_herald", "base_gear": ["head=...", "..."],
 "variants": [{"name": "ring_350", "gear": ["finger1=...,ilevel=350"]}],
 "iterations": 10000, "seed": 1}
```

`spec` is a built-in name (`retribution_herald`, `fire_mage`) or a path to a spec
file; `calib` defaults to the built-in's calibration (`"none"` turns it off).
Gear lines need a spec with a `[gear]` table (Ret today).

### Python module

```bash
pip install maturin
maturin develop --release          # into the active virtualenv
# or: maturin build --release      # abi3 wheel (Python >= 3.11) in target/wheels
pytest python/tests                # smoke test
```

```python
import azsim

azsim.sim("retribution_herald", gear_lines=lines, iterations=10_000, seed=1)
# {"spec", "iterations", "seconds", "dps", "dps_error_95"}

azsim.sim_batch("retribution_herald", base_gear=lines,
                variants=[{"name": "ring_350", "gear": ["finger1=...,ilevel=350"]}],
                target_error=10)       # or iterations=...
# {"base": {...}, "variants": [{"name", "dps", "dps_error_95", "delta", "delta_error_95"}], ...}

azsim.spec_names(); azsim.supports_gear("fire_mage")
```

Results are plain dicts, the GIL is released while simulating, and errors raise
`ValueError`. Specs and calibrations are embedded in the module. The Rust side
is `azsim::api` (`cargo test` covers it).

## Layout

```
src/
  main.rs      CLI, parallel iterations (rayon), reporting, JSON output
  sim.rs       the engine: event queue, cooldowns/charges, buffs, DoTs,
               triggers, procs, resources. No class-specific code.
  spec.rs      the spec file format (serde) and calibration overlay
  gear.rs      gear -> stats: parses /simc gear lines, item level from bonus
               IDs, stat budgets from the game's item scaling tables
  expr.rs      rotation condition language (holy_power>=3&!buff.x.up ...)
specs/
  retribution_herald.toml   Ret spec; every number tagged [db2] or [obs]
  fire_mage.toml            Fire spec, same tagging
data/
  gamedata.json.gz          item/bonus/gem/enchant + scaling tables (built by tools/gamedata.py)
profiles/
  MID2_Paladin_Retribution.simc   gear of SimC's MID2 sample Ret profile
calibration/
  retribution_herald.toml   generated per-hit multipliers + crit offsets
  fire_mage.toml
reference/                  observations used for calibration and validation
tools/
  spelldb.py         look up spells in raw DB2 tables
  observe.py         turn a SimC (stand-in for logs) report into observations (pets included)
  calibrate.py       fit per-hit multipliers; never touches hit counts
  compare.py         side-by-side per-source comparison
  validate_stats.py  held-out stat-sensitivity test
  paired_noise.py    paired vs independent noise of stat comparisons
  make_reference.sh  regenerate reference/ for a spec from a SimC profile + build
  gamedata.py        build data/gamedata.json.gz from wago.tools DB2/game tables
  check_gear.py      gear -> stats check against SimC's per-item and buffed stats
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
python3 tools/calibrate.py specs/retribution_herald.toml reference/ret_core_observations.json calibration/retribution_herald.toml
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
python3 tools/calibrate.py specs/fire_mage.toml reference/fire_core_observations.json calibration/fire_mage.toml --rounds 6
python3 tools/validate_stats.py specs/fire_mage.toml calibration/fire_mage.toml \
    reference/fire_core_observations.json \
    "+500 crit=reference/fire_probe_crit_rating.json" "+500 haste=reference/fire_probe_haste_rating.json" \
    "+500 mastery=reference/fire_probe_mastery_rating.json" "+500 vers=reference/fire_probe_versatility_rating.json" \
    "+200 intellect=reference/fire_probe_intellect.json"

# regenerate a reference: prefix, SimC profile, primary stat, SimC binary
tools/make_reference.sh fire $SIMC/profiles/MID2/MID2_Mage_Fire.simc intellect $SIMC/build/simc

# look up spells in the raw game data (CSV exports from wago.tools/db2/<Table>/csv)
# tables used: SpellName SpellEffect SpellMisc SpellCooldowns SpellCategories
#              SpellCategory SpellPower SpellAuraOptions SpellDuration
#              (optional) SpellCastTimes SpellProcsPerMinute
python3 tools/spelldb.py path/to/db2 "Blade of Justice" 383328
```

`validate_stats.py` needs Python 3.11+ (`tomllib`).

## Spec format, briefly

```toml
[[abilities]]
name = "hammer_of_wrath"
requires_buff = "avenging_wrath"
cooldown = 7.5
cooldown_hasted = true
charges = 2
gain = { holy_power = 1 }
damage = { ap_coeff = 2.15807, school = "holy", mastery = true }
on_cast = [
  { type = "trigger", ability = "blade_of_justice_walk_into_light" },
  { type = "trigger", ability = "hammer_of_wrath_second_sunrise", chance = 0.20, delay = 0.325 },
]
on_crit = [{ type = "trigger", ability = "sun_sear_apply" }]

[[rotation]]
action = "hammer_of_wrath"
if = "cooldown.hammer_of_wrath.charges=2"
```

Effects: `buff`, `consume`, `reset_cooldown`, `reduce_cooldown`, `gain`,
`trigger` (with `chance`, `delay`, `mastery_chance`), `extend_dot`,
`extend_buff`, and `if` (`{ type = "if", if = "<condition>", then = [...],
else = [...] }`). Abilities run `on_start` (before the hit), `on_crit` /
`on_noncrit` (per hit) and `on_cast` (after) effects, can have a `cast_time`,
and can report under a shared `source`.
Buffs can modify damage, crit (globally or per ability), crit damage, haste,
mastery, power, GCD and per-ability recharge rate, or store a percentage of
damage dealt for a later burst; they can stack with independent timers
(`overlap`), follow the target's health, and fire `on_expire` effects. DoTs
support hasted ticks, partial final ticks, `pandemic`/`extend`/`reset` refresh,
extend-on-crit, on-tick effects, and `banked` rolling damage fed by hits.

## Known gaps

### Retribution

- **Haste over-response (+0.7 pts).** Concentrated in the Hammer of Wrath
  chain inside Avenging Wrath (it competes with Final Verdict for GCDs and
  Holy Power) and in Expurgation, whose damage *per tick* rises with haste in
  the reference, probably a rolling DoT that carries damage over on refresh.
- **Fitted rather than derived:** Avenging Wrath's haste/GCD/recharge effects,
  Judgment's cooldown, and the always-on haste procs are fitted to observed
  cadence. Blessing of An'she isn't modelled individually.
- **Small sources:** Consecration overlap (-11%) and Sun Sear (+10%) are each
  under 1% of damage.
- **Gear → stats:** the `[gear]` constants are per spec and race (Ret dwarf
  today); racial stat bonuses such as Human's +2% secondaries, enchants that
  work through a spell aura (ring crit-damage, weapon procs), embellishments,
  and set bonuses aren't read from data yet. Two rare item-level bonuses are
  off: type 53 (the Aln'hara Lantern, 331 vs 337) and one Myth bonus (12856,
  340 vs 341). On-equip primary stat from trinkets counts; their effects
  don't. Item data is the full equippable item set of build 12.1.5.70575
  (793 KB gzip'd); rebuild it per patch with `tools/gamedata.py`.

### Fire

- **Haste over-response (+2.2 pts).** Fire Blast's charge economy reacts too
  strongly: +500 haste raises Fire Blast casts 15% in azsim against 12% in the
  reference, and Hot Streak Pyroblasts (and the Ignite they feed at ×2.25)
  follow. Base counts match, so it's the loop gain that's off: each Fire Blast
  makes a Hot Streak, which feeds spheres → meteorites, Fired Up and more direct
  hits back into Fire Blast's recharge. Candidates still to test against the
  probe: how recharge reductions interact with a hasted recharge, Fire Blast's
  0.5s lockout between casts, and casting Fire Blast mid-cast (tried; it
  starves Combustion with this rotation paraphrase, so it's left out).
- **Crit −0.4 pts.** Burn It All (Combustion damage +20% of crit chance) is a
  fixed 16% here, read from the buff's value in the log.
- **Fitted rather than derived [obs]:** the gear stat procs, folded into final
  stats as their time-averaged ratings; the Fired Up chance inside Combustion
  (35%), Heat Shimmer per-tick chance, the phoenix's cast mix, the phoenix's
  meteorites counting for Pyrocosm, crit multiplier and Fevered Incantation
  per stack, Meteor's 2.25s landing time.
- **Calibration nudges the crit chain.** Per-source crit offsets are averages
  across phases, so after calibration filler Pyroblast crits slightly more
  often: Pyroblast +3% hits and Fireball −8%. Uncalibrated, they're 146.4
  and 47.8 against the reference's 145.5 and 48.9.
- **Simplified:** no travel time; Fire Blast fires between casts rather than
  mid-cast; Rune of Unleashed Fire's lingering DoT overlaps too long (+17%
  ticks, 0.3% of damage); Mirror Image isn't modelled.

### Not yet built

Trinkets and tier sets, multi-target, movement, healers and tanks, a real Warcraft Logs ingestion pipeline, and an effect-interpretation layer for DB2 aura types.

## Next steps

1. Interpret DB2 aura/effect types (percent modifiers by spell family mask)
   to replace the calibration overlay with derived multipliers.
2. Warcraft Logs ingestion: the same `observe → calibrate → validate` loop on
   real logs instead of SimC output.
3. A rotation optimizer (search over priority lists with the engine as the judge).
4. Top Gear: sim many gear sets per run (the item data loads once), with
   paired seeds so the differences between sets carry little noise.
5. More specs, ideally generated as drafts by an AI from data + tooltips and
   then verified by the validation loop above.
