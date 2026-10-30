#!/usr/bin/env python3
"""Held-out validation: does the engine respond to stat changes like the reference?

  python3 tools/validate_stats.py <spec.toml> <calibration.toml> <base_obs.json> \
      "+500 crit=obs_crit.json" "+500 haste=obs_haste.json" ...

Each probe observation comes from the reference run with extra stats. We take
the probe's change in final stats (crit %, haste %, mastery, vers, attack or
spell power, whichever the spec uses), apply that delta to the spec's stats, re-run the engine with the SAME
calibration, and compare DPS gains. None of these probes are used for fitting.
"""
import json
import os
import subprocess
import sys
import tempfile
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ENGINE = os.path.join(HERE, "..", "target", "release", "azsim")
KEYS = {
    "attack_power": "attack_power",
    "spell_power": "spell_power",
    "crit": "attack_crit",
    "haste": "haste_pct",
    "mastery": "mastery_value",
    "versatility": "damage_versatility",
}


def stats(obs, keys, mast_ap=0.0):
    s = obs["buffed_stats"]["stats"]
    out = {k: s[v] for k, v in keys.items()}
    if mast_ap:
        # The reference's attack power already carries x(1 + c * mastery); the
        # spec applies mastery per source instead, so strip it here.
        out["attack_power"] = s["attack_power"] / (1 + mast_ap * s["mastery_value"])
        out["mastery"] = mast_ap * s["mastery_value"]
    return out


def engine_dps(spec, calib, overrides, n=20000):
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    cmd = [ENGINE, spec, "-n", str(n), "--seed", "42", "--calib", calib, "--json", out]
    for k, v in overrides.items():
        cmd += ["--set", f"player.{k}={v}"]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)
    dps = json.load(open(out))["dps"]
    os.unlink(out)
    return dps


def main():
    spec, calib, base_path, probes = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
    md = "--md" in probes
    probes = [p for p in probes if p != "--md"]
    full = tomllib.load(open(spec, "rb"))
    player = full["player"]
    # [gear] mastery_ap = c: attack power scales with (1 + c * mastery) (Protection Warrior)
    mast_ap = full.get("gear", {}).get("mastery_ap", 0.0)
    keys = {k: v for k, v in KEYS.items() if k in player}
    if "spell_power" in keys:
        keys["crit"] = "spell_crit"
    base_spec = {k: player[k] for k in keys}
    base_obs = json.load(open(base_path))
    b = stats(base_obs, keys, mast_ap)
    d0 = engine_dps(spec, calib, base_spec)
    rows = []
    for p in probes:
        label, path = p.split("=", 1)
        o = json.load(open(path))
        ps = stats(o, keys, mast_ap)
        st = {k: base_spec[k] + (ps[k] - b[k]) for k in keys}
        d1 = engine_dps(spec, calib, st)
        ref = (o["dps"] / base_obs["dps"] - 1) * 100
        ours = (d1 / d0 - 1) * 100
        rows.append((label, ref, ours))
    if md:
        print("| stat probe | reference DPS gain | azsim DPS gain | difference |")
        print("|---|---:|---:|---:|")
        for l, r, o in rows:
            print(f"| {l} | {r:+.2f}% | {o:+.2f}% | {o - r:+.2f} pts |")
    else:
        print(f"{'probe':18s} {'reference':>10} {'azsim':>9} {'diff':>8}")
        for l, r, o in rows:
            print(f"{l:18s} {r:+9.2f}% {o:+8.2f}% {o - r:+7.2f}")
    order = lambda i: " > ".join(r[0].split()[-1] for r in sorted(rows, key=lambda r: -r[i]))
    print(f"\nreference stat priority: {order(1)}\nazsim stat priority:     {order(2)}")


if __name__ == "__main__":
    main()
