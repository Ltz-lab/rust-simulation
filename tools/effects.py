#!/usr/bin/env python3
"""Derive per-source damage multipliers from the game's own spell modifiers.

  python3 tools/effects.py <db2_dir> <spec.toml> <observations.json> <talents> [--out derived.toml] [--explain SOURCE]

<talents> is a loadout string or a .simc profile with a talents= line.

Calibration (tools/calibrate.py) fits a residual multiplier per damage source
that stands in for every talent and aura we don't model. Most of that is
plain data: passive spells whose effects add a percentage to other spells'
damage. This tool reads them:

  1. active passives: the spec's own spells (SpecializationSpells), class
     spells learned by skill line, and the talents in the loadout string
     (tools/talents.py), keeping only spells with the passive attribute.
  2. every effect of those that is a spell modifier:
       aura 107/108  flat/percent modifier, targets by spell family mask
       aura 219/218  flat/percent modifier, targets by spell label
     on the ops that scale damage: 0 damage, 22 periodic damage,
     3/12/23/32/33 points of effect index 0..4, 8 points of every effect.
     plus aura 79, percent damage done to a school.
  3. for each damage source, multiply the percent modifiers that hit the
     spell that actually dealt it (its id in the observations, as a combat
     log records it). Talent ranks scale the points.
  4. buffs the spec models get the same treatment while they're up: the
     buff's spell (id from the observations) adds `ability_pct` /
     `school_pct` to the sources it modifies, unless the spec already sets
     its damage by hand (then the derived value is printed next to the
     spec's, to check or migrate it).
  5. target debuffs (aura 271, "damage taken from the caster's spells", e.g.
     Judgment +20%): looked for among the spells the sources trigger or
     apply and the spells in the observations' buff list, plus `--debuff
     SPELLID`. They are reported per source and written as `debuff` in the
     output; `--fold-debuffs` multiplies them into `derived`, scaled by
     `--debuff-uptime` (default 1: the debuff is assumed always up).
  6. talent rank curves (TraitDefinitionEffectPoints + CurvePoint), when
     those tables are present, replace the linear "base x ranks" scaling for
     the effects they name. Scripted aura-4 dummies on the active passives
     are listed (not applied; their numbers live in script).

The output feeds calibrate.py (`--derived`), which writes it into the
calibration file next to the residual it fits on top. Modifiers it can't
interpret (scripted dummies, procs, buffs the spec doesn't model) stay in
the residual; `--explain` lists what was applied to one source.
It also flags damage coefficients in the spec that differ from the game data.
"""
import argparse
import json
import os
import sys
import tomllib
from collections import defaultdict

from db2 import Db2, num
from talents import decode

PASSIVE = 0x40            # SpellMisc Attributes_0
PCT_MASK, FLAT_MASK = 108, 107
PCT_LABEL, FLAT_LABEL = 218, 219
OP_DAMAGE, OP_PERIODIC, OP_POINTS = 0, 22, 8
OP_POINTS_INDEX = {3: 0, 12: 1, 23: 2, 32: 3, 33: 4}
APPLY_AURA = {6, 35, 119, 128, 129, 143, 174}   # aura-applying effect types
# effects that deal damage: school damage, weapon damage variants, health leech, periodic damage aura
DAMAGE_EFFECTS = {2, 9, 17, 31, 58, 121}
PERIODIC_DAMAGE_AURAS = {3, 53, 89}
MOD_DAMAGE_DONE = 79      # misc0: school mask
MOD_DAMAGE_FROM_CASTER = 271   # target debuff: percent damage taken from the caster's spells
DUMMY = 4                 # scripted aura; the value lives in script, not in data
OP_ALL = -1               # debuffs scale whatever damage the matched spell does
TRIGGER_EFFECTS = {64, 140}   # trigger spell / trigger spell with value


