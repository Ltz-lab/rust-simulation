#!/usr/bin/env python3
"""Turn a SimC json2 report into an observations file.

SimC is used here as a stand-in for real combat logs: the same numbers
(casts per fight, average hit, crit rate, buff uptime) can be mined from
Warcraft Logs.

  python3 tools/observe.py report.json > observations.json
"""
import json
import sys


def mean(d, *path, default=0.0):
    for k in path:
        if not isinstance(d, dict) or k not in d:
            return default
        d = d[k]
    return d.get("mean", default) if isinstance(d, dict) else d


def main():
    rep = json.load(open(sys.argv[1]))
    p = rep["sim"]["players"][0]
    dps = p["collected_data"]["dps"]["mean"]
    length = rep["sim"]["statistics"]["simulation_length"]["mean"]
    sources = {}

    def walk(stats, prefix=""):
        for s in stats:
            if s.get("type") == "damage":
                dr, tr = s.get("direct_results", {}), s.get("tick_results", {})
                d_hit = dr.get("hit", {}).get("count", {}).get("mean", 0.0)
                d_crit = dr.get("crit", {}).get("count", {}).get("mean", 0.0)
                t_hit = tr.get("hit", {}).get("count", {}).get("mean", 0.0)
                t_crit = tr.get("crit", {}).get("count", {}).get("mean", 0.0)
                amount = mean(s, "actual_amount")
                if amount > 0:
                    sources[prefix + s["name"]] = {
                        "spell_id": s.get("id"),
                        "school": s.get("school"),
                        "dps": amount / length,
                        "share": s.get("portion_amount", 0.0),
                        "executes": mean(s, "num_executes"),
                        "direct_hits": d_hit + d_crit,
                        "direct_noncrit_avg": mean(dr, "hit", "actual_amount"),
                        "direct_crit_rate": d_crit / (d_hit + d_crit) if d_hit + d_crit else 0.0,
                        "ticks": t_hit + t_crit,
                        "tick_noncrit_avg": mean(tr, "hit", "actual_amount"),
                        "tick_crit_rate": t_crit / (t_hit + t_crit) if t_hit + t_crit else 0.0,
                    }
            walk(s.get("children", []), prefix)

    walk(p["stats"])
    # pets (e.g. a summoned phoenix) as "<pet>_<spell>"
    for pet, stats in p.get("stats_pets", {}).items():
        walk(stats, pet + "_")
    buffs = {
        b["name"]: {
            "spell_id": b.get("spell"),
            "uptime": b.get("uptime", 0.0) / 100.0,
            "starts": b.get("start_count", 0.0),
            "refreshes": b.get("refresh_count", 0.0),
            "interval": b.get("interval", 0.0),
            "duration": b.get("duration", 0.0),
        }
        for b in p.get("buffs", [])
    }
    stats = p["collected_data"].get("buffed_stats", {})
    out = {
        "player": p["name"],
        "dps": dps,
        "fight_length": length,
        "buffed_stats": {k: stats.get(k) for k in ("attributes", "stats")} if stats else {},
        "sources": dict(sorted(sources.items(), key=lambda kv: -kv[1]["dps"])),
        "buffs": buffs,
    }
    json.dump(out, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
