#!/usr/bin/env python3
"""Look up spells straight from the game's DB2 tables (CSV exports from wago.tools).

  python3 tools/spelldb.py <db2_dir> "Blade of Justice" 184575 ...

Prints, per spell: cooldown/charges, cost, duration, and every effect with its
type, aura, base points and attack-power coefficient. No SimC involved.
"""
import csv
import sys
from collections import defaultdict

csv.field_size_limit(10**8)


def load(d, name, key="SpellID", multi=False):
    out = defaultdict(list) if multi else {}
    with open(f"{d}/{name}.csv", newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            k = int(row[key])
            if multi:
                out[k].append(row)
            else:
                out.setdefault(k, row)
    return out


def num(x, default=0.0):
    try:
        return float(x)
    except (TypeError, ValueError):
        return default


def main():
    d, queries = sys.argv[1], sys.argv[2:]
    names = load(d, "SpellName", key="ID")
    by_name = defaultdict(list)
    for sid, row in names.items():
        by_name[row["Name_lang"].lower()].append(sid)
    effects = load(d, "SpellEffect", multi=True)
    misc = load(d, "SpellMisc")
    cds = load(d, "SpellCooldowns")
    cats = load(d, "SpellCategories")
    category = load(d, "SpellCategory", key="ID")
    power = load(d, "SpellPower", multi=True)
    auraopt = load(d, "SpellAuraOptions")
    durations = load(d, "SpellDuration", key="ID")

    ids = []
    for q in queries:
        if q.isdigit():
            ids.append(int(q))
        else:
            # only spells that actually do something (have effects)
            ids += [s for s in sorted(by_name.get(q.lower(), [])) if s in effects]

    for sid in ids:
        nm = names.get(sid, {}).get("Name_lang", "?")
        m = misc.get(sid, {})
        dur = durations.get(int(num(m.get("DurationIndex"))), {})
        cd = cds.get(sid, {})
        cat = cats.get(sid, {})
        charge_cat = category.get(int(num(cat.get("ChargeCategory"))), {})
        ao = auraopt.get(sid, {})
        print(f"=== {sid} {nm}")
        parts = []
        if num(cd.get("RecoveryTime")):
            parts.append(f"cd={num(cd['RecoveryTime'])/1000:g}s")
        if num(cd.get("CategoryRecoveryTime")):
            parts.append(f"cat_cd={num(cd['CategoryRecoveryTime'])/1000:g}s")
        if num(cd.get("StartRecoveryTime")):
            parts.append(f"gcd={num(cd['StartRecoveryTime'])/1000:g}s")
        if charge_cat:
            parts.append(f"charges={charge_cat.get('MaxCharges')} recharge={num(charge_cat.get('ChargeRecoveryTime'))/1000:g}s")
        if dur:
            parts.append(f"duration={num(dur.get('Duration'))/1000:g}s")
        if num(ao.get("CumulativeAura")):
            parts.append(f"max_stacks={ao['CumulativeAura']}")
        if num(ao.get("ProcChance")) and num(ao.get("ProcChance")) < 101:
            parts.append(f"proc_chance={ao['ProcChance']}%")
        if num(ao.get("ProcsPerMinute") if "ProcsPerMinute" in ao else 0):
            parts.append(f"ppm={ao['ProcsPerMinute']}")
        for p in power.get(sid, []):
            parts.append(f"cost={num(p.get('ManaCost'))-0:g} type={p.get('PowerType')}")
        if m.get("SchoolMask"):
            parts.append(f"school_mask={m['SchoolMask']}")
        if parts:
            print("   " + "  ".join(parts))
        for e in sorted(effects.get(sid, []), key=lambda r: (int(r["EffectIndex"]), int(r["DifficultyID"]))):
            trig = int(num(e.get("EffectTriggerSpell")))
            trig_s = f" trigger={trig}({names.get(trig, {}).get('Name_lang', '?')})" if trig else ""
            print(
                f"   #{e['EffectIndex']} effect={e['Effect']} aura={e['EffectAura']} "
                f"base={num(e['EffectBasePointsF']):g} ap={num(e['BonusCoefficientFromAP']):g} "
                f"sp={num(e['EffectBonusCoefficient']):g} misc={e['EffectMiscValue_0']}/{e['EffectMiscValue_1']} "
                f"period={num(e['EffectAuraPeriod'])/1000:g} "
                f"mask={e['EffectSpellClassMask_0']}/{e['EffectSpellClassMask_1']}/{e['EffectSpellClassMask_2']}/{e['EffectSpellClassMask_3']}"
                f"{trig_s}"
            )


if __name__ == "__main__":
    main()