class Spells:
    def __init__(self, db):
        self.db = db
        self.effects = db.multi("SpellEffect", "SpellID")
        self.misc = db.by("SpellMisc", "SpellID")
        self.classopt = db.by("SpellClassOptions", "SpellID")
        self.labels = defaultdict(set)
        for r in db.rows("SpellLabel"):
            self.labels[int(r["SpellID"])].add(int(r["LabelID"]))

    def effs(self, sid):
        """Base-difficulty effects, by index."""
        out = {}
        for e in self.effects.get(sid, []):
            i = int(e["EffectIndex"])
            if int(e["DifficultyID"]) == 0 or i not in out:
                out[i] = e
        return [out[i] for i in sorted(out)]

    def passive(self, sid):
        m = self.misc.get(sid)
        return m is not None and int(num(m["Attributes_0"])) & PASSIVE != 0

    def family(self, sid):
        o = self.classopt.get(sid)
        if not o:
            return 0, (0, 0, 0, 0)
        return int(o["SpellClassSet"]), tuple(int(num(o[f"SpellClassMask_{i}"])) & 0xFFFFFFFF for i in range(4))

    def damage_indices(self, sid):
        """(index, periodic) of the effects that deal damage."""
        out = []
        for e in self.effs(sid):
            t, aura = int(e["Effect"]), int(e["EffectAura"])
            if t in DAMAGE_EFFECTS:
                out.append((int(e["EffectIndex"]), False))
            elif t in APPLY_AURA and aura in PERIODIC_DAMAGE_AURAS:
                out.append((int(e["EffectIndex"]), True))
        return out


def active_passives(db, sp, spec_id, talents):
    """spell id -> (rank multiplier, origin) for every passive in effect."""
    out = {}
    for r in db.multi("SpecializationSpells", "SpecID").get(spec_id, []):
        out[int(r["SpellID"])] = (1, "spec")
    spec = db.by("ChrSpecialization")[spec_id]
    for m in ("MasterySpellID_0",):
        if int(num(spec.get(m))):
            out[int(spec[m])] = (1, "mastery")
    for t in talents:
        if t["spell"]:
            out[t["spell"]] = (t["ranks"], "talent")
    return {s: v for s, v in out.items() if sp.passive(s)}


class RankCurves:
    """TraitDefinitionEffectPoints: per talent definition and effect index, a
    curve of rank -> points. OperationType 0 sets the effect's points, 1
    multiplies them. Absent tables mean no curves (linear scaling stays)."""

    def __init__(self, db):
        self.by_def = defaultdict(dict)
        self.points = defaultdict(list)
        try:
            for r in db.rows("TraitDefinitionEffectPoints"):
                self.by_def[int(num(r["TraitDefinitionID"]))][int(num(r["EffectIndex"]))] = (
                    int(num(r["OperationType"])), int(num(r["CurveID"])))
            for r in db.rows("CurvePoint"):
                self.points[int(num(r["CurveID"]))].append(
                    (int(num(r["OrderIndex"])), num(r["Pos_0"]), num(r["Pos_1"])))
        except FileNotFoundError:
            self.by_def.clear()
        for v in self.points.values():
            v.sort()

    def value(self, definition, index, rank, base):
        """Points of effect `index` at `rank`, or None if no curve covers it."""
        c = self.by_def.get(definition, {}).get(index)
        pts = self.points.get(c[1]) if c else None
        if not pts:
            return None
        xs = [(x, y) for _, x, y in pts]
        if rank <= xs[0][0]:
            y = xs[0][1]
        elif rank >= xs[-1][0]:
            y = xs[-1][1]
        else:
            y = xs[-1][1]
            for (x0, y0), (x1, y1) in zip(xs, xs[1:]):
                if x0 <= rank <= x1:
                    y = y0 + (y1 - y0) * (rank - x0) / (x1 - x0) if x1 != x0 else y1
                    break
        return y if c[0] == 0 else base * y


def scaled(curves, definitions, sid, e, ranks, base):
    """Points at `ranks`: the talent's rank curve if it has one, else linear."""
    if curves and definitions and sid in definitions:
        v = curves.value(definitions[sid], int(e["EffectIndex"]), ranks, base)
        if v is not None:
            return v
    return base * ranks   # multi-rank talents scale linearly otherwise


