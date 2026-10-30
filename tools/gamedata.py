#!/usr/bin/env python3
"""Build data/gamedata.json.gz: everything `azsim --gear` needs to turn a
character's gear into stats, straight from the game's tables.

  python3 tools/gamedata.py [--build 12.1.5.70575] [--cache DIR] [--out data/gamedata.json.gz]

Downloads DB2 CSVs (wago.tools/db2/<Table>/csv) and game tables
(gametables/*.txt from CASC via wago.tools) into the cache dir, then writes a
compact gzip'd JSON file:

  items      every equippable armor/weapon item with stats:
             id -> [inventory_type, base_ilvl, quality, statset, sockets]
             (+ [subclass, delay_ms, dmg_variance] for weapons)
  statsets   deduplicated stat allocations [[stat_id, alloc], ...] (alloc in 1/10000ths of the budget)
  bonuses    bonus_id -> [[type, v0, v1, v2], ...], only the types gear->stats uses
             (1 ilvl add, 2 stat add, 6 socket, 25 crafted stat, 49 scaling config, 52 ilvl add after scaling);
             type 49's v0 is resolved to the scaling config's item level
  gems       gem item id -> [[stat_id, coeff, scaling_class, max_scaling_level], ...]
  enchants   enchant id   -> same shape (stat effects only)
  rand_prop_points   ilvl -> epic budget per slot type [major, minor, jewelry, one-hand, ranged]
  combat_rating_mult ilvl -> [armor, weapon, trinket, jewelry]
  stamina_mult       ilvl -> [armor, weapon, trinket, jewelry]
  weapon_dps         {"one_hand"|"two_hand": ilvl -> [dps per quality 0..6]}
  spell_scaling_item level -> the "Item" spell scaling column (scales gem and enchant values)
  combat_ratings     level -> rating per 1% for crit/haste/mastery/versatility
  rand_prop_damage   ilvl -> [damage_replace_stat, damage_secondary] (scaling classes -8, -9)
  item_effects       item id -> [[trigger_type, spell_id, cooldown_ms, category_cooldown_ms], ...]
                     (any equippable item: on-use 0 and on-equip 1 effects, plus chance-on-hit 2
                     effects of non-trinkets such as rings, necks and weapons)
  item_sets          item id -> item set id, for sets with bonuses
  set_bonuses        item set id -> [[pieces, spec_id (0 = any), spell_id], ...]
  spells             spell id -> {"duration", "school", "rppm", "rppm_mods", "icd", "chance",
                     "stacks", "cooldown", "effects": [[type, aura, base_points, coeff,
                     scaling_class, ap_coeff, sp_coeff, misc0, trigger], ...]} for every spell an
                     effect template names (data/item_effects.toml, `[[gear.effects]]` in specs)
                     and what those spells trigger
"""
import argparse
import csv
import glob
import re
import gzip
import json
import os
import urllib.request

csv.field_size_limit(10**9)
HERE = os.path.dirname(os.path.abspath(__file__))
DB2 = [
    "ItemSparse", "Item", "ItemBonus", "ItemScalingConfig", "RandPropPoints", "GemProperties",
    "SpellItemEnchantment", "ItemDamageOneHand", "ItemDamageTwoHand", "ItemEffect", "ItemXItemEffect",
    "ItemSet", "ItemSetSpell", "SpellEffect", "SpellMisc", "SpellDuration", "SpellAuraOptions",
    "SpellProcsPerMinute", "SpellProcsPerMinuteMod", "SpellCooldowns",
]
# CASC file data ids of the game tables we need.
GT = {"combatratingsmultbyilvl": 1391670, "staminamultbyilvl": 1980632, "combatratings": 1391669}
BONUS_TYPES = {1, 2, 6, 25, 49, 52}
# Inventory types that hold gear (skip shirts, tabards, bags, ammo, quivers, relics, non-equippable).
SKIP_INV = {0, 4, 18, 19, 24, 27, 28}
MAX_ILVL = 1300


def fetch(url, path):
    if not os.path.exists(path):
        print(f"fetching {url}")
        req = urllib.request.Request(url, headers={"User-Agent": "azsim-gamedata"})
        with urllib.request.urlopen(req, timeout=300) as r, open(path + ".tmp", "wb") as f:
            f.write(r.read())
        os.replace(path + ".tmp", path)
    return path


