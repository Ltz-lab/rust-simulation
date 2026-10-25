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

This first prototype implements **Retribution Paladin (Herald of the Sun,
Midnight Season 2)**. Since we don't have a Warcraft Logs pipeline yet,
SimulationCraft's output stands in for real logs. It is used strictly as a
black box (run it, read its report and combat log), never its source code.

## Results

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

### Speed

Same machine (2 cores), 10,000 iterations: azsim **1.6s**, SimC **25.3s**.
Treat this as indicative only, since azsim models less than SimC does.
Per-purpose random streams cost nothing measurable: on a 4-core container,
10,000 iterations take a median 0.73s with them and 0.73s with the old single
RNG (2.85s vs 2.82s single-threaded, medians of 8 runs).

## What the raw data alone gets you

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
data/
  gamedata.json.gz          item/bonus/gem/enchant + scaling tables (built by tools/gamedata.py)
profiles/
  MID2_Paladin_Retribution.simc   gear of SimC's MID2 sample Ret profile
calibration/
  retribution_herald.toml   generated per-hit multipliers + crit offsets
reference/                  observations used for calibration and validation
tools/
  spelldb.py         look up spells in raw DB2 tables
  observe.py         turn a SimC (stand-in for logs) report into observations
  calibrate.py       fit per-hit multipliers; never touches hit counts
  compare.py         side-by-side per-source comparison
  validate_stats.py  held-out stat-sensitivity test
  paired_noise.py    paired vs independent noise of stat comparisons
  make_reference.sh  regenerate reference/ from a SimC build
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

# look up spells in the raw game data (CSV exports from wago.tools/db2/<Table>/csv)
# tables used: SpellName SpellEffect SpellMisc SpellCooldowns SpellCategories
#              SpellCategory SpellPower SpellAuraOptions SpellDuration
python3 tools/spelldb.py path/to/db2 "Blade of Justice" 383328
```

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
`trigger` (with `chance`, `delay`, `mastery_chance`), `extend_dot`.
Buffs can modify damage, crit, haste, GCD and per-ability recharge rate, or
store a percentage of damage dealt for a later burst. DoTs support hasted
ticks, partial final ticks, `pandemic`/`extend`/`reset` refresh,
extend-on-crit and on-tick effects.

## Known gaps

- **Haste over-response (+0.7 pts).** Concentrated in the Hammer of Wrath
  chain inside Avenging Wrath (it competes with Final Verdict for GCDs and
  Holy Power) and in Expurgation, whose damage *per tick* rises with haste in
  the reference, probably a rolling DoT that carries damage over on refresh.
- **Fitted rather than derived:** Avenging Wrath's haste/GCD/recharge effects,
  Judgment's cooldown, and the always-on haste procs are fitted to observed
  cadence. Blessing of An'she isn't modelled individually.
- **Small sources:** Consecration overlap (-11%) and Sun Sear (+10%) are each
  under 1% of damage.
- **Not yet built:** gear → stats (we take final stats as input), trinkets and
  tier sets, multi-target, movement, healers and tanks, a real Warcraft Logs
  ingestion pipeline, and an effect-interpretation layer for DB2 aura types.

## Next steps

1. Interpret DB2 aura/effect types (percent modifiers by spell family mask)
   to replace the calibration overlay with derived multipliers.
2. Warcraft Logs ingestion: the same `observe → calibrate → validate` loop on
   real logs instead of SimC output.
3. A rotation optimizer (search over priority lists with the engine as the judge).
4. Gear → stats from item and scaling tables.
5. More specs, ideally generated as drafts by an AI from data + tooltips and
   then verified by the validation loop above.