def modifiers(sp, passives, curves=None, definitions=None):
    """All damage-scaling spell modifiers from {spell: (ranks, origin)}.
    `curves` (RankCurves) and `definitions` ({spell: trait definition id})
    replace the linear rank scaling for talents that have a rank curve."""
    mods = []
    for sid, (ranks, origin) in passives.items():
        for e in sp.effs(sid):
            if int(e["Effect"]) not in APPLY_AURA:
                continue
            aura = int(e["EffectAura"])
            if aura == MOD_DAMAGE_DONE:
                base = num(e["EffectBasePointsF"])
                if base:
                    mods.append({"spell": sid, "origin": origin, "index": int(e["EffectIndex"]), "pct": True,
                                 "school": int(num(e["EffectMiscValue_0"])), "label": None, "op": None,
                                 "value": scaled(curves, definitions, sid, e, ranks, base)})
                continue
            if aura not in (PCT_MASK, FLAT_MASK, PCT_LABEL, FLAT_LABEL):
                continue
            op = int(num(e["EffectMiscValue_0"]))
            if op not in (OP_DAMAGE, OP_PERIODIC, OP_POINTS) and op not in OP_POINTS_INDEX:
                continue
            base = num(e["EffectBasePointsF"])
            if base == 0:
                continue
            mods.append({
                "spell": sid,
                "origin": origin,
                "index": int(e["EffectIndex"]),
                "school": None,
                "pct": aura in (PCT_MASK, PCT_LABEL),
                "label": int(num(e["EffectMiscValue_1"])) if aura in (PCT_LABEL, FLAT_LABEL) else None,
                "mask": tuple(int(num(e[f"EffectSpellClassMask_{i}"])) & 0xFFFFFFFF for i in range(4)),
                "op": op,
                "value": scaled(curves, definitions, sid, e, ranks, base),
            })
    return mods


def applies(sp, mod, target, index, periodic):
    """Does `mod` scale effect `index` of spell `target`?"""
    if mod["school"] is not None:
        m = sp.misc.get(target)
        return m is not None and int(num(m["SchoolMask"])) & mod["school"] != 0
    if mod["label"] is not None:
        if mod["label"] not in sp.labels.get(target, ()):
            return False
    else:
        fam, mask = sp.family(target)
        mod_fam, _ = sp.family(mod["spell"])
        if fam == 0 or (mod_fam and fam != mod_fam):
            return False
        if not any(a & b for a, b in zip(mask, mod["mask"])):
            return False
    op = mod["op"]
    if op == OP_ALL:
        return True
    if op == OP_DAMAGE:
        return not periodic
    if op == OP_PERIODIC:
        return periodic
    if op == OP_POINTS:
        return True
    return OP_POINTS_INDEX[op] == index


def derive(sp, mods, target, periodic_hint):
    """-> (multiplier, [applied mods]) for one source spell."""
    idx = sp.damage_indices(target)
    if periodic_hint is not None:
        idx = [i for i in idx if i[1] == periodic_hint] or idx
    if not idx:
        return 1.0, []
    index, periodic = idx[0]
    mult, used = 1.0, []
    for m in mods:
        if m["pct"] and applies(sp, m, target, index, periodic):
            mult *= 1 + m["value"] / 100
            used.append(m)
    return mult, used


def coefficient(sp, target, periodic):
    """(ap, sp) coefficient of the spell's damage effect."""
    for e in sp.effs(target):
        t, aura = int(e["Effect"]), int(e["EffectAura"])
        if (periodic and t in APPLY_AURA and aura in PERIODIC_DAMAGE_AURAS) or (not periodic and t in DAMAGE_EFFECTS):
            return num(e["BonusCoefficientFromAP"]), num(e["EffectBonusCoefficient"])
    return None


def all_abilities(spec):
    """Owner abilities, then the pets' named as the engine reports them (`<pet>_<ability>`)."""
    yield from spec.get("abilities", [])
    for p in spec.get("pets", []):
        pre = p["name"] + "_"
        for a in p.get("abilities", []):
            q = dict(a, name=pre + a["name"])
            if "source" in a:
                q["source"] = pre + a["source"]
            if "dot" in a:
                q["dot"] = dict(a["dot"], name=pre + a["dot"]["name"])
            yield q


def sources(spec, obs):
    """source name -> (spell id, periodic, spec damage def). The spell id is
    the one the observations recorded; the spec's `spell_id` is the fallback."""
    out = {}
    for a in all_abilities(spec):
        if a.get("damage"):
            name = a.get("source", a["name"])
            o = obs["sources"].get(name)
            sid = o["spell_id"] if o else a.get("spell_id")
            if sid and name not in out:
                out[name] = (sid, bool(o and o["ticks"] and not o["direct_hits"]), a["damage"])
        d = a.get("dot")
        if d:
            o = obs["sources"].get(d["name"])
            if o and d["name"] not in out:
                out[d["name"]] = (o["spell_id"], True, d)
    return out


HAND_SET = ("damage_pct", "ability_pct", "school_pct")


