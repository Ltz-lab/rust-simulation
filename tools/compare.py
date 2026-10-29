#!/usr/bin/env python3
"""Side-by-side comparison of an azsim result against observations.

  python3 tools/compare.py observations.json azsim_result.json [--md]

Columns: DPS per source for both, and hits per fight (direct hits plus ticks).
Hit counts are the structural test — calibration only fixes damage per hit,
so a wrong count means a wrong mechanic or rotation.
"""
import json
import sys


def main():
    obs = json.load(open(sys.argv[1]))
    ours = json.load(open(sys.argv[2]))
    md = "--md" in sys.argv
    mine = {s["name"]: s for s in ours["sources"]}
    names = list(obs["sources"].keys()) + [n for n in mine if n not in obs["sources"]]
    rows = []
    for n in names:
        o = obs["sources"].get(n)
        m = mine.get(n)
        o_hits = (o["direct_hits"] + o["ticks"]) if o else 0.0
        m_hits = m["hits_per_fight"] if m else 0.0
        od, mdps = (o["dps"] if o else 0.0), (m["dps"] if m else 0.0)
        if od == 0 and mdps == 0:
            continue  # bookkeeping abilities that deal no damage
        rows.append((n, od, mdps, o_hits, m_hits))
    rows.sort(key=lambda r: -max(r[1], r[2]))
    tot_o, tot_m = obs["dps"], ours["dps"]
    if md:
        print("| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |")
        print("|---|---:|---:|---:|---:|---:|")
    else:
        print(f"{'source':38s} {'ref dps':>9} {'azsim':>9} {'diff':>7}   {'ref hits':>8} {'azsim':>8}")
    for n, od, mdps, oh, mh in rows:
        diff = (mdps / od - 1) * 100 if od else float("nan")
        if md:
            print(f"| {n} | {od:,.0f} | {mdps:,.0f} | {diff:+.1f}% | {oh:.1f} | {mh:.1f} |")
        else:
            flag = "  <--" if oh and abs(mh / oh - 1) > 0.10 and od > 0.005 * tot_o else ""
            print(f"{n:38s} {od:9.0f} {mdps:9.0f} {diff:+6.1f}%   {oh:8.1f} {mh:8.1f}{flag}")
    d = (tot_m / tot_o - 1) * 100
    if md:
        print(f"| **total** | **{tot_o:,.0f}** | **{tot_m:,.0f}** | **{d:+.2f}%** | | |")
    else:
        print(f"\n{'TOTAL':38s} {tot_o:9.0f} {tot_m:9.0f} {d:+6.2f}%")


if __name__ == "__main__":
    main()
