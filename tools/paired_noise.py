#!/usr/bin/env python3
"""How much does pairing seeds cut the noise of a gear/stat comparison?

  python3 tools/paired_noise.py <spec.toml> <calibration.toml> <base_obs.json> \
      "+500 crit=obs_crit.json" "+500 haste=obs_haste.json" ... \
      [--seeds 50] [-n 500] [--md]

For each probe we apply the same final-stat change as tools/validate_stats.py
(the probe's stats minus the base observation's, via --set player.<stat>=...),
then run base and variant for many seeds with n iterations each:

  paired       delta = variant(seed)       - base(seed)
  independent  delta = variant(seed + K)   - base(seed)

The SD of the delta across seeds is the noise on "is B better than A" at that
iteration count. The variance-reduction factor (SD_independent / SD_paired)^2
is how many times fewer iterations a paired comparison needs for the same
precision.
"""
import json
import math
import os
import subprocess
import sys
import tempfile
import tomllib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from validate_stats import ENGINE, KEYS, stats  # noqa: E402

INDEPENDENT_OFFSET = 100_000  # seeds far apart never share iteration seeds


def run(spec, calib, overrides, seed, n):
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    cmd = [ENGINE, spec, "-n", str(n), "--seed", str(seed), "--calib", calib, "--json", out]
    for k, v in overrides.items():
        cmd += ["--set", f"player.{k}={v}"]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)
    dps = json.load(open(out))["dps"]
    os.unlink(out)
    return dps


def sd(xs):
    m = sum(xs) / len(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def main():
    args = sys.argv[1:]
    md = "--md" in args
    args = [a for a in args if a != "--md"]
    opts = {"--seeds": 50, "-n": 500}
    for k in opts:
        if k in args:
            i = args.index(k)
            opts[k] = int(args[i + 1])
            del args[i : i + 2]
    seeds, n = opts["--seeds"], opts["-n"]
    spec, calib, base_path, probes = args[0], args[1], args[2], args[3:]

    player = tomllib.load(open(spec, "rb"))["player"]
    base_spec = {k: player[k] for k in KEYS}
    b = stats(json.load(open(base_path)))
    seed_list = list(range(1, seeds + 1))
    base = {s: run(spec, calib, base_spec, s, n) for s in seed_list}

    rows = []
    for p in probes:
        label, path = p.split("=", 1)
        ps = stats(json.load(open(path)))
        st = {k: base_spec[k] + (ps[k] - b[k]) for k in KEYS}
        paired = [run(spec, calib, st, s, n) - base[s] for s in seed_list]
        indep = [run(spec, calib, st, s + INDEPENDENT_OFFSET, n) - base[s] for s in seed_list]
        sp, si = sd(paired), sd(indep)
        rows.append((label, sum(paired) / seeds, si, sp, (si / sp) ** 2))

    # SD from k samples has relative error ~1/sqrt(2(k-1)); a variance ratio ~2x that.
    rel = 2 / math.sqrt(2 * (seeds - 1))
    if md:
        print(f"| probe | mean delta | independent SD | paired SD | variance reduction |")
        print("|---|---:|---:|---:|---:|")
        for l, d, si, sp, f in rows:
            print(f"| {l} | {d:+,.0f} | {si:,.0f} | {sp:,.0f} | **{f:,.1f}x** |")
    else:
        print(f"{seeds} seeds x {n} iterations; single-run DPS SD {sd(list(base.values())):.0f}\n")
        print(f"{'probe':16s} {'mean delta':>10} {'indep SD':>9} {'paired SD':>10} {'reduction':>10}")
        for l, d, si, sp, f in rows:
            print(f"{l:16s} {d:+10.0f} {si:9.0f} {sp:10.0f} {f:9.1f}x")
    print(f"\n(variance-reduction factors are good to roughly +/-{100 * rel:.0f}% at {seeds} seeds)")


if __name__ == "__main__":
    main()
