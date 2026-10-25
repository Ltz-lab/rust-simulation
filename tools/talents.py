#!/usr/bin/env python3
"""Decode a talent loadout string (the in-game export, also SimC's talents=)
into the talents it selects, straight from the DB2 trait tables.

  python3 tools/talents.py <db2_dir> <talent_string>

The string is base64 (A-Z a-z 0-9 + /), read as a bit stream, 6 bits per
character, least significant bit first:

  version 8, spec id 16, tree hash 128, then for every node of the class's
  trait tree in ascending node id order:
    selected 1 -> purchased 1 -> (partially ranked 1 -> ranks 6), (choice 1 -> entry 2)

A selected node that isn't purchased is granted (free, at full rank).
"""
import sys

from db2 import Db2, num

B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"


class Bits:
    def __init__(self, s):
        self.vals = [B64.index(c) for c in s]
        self.pos = 0

    def read(self, n):
        out = 0
        for i in range(n):
            c, b = divmod(self.pos, 6)
            out |= ((self.vals[c] >> b) & 1) << i
            self.pos += 1
        return out


def decode(db, s):
    """-> (spec_id, [dict(node, entry, definition, spell, ranks, max_ranks, granted)])"""
    bits = Bits(s)
    version = bits.read(8)
    spec = bits.read(16)
    for _ in range(16):
        bits.read(8)  # tree hash, not checked
    # a spec can have several loadout rows; the player talent tree is the one
    # tied to a class skill line
    class_trees = {int(r["TraitTreeID"]) for r in db.rows("SkillLineXTraitTree")}
    trees = {int(r["TraitTreeID"]) for r in db.multi("TraitTreeLoadout", "ChrSpecializationID").get(spec, [])}
    trees &= class_trees
    if len(trees) != 1:
        raise ValueError(f"spec {spec}: expected one class trait tree, found {sorted(trees)}")
    tree = trees.pop()
    node_rows = {int(r["ID"]): r for r in db.rows("TraitNode") if int(r["TraitTreeID"]) == tree}
    nodes = sorted(node_rows)
    node_entries = db.multi("TraitNodeXTraitNodeEntry", "TraitNodeID")
    entries = db.by("TraitNodeEntry")
    defs = db.by("TraitDefinition")

    out = []
    for node in nodes:
        if not bits.read(1):
            continue
        purchased = bits.read(1)
        ranks = None
        choice = 0
        if purchased:
            if bits.read(1):
                ranks = bits.read(6)
            if bits.read(1):
                choice = bits.read(2)
        ents = sorted(node_entries.get(node, []), key=lambda r: int(r["_Index"]))
        if not ents:
            continue
        e = entries[int(ents[min(choice, len(ents) - 1)]["TraitNodeEntryID"])]
        d = defs.get(int(e["TraitDefinitionID"]), {})
        max_ranks = int(e["MaxRanks"])
        out.append({
            "node": node,
            "entry": int(e["ID"]),
            "definition": int(e["TraitDefinitionID"]),
            "spell": int(num(d.get("SpellID"))),
            "ranks": max_ranks if ranks is None else ranks,
            "max_ranks": max_ranks,
            "granted": not purchased,
            "subtree": int(node_rows[node]["TraitSubTreeID"]),
            "selects": int(e["TraitSubTreeID"]) if not int(num(d.get("SpellID"))) else 0,
        })
    if bits.pos > len(bits.vals) * 6:
        raise ValueError("talent string ran out of bits: wrong tree or game build?")
    # hero talents: keep only the subtree the selection node picked (the
    # other hero tree's entry nodes still come through as granted)
    hero = {t["selects"] for t in out if t["selects"]}
    out = [t for t in out if not t["selects"] and (not t["subtree"] or t["subtree"] in hero)]
    return version, spec, out


def main():
    db = Db2(sys.argv[1])
    version, spec, talents = decode(db, sys.argv[2])
    print(f"version {version}  spec {spec}  {len(talents)} nodes selected")
    for t in talents:
        tag = "granted" if t["granted"] else f"{t['ranks']}/{t['max_ranks']}"
        sub = f" hero {t['subtree']}" if t["subtree"] else ""
        print(f"  node {t['node']:6d}  spell {t['spell']:7d}  {tag:7s}{sub}  {db.name(t['spell'])}")


if __name__ == "__main__":
    main()
