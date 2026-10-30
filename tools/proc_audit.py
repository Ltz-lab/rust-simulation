#!/usr/bin/env python3
"""Which gear procs does a spec's `[gear.procs]` / `[player]` actually carry?

For each case (SimC profile, azsim spec) it runs SimC on the profile without its
trinkets and with them, with SimC's per-second stat timelines on, and prints how
far the time-averaged ratings sit above the gear's own ratings:

  none   procs of the non-trinket gear (enchants, weapon, neck, rings, embellishments)
  trk    what the trinkets add on top

next to the spec's `[gear.procs]` (percent / power units). A spec that bakes in
the sample gear's trinket procs has proc values near `none + trk`; one that
follows the trinket-free character has them near `none`.

    python3 tools/proc_audit.py --simc /path/to/simc \\
        --case profiles/MID2_Mage_Fire.simc:fire_mage ...
"""
import argparse, json, os, subprocess, sys, tempfile, tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
STATS = "crit_rating,haste_rating,mastery_rating,versatility_rating"
CORE = ["potion=disabled", "flask=disabled", "food=disabled", "augmentation=disabled",
        "temporary_enchant=disabled", "set_bonus=midnight_season_2_2pc=0",
        "set_bonus=midnight_season_2_4pc=0", "set_bonus=bite_of_zuljan_2pc=0", "override.bloodlust=0"]


def profile(src, keep_trinkets):
    lines = open(src).read().splitlines()
    if not keep_trinkets:
        lines = [l for l in lines if not l.lstrip().startswith(("trinket1=", "trinket2="))]
    return "\n".join(lines) + "\n"


def timeline_means(simc, text, n):
    """Mean of each stat's per-second timeline, skipping t=0 (stats are not applied yet)."""
    with tempfile.TemporaryDirectory() as d:
        p, j = os.path.join(d, "p.simc"), os.path.join(d, "o.json")
        open(p, "w").write(text)
        subprocess.run([simc, p, "fight_style=Patchwerk", "max_time=300", "vary_combat_length=0",
                        f"iterations={n}", "threads=4", "report_details=1", f"stat_timelines={STATS}",
                        "json2=" + j, *CORE], check=True, capture_output=True)
        cd = json.load(open(j))["sim"]["players"][0]["collected_data"]["stat_timelines"]
    out = {}
    for k, v in cd.items():
        v = v if isinstance(v, list) else v["data"]
        out[k] = sum(v[1:290]) / 289
    return out


def gear_ratings(azsim, spec, text):
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "p.simc")
        open(p, "w").write(text)
        args = [azsim, spec, "--gear", p, "--gear-stats", "--set", "gear.trinket_effects=0",
                "--set", "gear.set_bonuses=0"]
        if "[gear.kit]" in open(spec).read():
            args += ["--set", "gear.kit.enabled=0"]
        return json.loads(subprocess.check_output(args, text=True))["gear"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--simc", required=True)
    ap.add_argument("--azsim", default=os.path.join(ROOT, "target", "release", "azsim"))
    ap.add_argument("--case", action="append", required=True, help="profile.simc:azsim_spec")
    ap.add_argument("-n", type=int, default=600)
    a = ap.parse_args()
    for c in a.case:
        path, name = c.split(":")
        spec = os.path.join(ROOT, "specs", name + ".toml")
        procs = tomllib.load(open(spec, "rb")).get("gear", {}).get("procs", {})
        none_t, full_t = profile(path, False), profile(path, True)
        base = gear_ratings(a.azsim, spec, none_t)
        m0, m1 = timeline_means(a.simc, none_t, a.n), timeline_means(a.simc, full_t, a.n)
        cols = " | ".join(f"{k[:-7]} none {m0[k] - base[k]:+5.0f} trk {m1[k] - m0[k]:+5.0f}" for k in STATS.split(","))
        print(f"{name}: {cols}\n    [gear.procs] {procs or '-'}")


if __name__ == "__main__":
    sys.exit(main())