def rows(cache, build, table):
    p = fetch(f"https://wago.tools/db2/{table}/csv?build={build}", f"{cache}/{table}.csv")
    with open(p, newline="", encoding="utf-8") as f:
        yield from csv.DictReader(f)


def gametable(cache, build, name):
    p = fetch(f"https://wago.tools/api/casc/{GT[name]}?download&version={build}", f"{cache}/{name}.txt")
    lines = open(p, encoding="utf-8").read().splitlines()
    head = lines[0].split("\t")
    return head, [[float(x) for x in l.split("\t")] for l in lines[1:] if l.strip()]


def by_ilvl(table, width):
    out = [[0.0] * width for _ in range(MAX_ILVL + 1)]
    for r in table:
        if 0 <= int(r[0]) <= MAX_ILVL:
            out[int(r[0])] = r[1:1 + width]
    return out


def stat_effects(e):
    """Stat effects (effect type 5) of a SpellItemEnchantment row."""
    out = []
    for i in range(3):
        if e[f"Effect_{i}"] == "5":
            out.append([int(e[f"EffectArg_{i}"]), float(e[f"EffectScalingPoints_{i}"]),
                        int(e["ScalingClass"]), int(e["MaxLevel"]), int(e[f"EffectPointsMin_{i}"])])
    return out


def template_spells():
    """Spell ids the effect templates name: `spell = N` keys and `{N.field}` placeholders."""
    ids = set()
    for path in [os.path.join(HERE, "..", "data", "item_effects.toml"), *glob.glob(os.path.join(HERE, "..", "specs", "*.toml"))]:
        text = open(path, encoding="utf-8").read()
        ids.update(int(x) for x in re.findall(r"^\s*spell\s*=\s*(\d+)", text, re.M))
        ids.update(int(x) for x in re.findall(r"\{(\d+)\.", text))
    return ids


