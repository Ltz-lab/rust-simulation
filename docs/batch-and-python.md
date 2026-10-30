# Batch mode and Python

Gear decisions (Top Gear, upgrades) sim one character against many gear
variants. Batch mode does that in one call: the spec and calibration are loaded
once, every variant runs the same iteration seeds (so they share their luck, see
[paired comparisons](paired-comparisons.md)), and each variant reports its DPS plus the **paired delta
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
Gear lines need a spec with a `[gear]` table (Ret, Fire and Destruction today).

## Python module

```bash
pip install maturin
maturin develop --release          # into the active virtualenv
# or: maturin build --release      # abi3 wheel (Python >= 3.11) in target/wheels
pytest python/tests                # smoke test
```

```python
import azsim

azsim.sim("retribution_herald", gear_lines=lines, iterations=10_000, seed=1)
# {"spec", "iterations", "seconds", "dps", "dps_error_95", "fight_length", "sources", "buffs"}

azsim.sim_batch("retribution_herald", base_gear=lines,
                variants=[{"name": "ring_350", "gear": ["finger1=...,ilevel=350"]}],
                target_error=10)       # or iterations=...
# {"base": {...}, "variants": [{"name", "dps", "dps_error_95", "delta", "delta_error_95"}], ...}

azsim.spec_names(); azsim.supports_gear("fire_mage")
azsim.gear_stats("retribution_herald", lines)   # per-item ilvl + stats, totals
```

Both also return the base profile's breakdown, averaged per iteration:
`fight_length` (seconds), `sources` (every row that dealt damage, most first:
`id` as the spec names it, display `name`, owning `pet`, `spell_id`,
`damage`, `casts`, `hits`, `crits`, `dps`, `crit_pct` as 0-1) and `buffs`
(every buff that was ever up, highest uptime first: `id`, `name`, `spell_id`
borrowed from the same-named ability, `uptime` as 0-1, `count` of times it
went from down to up). Buffs tied to target health aren't tracked.

Results are plain dicts, the GIL is released while simulating, and errors raise
`ValueError`. Specs and calibrations are embedded in the module. The Rust side
is `azsim::api` (`cargo test` covers it).
