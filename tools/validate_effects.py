#!/usr/bin/env python3
"""Trinket effect vs SimC: one trinket in an otherwise trinket-free profile.

For each case (SimC profile, azsim spec, trinket slot to test) it runs, on SimC and on azsim:
  none    both trinkets removed
  item    only the trinket under test
and reports each engine's DPS gain from the item, and azsim's gap to SimC for the
"item" profile with the effect modelled and with `gear.trinket_effects=0` (stats only).

    python3 tools/validate_effects.py --simc /path/to/simc --azsim target/release/azsim \
        --case fire_mage.txt:fire_mage:trinket2 ...  (a case file is a SimC profile)
"""
import argparse, json, os, re, subprocess, sys, tempfile

RAID = ["bloodlust", "arcane_intellect", "power_word_fortitude", "battle_shout", "mystic_touch",
        "chaos_brand", "skyfury", "mark_of_the_wild", "hunters_mark", "bleeding"]


def profile(src, keep):
    out = []
    for ln in open(src).read().splitlines():
        m = re.match(r"(trinket[12])=", ln)
        if m and m.group(1) not in keep:
            continue
        out.append(ln)
    return "\n".join(out) + "\n"


def simc_dps(simc, text, n):
    with tempfile.TemporaryDirectory() as d:
        p, j = os.path.join(d, "p.simc"), os.path.join(d, "o.json")
        open(p, "w").write(text)
        args = [simc, p, "fight_style=Patchwerk", "max_time=300", "vary_combat_length=0.2", "desired_targets=1",
                *[f"override.{b}=1" for b in RAID], f"iterations={n}", "threads=4", "json2=" + j, "report_details=0"]
        subprocess.run(args, check=True, capture_output=True)
        return json.load(open(j))["sim"]["players"][0]["collected_data"]["dps"]["mean"]


def azsim_dps(azsim, spec, text, n, off=False):
    with tempfile.TemporaryDirectory() as d:
        p, j = os.path.join(d, "p.simc"), os.path.join(d, "o.json")
        open(p, "w").write(text)
        args = [azsim, f"specs/{spec}.toml", "--gear", p, "-n", str(n), "--seed", "1", "--json", j]
        cal = f"calibration/{spec}.toml"
        if os.path.exists(cal):
            args += ["--calib", cal]
        if off:
            args += ["--set", "gear.trinket_effects=0"]
        subprocess.run(args, check=True, capture_output=True)
        r = json.load(open(j))
        return r["dps"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--simc", required=True)
    ap.add_argument("--azsim", default="target/release/azsim")
    ap.add_argument("--case", action="append", required=True, help="profile.txt:azsim_spec:trinket slot")
    ap.add_argument("-n", type=int, default=20000)
    a = ap.parse_args()
    for c in a.case:
        path, spec, slot = c.split(":")
        none, item = profile(path, set()), profile(path, {slot})
        s0, s1 = simc_dps(a.simc, none, a.n), simc_dps(a.simc, item, a.n)
        z0 = azsim_dps(a.azsim, spec, none, a.n)
        z1, zoff = azsim_dps(a.azsim, spec, item, a.n), azsim_dps(a.azsim, spec, item, a.n, off=True)
        print(f"{spec} {slot}: simc none {s0:,.0f} item {s1:,.0f} (+{s1 / s0 - 1:.2%}) | azsim none {z0:,.0f} "
              f"item {z1:,.0f} (+{z1 / z0 - 1:.2%}), stats only {zoff:,.0f} (+{zoff / z0 - 1:.2%}) | gap to simc "
              f"none {z0 / s0 - 1:+.2%} item modelled {z1 / s1 - 1:+.2%} stats only {zoff / s1 - 1:+.2%}")


if __name__ == "__main__":
    main()