def buff_patches(db, sp, spec, obs, srcs):
    """buff name -> ({source: pct}, [(source, pct, mod)]) from the buff spells' own modifiers."""
    out = {}
    for b in spec.get("buffs", []):
        if not b.get("derive_damage", True):
            continue
        o = obs["buffs"].get(b["name"])
        sid = b.get("spell_id") or (o and o["spell_id"])
        if not sid:
            continue
        if any(b.get(k) for k in HAND_SET):
            print(f"buff {b['name']}: damage set by hand in the spec, not derived", file=sys.stderr)
        mods = modifiers(sp, {sid: (1, "buff")})
        mods += debuff_mods(sp, [sid])[0]     # a target debuff the spec models as a buff (Judgment)
        pct = {}
        for name, (target, periodic, _) in srcs.items():
            _, used = derive(sp, mods, target, periodic)
            total = 1.0
            for m in used:
                total *= 1 + m["value"] / 100
            if used:
                pct[name] = (total - 1) * 100
        if any(b.get(k) for k in HAND_SET):
            for k in HAND_SET:
                if b.get(k):
                    print(f"  hand-set {k}={b[k]!r}; derived from {sid}: {pct or 'nothing'}", file=sys.stderr)
            continue
        if pct:
            out[b["name"]] = pct
    return out


def debuff_mods(sp, spell_ids):
    """Aura-271 target debuffs on `spell_ids` as percent modifiers restricted
    by class mask; those with an empty mask are returned apart (not applied)."""
    mods, unresolved = [], []
    for sid in spell_ids:
        for e in sp.effs(sid):
            if int(e["Effect"]) not in APPLY_AURA or int(e["EffectAura"]) != MOD_DAMAGE_FROM_CASTER:
                continue
            base = num(e["EffectBasePointsF"])
            mask = tuple(int(num(e[f"EffectSpellClassMask_{i}"])) & 0xFFFFFFFF for i in range(4))
            m = {"spell": sid, "origin": "debuff", "index": int(e["EffectIndex"]), "pct": True, "school": None,
                 "label": None, "mask": mask, "op": OP_ALL, "value": base}
            (mods if any(mask) and base else unresolved).append(m)
    return mods, unresolved


def candidate_debuffs(sp, srcs, obs, extra):
    """Spells that may carry a target debuff: those the sources trigger or
    apply, the observations' buffs, and explicit ids."""
    out = set(extra)
    for target, _, _ in srcs.values():
        out.add(target)
        for e in sp.effs(target):
            t = int(num(e.get("EffectTriggerSpell")))
            if int(e["Effect"]) in TRIGGER_EFFECTS and t:
                out.add(t)
    out.update(b["spell_id"] for b in obs["buffs"].values() if b.get("spell_id"))
    return out


SCRIPTED = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "scripted_effects.toml")


def scripted_effects(sp, passives, path=SCRIPTED):
    """Aura-4 dummies whose damage effect is known from SimC's class module
    (data/scripted_effects.toml), for the passives in effect: a list of
    {spell, index, value (percent, rank-scaled), sources (spell ids), note}.
    The number still comes from the effect's points in DB2; the file only
    says which spells it multiplies."""
    if not os.path.exists(path):
        return []
    out = []
    for r in tomllib.load(open(path, "rb")).get("effect", []):
        if r["spell"] not in passives:
            continue
        effs = {int(e["EffectIndex"]): e for e in sp.effs(r["spell"])}
        e = effs.get(r["index"])
        if e is None or int(e["EffectAura"]) != DUMMY:
            continue
        out.append({"spell": r["spell"], "index": r["index"], "sources": set(r["sources"]), "note": r.get("simc", ""),
                    "value": num(e["EffectBasePointsF"]) * passives[r["spell"]][0]})
    return out