def spell_data(cache, B, wanted):
    """Compact spell records for `wanted` and every spell they trigger."""
    eff = {}
    for r in rows(cache, B, "SpellEffect"):
        if r["DifficultyID"] == "0":
            eff.setdefault(int(r["SpellID"]), []).append(r)
    misc = {int(r["SpellID"]): r for r in rows(cache, B, "SpellMisc") if r["DifficultyID"] == "0"}
    dur = {int(r["ID"]): int(r["Duration"]) for r in rows(cache, B, "SpellDuration")}
    aura = {int(r["SpellID"]): r for r in rows(cache, B, "SpellAuraOptions") if r["DifficultyID"] == "0"}
    rppm = {int(r["ID"]): float(r["BaseProcRate"]) for r in rows(cache, B, "SpellProcsPerMinute")}
    rmods = {}
    for r in rows(cache, B, "SpellProcsPerMinuteMod"):
        rmods.setdefault(int(r["SpellProcsPerMinuteID"]), []).append([int(r["Type"]), int(r["Param"]), float(r["Coeff"])])
    cds = {int(r["SpellID"]): r for r in rows(cache, B, "SpellCooldowns") if r["DifficultyID"] == "0"}
    out, todo = {}, sorted(wanted)
    while todo:
        sid = todo.pop()
        if sid in out or sid not in misc and sid not in eff:
            continue
        m, a, c = misc.get(sid, {}), aura.get(sid, {}), cds.get(sid, {})
        effects = []
        for e in sorted(eff.get(sid, []), key=lambda e: int(e["EffectIndex"])):
            trig = int(e["EffectTriggerSpell"])
            effects.append([int(e["Effect"]), int(e["EffectAura"]), round(float(e["EffectBasePointsF"]), 6),
                            round(float(e["Coefficient"]), 9), int(e["ScalingClass"]),
                            round(float(e["BonusCoefficientFromAP"]), 6), round(float(e["EffectBonusCoefficient"]), 6),
                            int(e["EffectMiscValue_0"]), trig])
            if trig:
                todo.append(trig)
        rid = int(a.get("SpellProcsPerMinuteID") or 0)
        out[sid] = {
            "duration": dur.get(int(m.get("DurationIndex") or 0), 0),
            "school": int(m.get("SchoolMask") or 0),
            "rppm": rppm.get(rid, 0.0),
            "rppm_mods": rmods.get(rid, []),
            "icd": int(a.get("ProcCategoryRecovery") or 0),
            "chance": float(a.get("ProcChance") or 0),
            "stacks": int(a.get("CumulativeAura") or 0),
            "cooldown": int(c.get("RecoveryTime") or 0),
            "effects": effects,
        }
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", default="12.1.5.70575")
    ap.add_argument("--cache", default=os.path.join(HERE, "..", "target", "gamedata-cache"))
    ap.add_argument("--out", default=os.path.join(HERE, "..", "data", "gamedata.json.gz"))
    ap.add_argument("--level", type=int, default=90)
    a = ap.parse_args()
    cache = os.path.join(a.cache, a.build)
    os.makedirs(cache, exist_ok=True)
    B = a.build

    cls = {int(r["ID"]): (int(r["ClassID"]), int(r["SubclassID"])) for r in rows(cache, B, "Item")}
    items, statsets, setidx, gem_enchant = {}, [], {}, {}
    for r in rows(cache, B, "ItemSparse"):
        iid, inv = int(r["ID"]), int(r["InventoryType"])
        c, sub = cls.get(iid, (-1, -1))
        if c == 3 and int(r["Gem_properties"] or 0):
            gem_enchant[iid] = int(r["Gem_properties"])
        if c not in (2, 4) or inv in SKIP_INV:
            continue
        stats = []
        for i in range(10):
            sid, alloc = int(r[f"StatModifier_bonusStat_{i}"]), int(r[f"StatPercentEditor_{i}"])
            if sid > 0 and alloc:
                stats.append([sid, alloc])
        sockets = sum(1 for i in range(3) if r[f"SocketType_{i}"] not in ("0", ""))
        if not stats and not sockets and c != 2:
            continue
        key = json.dumps(stats)
        if key not in setidx:
            setidx[key] = len(statsets)
            statsets.append(stats)
        items[iid] = [inv, int(r["ItemLevel"]), int(r["OverallQualityID"]), setidx[key], sockets]
        if c == 2:
            items[iid] += [sub, int(r["ItemDelay"]), round(float(r["DmgVariance"]), 4)]

    isc = {int(r["ID"]): int(r["ItemLevel"]) for r in rows(cache, B, "ItemScalingConfig")}
    bonuses = {}
    for r in sorted(rows(cache, B, "ItemBonus"), key=lambda r: (int(r["ParentItemBonusListID"]), int(r["OrderIndex"]))):
        t = int(r["Type"])
        if t not in BONUS_TYPES:
            continue
        v = [int(r[f"Value_{i}"]) for i in range(3)]
        if t == 49:
            v[0] = isc.get(v[0], 0)
        bonuses.setdefault(int(r["ParentItemBonusListID"]), []).append([t] + v)

    gp = {int(r["ID"]): int(r["Enchant_ID"]) for r in rows(cache, B, "GemProperties")}
    ench_rows = {int(r["ID"]): r for r in rows(cache, B, "SpellItemEnchantment")}
    enchants = {eid: s for eid, e in ench_rows.items() if (s := stat_effects(e))}
    gems = {iid: enchants[gp[p]] for iid, p in gem_enchant.items() if gp.get(p) in enchants}

    rpd = by_ilvl([[float(r["ID"]), float(r["DamageReplaceStatF"]), float(r["DamageSecondaryF"])] for r in rows(cache, B, "RandPropPoints")], 2)
    rpp = by_ilvl([[float(r["ID"])] + [float(r[f"EpicF_{i}"]) for i in range(5)] for r in rows(cache, B, "RandPropPoints")], 5)
    _, crm = gametable(cache, B, "combatratingsmultbyilvl")
    _, stm = gametable(cache, B, "staminamultbyilvl")
    head, cr = gametable(cache, B, "combatratings")
    col = {h: i for i, h in enumerate(head)}
    ratings = {}
    for r in cr:
        ratings[int(r[0])] = {
            "crit": r[col["Crit - Melee"]], "haste": r[col["Haste - Melee"]],
            "mastery": r[col["Mastery"]], "versatility": r[col["Versatility - Damage Done"]],
        }

    def dmg(table):
        out = [[0.0] * 7 for _ in range(MAX_ILVL + 1)]
        for r in rows(cache, B, table):
            il = int(r["ItemLevel"])
            if il <= MAX_ILVL:
                out[il] = [round(float(r[f"Quality_{i}"]), 4) for i in range(7)]
        return out

    # The "Item" column of the spell scaling table, from Raidbots' static export
    # of the client's SpellScaling game table (the CASC copy isn't on wago).
    sst = json.load(open(fetch("https://www.raidbots.com/static/data/live/spell-scaling-table.json", f"{cache}/spell-scaling-table.json")))
    item_col = next(c for c in sst if c["field"] == "Item")["data"]

    # Item on-use / on-equip / proc effects (any slot), set membership and set bonuses.
    ie = {int(r["ID"]): r for r in rows(cache, B, "ItemEffect")}
    item_effects = {}
    templated = template_spells()
    for r in rows(cache, B, "ItemXItemEffect"):
        iid, e = int(r["ItemID"]), ie.get(int(r["ItemEffectID"]))
        # Trinkets keep every on-use / on-equip effect (unmodelled ones are reported). Other slots
        # keep only effects with a template: thousands of ring/neck/weapon effects have none, and
        # listing them as unmodelled would send every swap of such an item to SimC.
        if e and iid in items and (e["TriggerType"] in ("0", "1") if items[iid][0] == 12 else
                                   e["TriggerType"] in ("0", "1", "2") and int(e["SpellID"]) in templated):
            item_effects.setdefault(iid, []).append(
                [int(e["TriggerType"]), int(e["SpellID"]), int(e["CoolDownMSec"]), int(e["CategoryCoolDownMSec"])])
    for v in item_effects.values():
        v.sort()
    set_bonuses = {}
    for r in rows(cache, B, "ItemSetSpell"):
        set_bonuses.setdefault(int(r["ItemSetID"]), []).append([int(r["Threshold"]), int(r["ChrSpecID"]), int(r["SpellID"])])
    for v in set_bonuses.values():
        v.sort()
    item_sets = {}
    for r in rows(cache, B, "ItemSparse"):
        iid, sid = int(r["ID"]), int(r["ItemSet"] or 0)
        if sid in set_bonuses and iid in items:
            item_sets[iid] = sid
    spells = spell_data(cache, B, template_spells())

    rnd = lambda t: [[round(x, 6) for x in row] for row in t]
    out = {
        "build": B,
        "items": items,
        "statsets": statsets,
        "bonuses": bonuses,
        "gems": gems,
        "enchants": enchants,
        "rand_prop_points": rnd(rpp),
        "combat_rating_mult": rnd(by_ilvl(crm, 4)),
        "stamina_mult": rnd(by_ilvl(stm, 4)),
        "weapon_dps": {"one_hand": dmg("ItemDamageOneHand"), "two_hand": dmg("ItemDamageTwoHand")},
        "spell_scaling_item": [0.0] + [round(float(x), 6) for x in item_col],
        "combat_ratings": ratings,
        "rand_prop_damage": rnd(rpd),
        "item_effects": dict(sorted(item_effects.items())),
        "item_sets": dict(sorted(item_sets.items())),
        "set_bonuses": dict(sorted(set_bonuses.items())),
        "spells": dict(sorted(spells.items())),
    }
    data = json.dumps(out, separators=(",", ":")).encode()
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with gzip.GzipFile(a.out, "wb", compresslevel=9, mtime=0) as f:
        f.write(data)
    print(f"wrote {a.out}: {len(items)} items, {len(statsets)} stat sets, {len(bonuses)} bonuses, "
          f"{len(gems)} gems, {len(enchants)} enchants, {len(item_effects)} items with effects, "
          f"{len(item_sets)} set pieces, {len(spells)} spells; {len(data)/1e6:.1f} MB json, {os.path.getsize(a.out)/1e3:.0f} KB gz")


if __name__ == "__main__":
    main()
