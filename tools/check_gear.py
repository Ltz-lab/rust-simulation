#!/usr/bin/env python3
"""Check azsim gear -> stats against SimulationCraft.

  python3 tools/check_gear.py <simc_binary> <profile.simc> \\
      [--spec specs/<spec>.toml] [--calib calibration/<spec>.toml] \\
      [--no-trinkets] [--dps] [--md]

Runs SimC as a black-box reference (gear-only buffed stats, consumables and set
bonuses disabled) and compares per-item ilvl/ratings, total character ratings
and final stats (attack or spell power, crit, haste, mastery, versatility), and
optional 5000-iteration DPS against azsim. The spec and calibration default to
the one matching the profile's `spec=` line.
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ENGINE = os.path.join(ROOT, "target", "release", "azsim")
DEFAULT_SIMC = "/Users/jack/Dev/wow sim/simc/build/simc"
DEFAULT_PROFILE = os.path.join(ROOT, "profiles", "MID2_Paladin_Retribution.simc")
SPEC_BY_PROFILE = {
    "retribution": "retribution_herald",
    "fire": "fire_mage",
    "destruction": "destruction_hellcaller",
    "affliction": "affliction_hellcaller",
    "subtlety": "subtlety_deathstalker",
}
DEFAULT_TALENTS = (
    "CYEAAAAAAAAAAAAAAAAAAAAAAAAAAAANbbzMzywMDAAAAAAjyYGmZsNmthZ2mxYMGmxCbDD"
    "YWmtZmZrBBAAsAYAwYGmBzYmZDYmZYMjBD"
)
SIMC_OVERRIDES = [
    "potion=disabled",
    "flask=disabled",
    "food=disabled",
    "augmentation=disabled",
    "temporary_enchant=disabled",
    "set_bonus=midnight_season_2_2pc=0",
    "set_bonus=midnight_season_2_4pc=0",
    "set_bonus=bite_of_zuljan_2pc=0",
    "override.bloodlust=0",
]
RATINGS = ("crit_rating", "haste_rating", "mastery_rating", "versatility_rating")


def norm_slot(slot):
    return slot[:-1] if slot in ("shoulders", "wrists") else slot


def filter_profile(profile_path, no_trinkets):
    lines = open(profile_path).read().splitlines()
    if no_trinkets:
        lines = [l for l in lines if not l.lstrip().startswith(("trinket1=", "trinket2="))]
    return lines


def spec_for_profile(profile_lines):
    for l in profile_lines:
        k, _, v = l.strip().partition("=")
        if k == "spec" and v.strip().lower() in SPEC_BY_PROFILE:
            return SPEC_BY_PROFILE[v.strip().lower()]
    return "retribution_herald"


def run_simc(simc_bin, profile_lines, dps_mode):
    lines = list(profile_lines)
    if not any(l.lstrip().startswith("talents=") for l in lines):
        lines.append(f"talents={DEFAULT_TALENTS}")
    lines.extend(SIMC_OVERRIDES)
    iters = 5000 if dps_mode else 1
    with tempfile.NamedTemporaryFile("w", suffix=".simc", delete=False) as f:
        f.write("\n".join(lines) + "\n")
        simc_in = f.name
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        simc_out = f.name
    try:
        subprocess.run(
            [simc_bin, simc_in, f"iterations={iters}", f"json2={simc_out}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )
        return json.load(open(simc_out))["sim"]["players"][0]
    finally:
        for p in (simc_in, simc_out):
            if os.path.exists(p):
                os.unlink(p)


def ensure_engine():
    if not os.path.exists(ENGINE):
        subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)


def core_kit(spec):
    """SimC runs here are core kit, so switch off the spec's consumables/Bloodlust,
    trinket effects and set bonuses."""
    off = ["--set", "gear.trinket_effects=0", "--set", "gear.set_bonuses=0"]
    return (["--set", "gear.kit.enabled=0"] if "[gear.kit]" in open(spec).read() else []) + off


def run_azsim_gear_stats(spec, gear_path):
    ensure_engine()
    out = subprocess.check_output([ENGINE, spec, "--gear", gear_path, "--gear-stats", *core_kit(spec)], text=True)
    return json.loads(out)


def run_azsim_dps(spec, calib, gear_path=None):
    ensure_engine()
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        out = f.name
    try:
        cmd = [ENGINE, spec, "--calib", calib, "-n", "5000", "--seed", "42", "--json", out]
        if gear_path:
            cmd += ["--gear", gear_path, *core_kit(spec)]
        subprocess.run(cmd, stdout=subprocess.DEVNULL, check=True)
        return json.load(open(out))["dps"]
    finally:
        if os.path.exists(out):
            os.unlink(out)


def compare_items(az_items, sc_gear):
    sc_by_slot = {norm_slot(k): v for k, v in sc_gear.items()}
    rows = []
    bad = False
    for it in az_items:
        slot = norm_slot(it["slot"])
        sc = sc_by_slot.get(slot, {})
        az_ilvl = it["ilvl"]
        sc_ilvl = sc.get("ilevel")
        if az_ilvl != sc_ilvl:
            bad = True
        enc = sc.get("encoded_item", "")
        mismatches = []
        if "gem_id=" not in enc and "enchant_id=" not in enc:
            for r in RATINGS:
                av = int(round(it.get("stats", {}).get(r, 0)))
                sv = int(round(sc.get(r, 0)))
                if av != sv:
                    mismatches.append(f"{r}: {av} vs {sv}")
                    if abs(av - sv) > 1:
                        bad = True
        rows.append((slot, it["id"], az_ilvl, sc_ilvl, ", ".join(mismatches) if mismatches else "ok"))
    return rows, bad


def compare_char(az, sc_stats):
    rows = []
    bad = False
    for r in RATINGS:
        av = float(az["gear"].get(r, 0))
        sv = float(sc_stats.get(r, 0))
        diff = av - sv
        if abs(diff) > 1:
            bad = True
        rows.append((r, f"{av:.0f}", f"{sv:.0f}", f"{diff:+.0f}"))

    # Buffed stats (before the spec's [gear.procs] averages) are what SimC reports.
    buffed = az.get("buffed", az["player"])
    for power in ("attack_power", "spell_power"):
        if power in sc_stats and power in buffed:
            p_a = float(buffed[power])
            p_s = float(sc_stats[power])
            rows.append((power, f"{p_a:.1f}", f"{p_s:.1f}", f"{p_a - p_s:+.1f}"))

    pct_fields = [
        ("crit %", "crit", "attack_crit"),
        ("haste %", "haste", "haste_pct"),
        ("mastery %", "mastery", "mastery_value"),
        ("versatility %", "versatility", "damage_versatility"),
    ]
    for label, ak, sk in pct_fields:
        av = float(buffed[ak]) * 100.0
        if sk not in sc_stats:
            continue
        sv = float(sc_stats[sk]) * 100.0
        rows.append((label, f"{av:.2f}%", f"{sv:.2f}%", f"{av - sv:+.2f}%"))
    return rows, bad


def print_tables(item_rows, char_rows, dps_info, md):
    if md:
        print("| slot | id | azsim ilvl | SimC ilvl | ratings |")
        print("|---|---:|---:|---:|---|")
        for slot, iid, ai, si, status in item_rows:
            print(f"| {slot} | {iid} | {ai} | {si} | {status} |")
        print()
        print("| stat | azsim | SimC | diff |")
        print("|---|---:|---:|---:|")
        for name, av, sv, dv in char_rows:
            print(f"| {name} | {av} | {sv} | {dv} |")
        if dps_info:
            d_spec, d_gear, d_simc = dps_info
            g_vs_s = (d_gear / d_simc - 1.0) * 100.0
            g_vs_p = (d_gear / d_spec - 1.0) * 100.0
            print()
            print("| run | DPS | vs SimC | vs spec stats |")
            print("|---|---:|---:|---:|")
            print(f"| SimC | {d_simc:,.0f} | — | — |")
            print(f"| azsim (spec stats) | {d_spec:,.0f} | {(d_spec / d_simc - 1.0) * 100.0:+.2f}% | — |")
            print(f"| azsim (--gear) | {d_gear:,.0f} | {g_vs_s:+.2f}% | {g_vs_p:+.2f}% |")
    else:
        print(f"{'slot':12s} {'id':>8s} {'az ilvl':>8s} {'sc ilvl':>8s}  {'ratings'}")
        for slot, iid, ai, si, status in item_rows:
            print(f"{slot:12s} {iid:8d} {str(ai):>8s} {str(si):>8s}  {status}")
        print()
        print(f"{'stat':20s} {'azsim':>10s} {'SimC':>10s} {'diff':>10s}")
        for name, av, sv, dv in char_rows:
            print(f"{name:20s} {av:>10s} {sv:>10s} {dv:>10s}")
        if dps_info:
            d_spec, d_gear, d_simc = dps_info
            print()
            print(f"{'SimC DPS':20s} {d_simc:10.0f}")
            print(f"{'azsim (spec) DPS':20s} {d_spec:10.0f} ({(d_spec / d_simc - 1.0) * 100.0:+.2f}% vs SimC)")
            print(
                f"{'azsim (--gear) DPS':20s} {d_gear:10.0f} "
                f"({(d_gear / d_simc - 1.0) * 100.0:+.2f}% vs SimC, "
                f"{(d_gear / d_spec - 1.0) * 100.0:+.2f}% vs spec)"
            )


def main():
    ap = argparse.ArgumentParser(description="Check azsim gear -> stats against SimC.")
    ap.add_argument("simc_binary", nargs="?", default=DEFAULT_SIMC)
    ap.add_argument("profile", nargs="?", default=DEFAULT_PROFILE)
    ap.add_argument("--spec", default=None)
    ap.add_argument("--calib", default=None)
    ap.add_argument("--no-trinkets", action="store_true")
    ap.add_argument("--dps", action="store_true")
    ap.add_argument("--md", action="store_true")
    args = ap.parse_args()

    prof_lines = filter_profile(args.profile, args.no_trinkets)
    spec_name = spec_for_profile(prof_lines)
    args.spec = args.spec or os.path.join(ROOT, "specs", f"{spec_name}.toml")
    args.calib = args.calib or os.path.join(ROOT, "calibration", f"{spec_name}.toml")
    sc_player = run_simc(args.simc_binary, prof_lines, args.dps)
    sc_gear = sc_player["gear"]
    sc_stats = sc_player["collected_data"]["buffed_stats"]["stats"]

    with tempfile.NamedTemporaryFile("w", suffix=".simc", delete=False) as f:
        f.write("\n".join(prof_lines) + "\n")
        gear_tmp = f.name
    try:
        az = run_azsim_gear_stats(args.spec, gear_tmp)
        dps_info = None
        if args.dps:
            d_spec = run_azsim_dps(args.spec, args.calib)
            d_gear = run_azsim_dps(args.spec, args.calib, gear_tmp)
            d_simc = float(sc_player["collected_data"]["dps"]["mean"])
            dps_info = (d_spec, d_gear, d_simc)
    finally:
        if os.path.exists(gear_tmp):
            os.unlink(gear_tmp)

    item_rows, bad_items = compare_items(az["items"], sc_gear)
    char_rows, bad_char = compare_char(az, sc_stats)
    print_tables(item_rows, char_rows, dps_info, args.md)
    if bad_items or bad_char:
        sys.exit(1)


if __name__ == "__main__":
    main()