def dummy_report(sp, passives, applied=()):
    """(spell, effect index, points) of aura-4 dummies on active passives: scripted, not derivable."""
    return [(sid, int(e["EffectIndex"]), num(e["EffectBasePointsF"]))
            for sid in sorted(passives) for e in sp.effs(sid)
            if int(e["Effect"]) in APPLY_AURA and int(e["EffectAura"]) == DUMMY
            and (sid, int(e["EffectIndex"])) not in {(x["spell"], x["index"]) for x in applied}]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("db2")
    ap.add_argument("spec")
    ap.add_argument("observations")
    ap.add_argument("talents")
    ap.add_argument("--out")
    ap.add_argument("--explain", action="append", default=[])
    ap.add_argument("--debuff", type=int, action="append", default=[], help="extra spell id to scan for aura-271 target debuffs")
    ap.add_argument("--fold-debuffs", action="store_true", help="multiply target debuffs into `derived` (default: report and write `debuff` only)")
    ap.add_argument("--debuff-uptime", type=float, default=1.0, help="fraction of the time folded debuffs are up")
    a = ap.parse_args()

    db = Db2(a.db2)
    sp = Spells(db)
    spec = tomllib.load(open(a.spec, "rb"))
    obs = json.load(open(a.observations))
    loadout = a.talents
    if os.path.isfile(loadout):
        loadout = next(l.split("=", 1)[1].strip() for l in open(loadout) if l.startswith("talents="))
    _, spec_id, talents = decode(db, loadout)
    passives = active_passives(db, sp, spec_id, talents)
    curves = RankCurves(db)
    definitions = {t["spell"]: t["definition"] for t in talents if t["spell"]}
    mods = modifiers(sp, passives, curves, definitions)
    if not curves.by_def:
        print("no TraitDefinitionEffectPoints/CurvePoint tables: talent ranks scale linearly", file=sys.stderr)
    scripted = scripted_effects(sp, passives)
    for sid, i, v in dummy_report(sp, passives, scripted):
        print(f"scripted aura-4 dummy, not derived: {sid} #{i} ({db.name(sid)}) points {v:g}", file=sys.stderr)
    print(f"spec {spec_id}: {len(passives)} active passives, {len(mods)} damage modifiers", file=sys.stderr)

    srcs = sources(spec, obs)
    derived = {}
    for name, (sid, periodic, d) in sorted(srcs.items()):
        mult, used = derive(sp, mods, sid, periodic)
        for x in scripted:
            if sid in x["sources"]:
                mult *= 1 + x["value"] / 100
                used.append({"value": x["value"], "origin": "script", "spell": x["spell"], "index": x["index"]})
        derived[name] = mult
        flag = ""
        c = coefficient(sp, sid, periodic)
        mine = d.get("ap_coeff", 0) or d.get("sp_coeff", 0)
        if c and mine and not d.get("banked"):
            theirs = c[0] if d.get("ap_coeff") else c[1]
            if theirs and abs(mine / theirs - 1) > 0.01:
                flag = f"  coefficient {mine:g} in spec, {theirs:g} in game data"
        print(f"{name:42s} {sid:8d} {'tick' if periodic else 'hit ':4s} x{mult:6.3f}  ({len(used)} mods){flag}", file=sys.stderr)
        if name in a.explain:
            for m in used:
                print(f"    {m['value']:+8.1f}%  {m['origin']:7s} {m['spell']:8d} #{m['index']} {db.name(m['spell'])}", file=sys.stderr)
    dmods, dunres = debuff_mods(sp, sorted(candidate_debuffs(sp, srcs, obs, a.debuff)))
    for m in dunres:
        print(f"target debuff {m['spell']} ({db.name(m['spell'])}) aura 271 {m['value']:g}% has no class mask: not applied", file=sys.stderr)
    debuff = {}
    for name, (sid, periodic, _) in sorted(srcs.items()):
        mult, used = derive(sp, dmods, sid, periodic)
        if used:
            debuff[name] = mult
            print(f"{name:42s} target debuff x{mult:6.3f}: " + ", ".join(f"{m['spell']} {db.name(m['spell'])} {m['value']:+g}%" for m in used), file=sys.stderr)
            if a.fold_debuffs:
                derived[name] *= 1 + (mult - 1) * a.debuff_uptime
    buffs = buff_patches(db, sp, spec, obs, srcs)
    for b, pct in buffs.items():
        print(f"buff {b}: " + ", ".join(f"{k} +{v:g}%" for k, v in sorted(pct.items())), file=sys.stderr)

    if a.out:
        with open(a.out, "w") as f:
            f.write("# Generated by tools/effects.py: per-hit multipliers derived from the\n")
            f.write("# game's spell modifiers (talents, spec auras), and the damage modifiers\n")
            f.write("# of the buffs the spec models. calibrate.py --derived merges it.\n\n")
            for name in sorted(derived):
                f.write(f'[sources."{name}"]\nderived = {derived[name]:.6f}\n')
                if name in debuff:
                    f.write(f'debuff = {debuff[name]:.6f}   # aura 271; folded into derived: {str(a.fold_debuffs).lower()}\n')
                f.write("\n")
            for b in sorted(buffs):
                f.write(f'[buffs."{b}".ability_pct]\n')
                for k, v in sorted(buffs[b].items()):
                    f.write(f'"{k}" = {v:.4f}\n')
                f.write("\n")


if __name__ == "__main__":
    main()
