#!/usr/bin/env python3
"""Omnium Folio traits (`omnium_talents=` in a SimC profile): what they are and
what each is worth.

  python3 tools/omnium.py traits [DB2_DIR] [PROFILE ...]
  python3 tools/omnium.py ablate PROFILE --simc /path/to/simc [--iterations N] [--prefix ret]

`traits` maps each trait id (a TraitNodeEntry id, as SimC reads it) to its
definition spell in the DB2 tables (default target/db2, from tools/db2.py), and
lists which profiles use it.

`ablate` needs a SimC binary. It runs the profile's core kit (as
tools/make_reference.sh builds it) with every trait on, with each one removed,
and with none, then prints the DPS table and a `[gear.omnium]` block for the
spec. Per-trait values come from leave-one-out ratios r_i = all / without_i.
Traits interact (in SimC, Overload, Lingering and Burning Haste only act
through Rune of Unleashed Fire), so the product of the r_i is not the total;
they are rescaled by one exponent so the product equals all / none exactly,
which keeps the reference character unchanged and a character without any trait
exact. Use a high iteration count: a trait worth 0.3% needs the error below 0.1%.
"""
import csv
import glob
import json
import os
import re
import subprocess
import sys
import tempfile
from math import exp, log

HERE = os.path.dirname(os.path.abspath(__file__))
STRIP = re.compile(r"^(trinket[12]|potion|flask|food|augmentation|temporary_enchant)=")
CORE_TAIL = """potion=disabled
flask=disabled
food=disabled
augmentation=disabled
temporary_enchant=disabled
set_bonus=midnight_season_2_2pc=0/midnight_season_2_4pc=0/bite_of_zuljan_2pc=0
override.bloodlust=0
"""


def traits_of(profile_text):
    for line in profile_text.splitlines():
        k, _, v = line.partition("=")
        if k.strip() == "omnium_talents":
            return [t.split(":")[0].strip() for t in v.strip().split("/") if t.strip()]
    return []


def rows(path):
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def trait_table(d):
    entries = {r["ID"]: r for r in rows(f"{d}/TraitNodeEntry.csv")}
    defs = {r["ID"]: r for r in rows(f"{d}/TraitDefinition.csv")}
    names = {r["ID"]: r["Name_lang"] for r in rows(f"{d}/SpellName.csv")}
    out = {}
    for tid, e in entries.items():
        spell = defs.get(e["TraitDefinitionID"], {}).get("SpellID")
        if spell and names.get(spell, "").startswith("Rune of"):
            out[tid] = (spell, names[spell])
    return out


def cmd_traits(args):
    d = args[0] if args and os.path.isdir(args[0]) else os.path.join(HERE, "..", "target", "db2")
    profiles = [a for a in args if os.path.isfile(a)] or sorted(glob.glob(os.path.join(HERE, "..", "profiles", "*.simc")))
    used = {}
    for p in profiles:
        for t in traits_of(open(p).read()):
            used.setdefault(t, []).append(os.path.basename(p).replace("MID2_", "").replace(".simc", ""))
    for tid, (spell, name) in sorted(trait_table(d).items(), key=lambda kv: int(kv[0])):
        if tid in used or 136814 <= int(tid) <= 136823:
            print(f"{tid}  {name:32s} spell {spell:>8s}  used by {len(used.get(tid, []))} profiles")


def simc_dps(simc, text, iterations, tag):
    with tempfile.TemporaryDirectory() as w:
        prof, out = os.path.join(w, "p.simc"), os.path.join(w, "o.json")
        open(prof, "w").write(text)
        subprocess.run([simc, prof, f"iterations={iterations}", f"threads={os.cpu_count()}", f"json2={out}"],
                       check=True, stdout=subprocess.DEVNULL)
        dps = json.load(open(out))["sim"]["players"][0]["collected_data"]["dps"]
        print(f"  {tag:34s} {dps['mean']:>10,.0f}  (+/- {dps.get('mean_std_dev', 0) * 1.96:,.0f})", file=sys.stderr)
        return dps["mean"]


def solve(all_dps, none_dps, without):
    """Leave-one-out ratios, rescaled so their product is all / none."""
    r = {t: all_dps / v for t, v in without.items()}
    total = log(all_dps / none_dps)
    s = sum(log(v) for v in r.values())
    k = total / s if s else 0.0
    return {t: exp(k * log(v)) for t, v in r.items()}, total


def cmd_ablate(args):
    simc = args[args.index("--simc") + 1]
    iters = int(args[args.index("--iterations") + 1]) if "--iterations" in args else 20000
    profile = args[0]
    lines = [l for l in open(profile).read().splitlines() if not STRIP.match(l) and not l.startswith("omnium_talents")]
    traits = traits_of(open(profile).read())
    if not traits:
        sys.exit("profile has no omnium_talents line")

    def text(ts):
        return "\n".join(lines + ([f"omnium_talents={'/'.join(ts)}"] if ts else [])) + "\n" + CORE_TAIL

    full = [t for t in open(profile).read().splitlines() if t.startswith("omnium_talents=")][0].split("=", 1)[1].split("/")
    keys = [t.split(":")[0] for t in full]
    print(f"{profile}: {iters} iterations", file=sys.stderr)
    all_dps = simc_dps(simc, text(full), iters, "all traits")
    none_dps = simc_dps(simc, text([]), iters, "no traits")
    without = {k: simc_dps(simc, text([t for t in full if t.split(":")[0] != k]), iters, f"without {k}") for k in keys}
    mult, total = solve(all_dps, none_dps, without)
    print(f"# all {all_dps:,.0f}, none {none_dps:,.0f}: traits are worth {exp(total) - 1:+.2%}")
    print("[gear.omnium]")
    print("baseline = [" + ", ".join(f'"{k}"' for k in keys) + "]")
    print("traits = [")
    for k in keys:
        print(f'  {{ id = "{k}", name = "", global_mult = {mult[k]:.5f} }},   # [simc] leave-one-out {all_dps / without[k]:.5f}')
    print("]")


if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] not in ("traits", "ablate"):
        sys.exit(__doc__)
    {"traits": cmd_traits, "ablate": cmd_ablate}[sys.argv[1]](sys.argv[2:])
