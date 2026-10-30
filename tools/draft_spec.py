#!/usr/bin/env python3
"""First-draft spec from a SimC profile (APL, talents) and the DB2 tables.

  python3 tools/draft_spec.py <profile.simc> [--db2 target/db2] [--hero NAME]
                              [--out specs/<spec>.toml] [--diff specs/hand.toml]
  python3 tools/draft_spec.py <profile.simc> --apl-only     (no DB2 needed)

Two independent halves:

  APL -> [[rotation]]   (no game data needed)
      `actions=` lines are flattened (call_action_list inlines the called
      list under the call's condition, run_action_list also guards the
      entries after it) and each ability entry's `if=` is translated to the
      expression language of docs/spec-format.md: buff./dot./cooldown.
      fields, resources, gcd, time, fight_remains, target.health.pct,
      prev_gcd, constants folded in (talent.X when the loadout is known,
      fight_style.*, hero_tree.*), variable.X inlined. An entry whose
      condition can't be translated is written commented out, with the
      reason and the original SimC text (TODO), never with a guessed or
      shortened condition.

  DB2 -> [[abilities]] / [[buffs]]   (Db2Source, the only place that reads tables)
      per ability: cooldown, charges, cast time, GCD, cost, gain and the
      damage coefficient (AP or SP) and school from SpellEffect; per buff
      the DoT/aura duration and stacks. Every number is tagged [db2 <id>];
      what comes from the APL is tagged [simc]. Without --db2 the abilities
      are written as bare stubs (name only) with a TODO.

Nothing here is checked against the engine: the output is a starting point
(docs/draft-spec.md), run it through the recipe in AGENTS.md.
"""
import argparse
import os
import re
import sys
import tomllib

# --------------------------------------------------------------------------
# SimC expression -> engine expression
# --------------------------------------------------------------------------


class Untranslatable(Exception):
    pass


class NeverFires(Untranslatable):
    """The condition folds to false with this loadout/target count."""


TOKEN = re.compile(
    r"\s*(?:(?P<num>\d+(?:\.\d+)?)|(?P<id>[A-Za-z_][A-Za-z0-9_.]*)"
    r"|(?P<op><\?|>\?|<=|>=|!=|==|%%|&&|\|\||[|&!<>=+\-*/%()@^]))"
)
PREC = {"|": 1, "&": 2, "^": 2, "<": 3, "<=": 3, ">": 3, ">=": 3, "=": 3, "!=": 3,
        "+": 4, "-": 4, "*": 5, "/": 5, "%%": 5}


def lex(s):
    out, i = [], 0
    s = s.strip()
    while i < len(s):
        m = TOKEN.match(s, i)
        if not m or m.end() == i:
            raise Untranslatable(f"cannot tokenise {s[i:i + 12]!r}")
        i = m.end()
        out.append(("num", float(m["num"])) if m["num"] else ("id", m["id"]) if m["id"] else ("op", m["op"]))
    return out


def parse(s):
    t = lex(s)
    pos = [0]

    def peek():
        return t[pos[0]] if pos[0] < len(t) else (None, None)

    def unary():
        k, v = peek()
        pos[0] += 1
        if k == "num":
            return ("num", v)
        if k == "id":
            return ("id", v)
        if (k, v) == ("op", "("):
            e = expr(0)
            if peek() != ("op", ")"):
                raise Untranslatable("missing )")
            pos[0] += 1
            return e
        if (k, v) in (("op", "!"), ("op", "-")):
            return ("un", v, unary())
        if (k, v) == ("op", "@"):
            raise Untranslatable("abs value (@)")
        raise Untranslatable(f"unexpected {v!r}")

    def expr(minp):
        lhs = unary()
        while True:
            k, v = peek()
            if k != "op":
                break
            v = {"&&": "&", "||": "|", "==": "="}.get(v, v)
            if v in ("<?", ">?"):
                raise Untranslatable(f"operator {v}")
            if v == "%":
                v = "/"
            if v not in PREC or PREC[v] < minp:
                break
            pos[0] += 1
            lhs = ("bin", v, lhs, expr(PREC[v] + 1))
        return lhs

    e = expr(0)
    if pos[0] != len(t):
        raise Untranslatable("trailing tokens")
    return e


def unparse(e, parent=0, right=False):
    k = e[0]
    if k == "num":
        v = e[1]
        return str(int(v)) if v == int(v) else f"{v:g}"
    if k == "id":
        return e[1]
    if k == "un":
        inner = unparse(e[2], 9)
        return f"{e[1]}{inner}"
    p = PREC[e[1]]
    s = f"{unparse(e[2], p)}{e[1]}{unparse(e[3], p, True)}"
    return f"({s})" if p < parent or (p == parent and right) else s


def fold(e):
    k = e[0]
    if k == "un":
        a = fold(e[2])
        if a[0] == "num":
            return ("num", float(not a[1]) if e[1] == "!" else -a[1])
        if e[1] == "!" and a[0] == "un" and a[1] == "!":
            return a[2]
        return ("un", e[1], a)
    if k != "bin":
        return e
    op, a, b = e[1], fold(e[2]), fold(e[3])
    if a[0] == "num" and b[0] == "num":
        x, y = a[1], b[1]
        f = {"|": lambda: float(bool(x) or bool(y)), "&": lambda: float(bool(x) and bool(y)),
             "<": lambda: float(x < y), "<=": lambda: float(x <= y), ">": lambda: float(x > y),
             ">=": lambda: float(x >= y), "=": lambda: float(x == y), "!=": lambda: float(x != y),
             "+": lambda: x + y, "-": lambda: x - y, "*": lambda: x * y,
             "/": lambda: x / y if y else 0.0, "%%": lambda: x % y if y else 0.0}.get(op)
        if f:
            return ("num", f())
    for c, o in ((a, b), (b, a)):
        if c[0] == "num" and op == "&":
            return o if c[1] else ("num", 0.0)
        if c[0] == "num" and op == "|":
            return ("num", 1.0) if c[1] else o
    return ("bin", op, a, b)


RESOURCES = {"mana", "rage", "energy", "focus", "fury", "runic_power", "holy_power", "soul_shard",
             "soul_shards", "combo_points", "astral_power", "insanity", "maelstrom", "chi", "essence",
             "runes", "arcane_charges", "pain", "lunar_power"}
CD_FIELDS = {"ready": "ready", "up": "ready", "remains": "remains", "charges": "charges",
             "charges_fractional": "charges_fractional", "duration": "duration",
             "full_recharge_time": "full_recharge_time"}


class Ctx:
    """What the translator knows: the loadout, the hero tree, the action's own name, variables."""

    def __init__(self, talent=None, hero=None, variables=None, action=None, bare=False, fight_length=300.0):
        self.fight_length = fight_length
        self.bare = bare            # no trinkets or other equipped items (the core-kit references)
        self.talent = talent or (lambda name: None)    # name -> bool | None (unknown)
        self.hero = hero
        self.variables = variables or {}
        self.action = action
        self.depth = 0
        self.resources = set()


def num(v):
    return ("num", float(v))


def ident(*parts):
    return ("id", ".".join(parts))


def map_id(name, cx):
    p = name.split(".")
    h = p[0]
    n = len(p)
    if h == "talent" and n in (2, 3) and (n == 2 or p[2] == "enabled"):
        v = cx.talent(p[1])
        if v is None:
            raise Untranslatable(f"talent {p[1]} unknown (no loadout/--talent)")
        return num(v)
    if h == "hero_tree" and n == 2:
        if cx.hero is None:
            raise Untranslatable("hero tree unknown")
        return num(norm(p[1]) == norm(cx.hero))
    if h == "fight_style" and n == 2:
        return num(p[1] == "patchwerk")
    if cx.bare and (h == "equipped" or h == "trinket"):
        return num(0)
    if h == "raid_event" and n == 3:
        # patchwerk has no raid events: nothing exists or is up, and the next one never comes
        if p[2] in ("exists", "up", "down", "remains", "count", "cooldown", "duration", "max_distance"):
            return num(1 if p[2] == "down" else 0)
        if p[2] == "in":
            return num(1e9)
    if h == "buff" and n == 3:
        f = p[2]
        if f == "up":
            return ident(*p)
        if f == "down":
            return ("un", "!", ident("buff", p[1], "up"))
        if f in ("remains", "stack"):
            return ident(*p)
        if f in ("react", "stacks"):
            return ident("buff", p[1], "stack")
        raise Untranslatable(f"buff field {f}")
    if h == "dot" and n == 3:
        f = p[2]
        if f in ("ticking", "remains", "stack"):
            return ident(*p)
        if f == "up":
            return ident("dot", p[1], "ticking")
        if f == "down":
            return ("un", "!", ident("dot", p[1], "ticking"))
        raise Untranslatable(f"dot field {f}")
    if h == "cooldown" and n == 3:
        if p[2] == "down":
            return ("un", "!", ident("cooldown", p[1], "ready"))
        if p[2] in CD_FIELDS:
            return ident("cooldown", p[1], CD_FIELDS[p[2]])
        raise Untranslatable(f"cooldown field {p[2]}")
    if name in ("full_recharge_time", "charges", "charges_fractional") and cx.action:
        return ident("cooldown", cx.action, name)
    if name == "gcd.max":
        return ident("gcd")
    if name == "expected_combat_length":
        return num(cx.fight_length)
    if name == "active_enemies":
        return num(1)    # the engine simulates a single target
    if name in ("gcd", "gcd.remains", "time", "fight_remains", "target.health.pct",
                "mastery", "haste"):
        return ident(name)
    if name == "target.time_to_die" or name == "time_to_die":
        return ident("fight_remains")
    if h == "spell_targets":
        return num(1)
    if h == "prev_gcd" and n == 3 and p[1] == "1":
        return ident("prev_gcd", p[2])
    if h == "variable" and n == 2:
        v = cx.variables.get(p[1])
        if v is None or cx.depth > 8:
            raise Untranslatable(f"variable {p[1]} not a plain value")
        cx.depth += 1
        try:
            return translate_ast(parse(v), cx)
        finally:
            cx.depth -= 1
    if h in RESOURCES and n == 1:
        cx.resources.add(h)
        return ident(h)
    if h in RESOURCES and n == 2 and p[1] in ("max", "pct"):
        cx.resources.add(h)
        return ident(*p)
    if h in RESOURCES and n == 2 and p[1] == "deficit":
        cx.resources.add(h)
        return ("bin", "-", ident(h, "max"), ident(h))
    raise Untranslatable(f"{name}")


def translate_ast(e, cx):
    k = e[0]
    if k == "num":
        return e
    if k == "id":
        return map_id(e[1], cx)
    if k == "un":
        return ("un", e[1], translate_ast(e[2], cx))
    return ("bin", e[1], translate_ast(e[2], cx), translate_ast(e[3], cx))


def translate(cond, cx):
    """SimC condition text -> engine expression text, or raises Untranslatable.
    A constant-true condition comes back as '' (no `if`)."""
    e = fold(translate_ast(parse(cond), cx))
    if "%%" in unparse(e) if e[0] != "num" else False:
        raise Untranslatable("operator %% on a value known only at run time")
    if e[0] == "num":
        if e[1]:
            return ""
        raise NeverFires("constant false")
    return unparse(e)


# --------------------------------------------------------------------------
# APL parsing and flattening
# --------------------------------------------------------------------------

LINE = re.compile(r"^actions(?:\.(?P<list>[A-Za-z0-9_]+))?(?P<plus>\+)?=(?P<body>.*)$")
# actions with no counterpart in a spec (stats, items, racials, interrupts, movement, ...)
NOT_ABILITIES = {
    "auto_attack", "snapshot_stats", "use_item", "use_items", "potion", "flask", "food", "augmentation",
    "invoke_external_buff", "retarget", "variable", "blood_fury", "berserking", "fireblood",
    "ancestral_call", "arcane_torrent", "lights_judgment", "bag_of_tricks", "arcane_pulse", "rocket_barrage",
    "charge", "heroic_leap", "intervene", "pummel", "muzzle", "counter_shot", "rebuke", "mind_freeze",
    "disrupting_shout", "spear_hand_strike", "wind_shear", "kick", "counterspell", "skull_bash", "solar_beam",
    "consume_magic", "disrupt", "storm_bolt", "wait", "cancel_buff", "cancel_aura", "sequence",
    "strict_sequence", "pool_resource", "summon_pet", "call_action_list", "run_action_list", "swap_action_list",
    "berserker_stance", "defensive_stance", "battle_stance", "toggle", "stance",
}


def split_entry(body):
    parts = re.split(r",(?=[A-Za-z_]+=)", body.strip())
    head = parts[0].split(",")
    name, params = head[0], {}
    for extra in head[1:]:
        params.setdefault("_flags", []).append(extra)
    for kv in parts[1:]:
        k, _, v = kv.partition("=")
        params[k] = v
    return name, params


def parse_apl(text):
    """-> {list name: [(action, params)]}, in file order. The default list is 'default'."""
    lists = {}
    for raw in text.splitlines():
        m = LINE.match(raw.strip())
        if not m:
            continue
        lst = m["list"] or "default"
        body = m["body"]
        if m["plus"]:
            body = body.lstrip("/")
        lists.setdefault(lst, []).append(split_entry(body))
    return lists


def collect_variables(lists):
    """variable,name=X,value=V (op=set or unspecified) -> {X: V}. Later `set`s win; setif etc. are left out."""
    out = {}
    for entries in lists.values():
        for a, p in entries:
            if a == "variable" and p.get("op", "set") == "set" and "if" not in p and "value" in p:
                out[p["name"]] = p["value"]
    # a variable that is also set conditionally or by setif/max/min is not a plain value
    # (a reset only puts it back to its default, so a variable that is only ever reset stays at it)
    for entries in lists.values():
        for a, p in entries:
            if a == "variable" and p.get("op") == "reset" and "default" in p and p["name"] not in out:
                out[p["name"]] = p["default"]
    for entries in lists.values():
        for a, p in entries:
            if a == "variable" and (p.get("op", "set") not in ("set", "reset") or "if" in p):
                out.pop(p["name"], None)
    return out


def flatten(lists, name="default", conds=(), stack=()):
    """-> [(action, params, [simc condition, ...], list name)] in priority order."""
    out = []
    if name not in lists or name in stack:
        return out
    guard = []
    for a, p in lists[name]:
        here = list(conds) + guard
        if a in ("call_action_list", "run_action_list"):
            sub = p.get("name", "")
            c = [p["if"]] if "if" in p else []
            out += flatten(lists, sub, here + c, stack + (name,))
            if a == "run_action_list":
                if not c:
                    break
                guard.append(f"!({p['if']})")
            continue
        out.append((a, p, here + ([p["if"]] if "if" in p else []), name))
    return out


def translate_conds(conds, cx):
    parts = []
    for c in conds:
        t = translate(c, cx)
        if t:
            parts.append(t)
    if len(parts) == 1:
        return parts[0]
    return "&".join(f"({x})" if re.search(r"\|", x) else x for x in parts)


# the APL's name for an action -> the spell that replaces it when the loadout has that talent
OVERRIDES = {"templars_verdict": "final_verdict"}


def adjust_rows(rows, skipped, known):
    """Rename overridden actions and prune the ones whose talent isn't in the loadout
    (a bare `templar_strike` has no `talent.` guard in the APL, the spell simply isn't there)."""
    out = []
    for r in rows:
        a = r["action"]
        if OVERRIDES.get(a) and known(OVERRIDES[a]):
            r = dict(r, action=OVERRIDES[a])
        elif known(a) is False:
            skipped["(talent not taken)"] = skipped.get("(talent not taken)", 0) + 1
            continue
        out.append(r)
    return out


def build_rotation(lists, cx_factory):
    """-> list of dicts: action, if (str or None), status 'ok' | 'todo', reason, simc, list."""
    cx0 = cx_factory(None)
    cx0.variables = collect_variables(lists)
    rows, skipped = [], {}
    for a, p, conds, lname in flatten(lists):
        if a in NOT_ABILITIES:
            skipped[a] = skipped.get(a, 0) + 1
            continue
        cx = cx_factory(a)
        cx.variables = cx0.variables
        simc = "&".join(f"({c})" if "|" in c else c for c in conds)
        row = {"action": a, "simc": simc, "list": lname}
        try:
            row["if"] = translate_conds(conds, cx) or None
            row["status"] = "ok"
        except NeverFires:
            row["status"], row["reason"] = "pruned", "never fires with this loadout / on one target"
        except Untranslatable as e:
            row["status"], row["reason"] = "todo", str(e)
        cx0.resources |= cx.resources
        rows.append(row)
    return rows, skipped, cx0.resources


# --------------------------------------------------------------------------
# DB2 lookups: the only code that reads game tables
# --------------------------------------------------------------------------

POWER = {0: "mana", 1: "rage", 2: "focus", 3: "energy", 4: "combo_points", 5: "runes", 6: "runic_power",
         7: "soul_shards", 8: "astral_power", 9: "holy_power", 11: "maelstrom", 12: "chi", 13: "insanity",
         16: "arcane_charges", 17: "fury", 18: "pain", 19: "essence"}
POWER_DIV = {1: 10, 6: 10, 8: 10, 13: 10, 17: 10, 18: 10}   # stored in tenths (rage, runic power, ...)
# SchoolMask -> the name SimC (and the hand-written specs) use, including the multi-school ones
SCHOOL = {0x01: "physical", 0x02: "holy", 0x04: "fire", 0x08: "nature", 0x10: "frost", 0x20: "shadow", 0x40: "arcane",
          0x03: "holystrike", 0x05: "flamestrike", 0x06: "holyfire", 0x09: "stormstrike", 0x0a: "holystorm",
          0x0c: "firestorm", 0x11: "froststrike", 0x12: "holyfrost", 0x14: "frostfire", 0x18: "froststorm",
          0x21: "shadowstrike", 0x22: "shadowlight", 0x24: "shadowflame", 0x28: "shadowstorm", 0x30: "shadowfrost",
          0x41: "spellstrike", 0x42: "divine", 0x44: "spellfire", 0x48: "astral", 0x50: "spellfrost",
          0x60: "spellshadow", 0x1c: "elemental", 0x6a: "cosmic", 0x7c: "chromatic", 0x7d: "chromastrike",
          0x7e: "magic", 0x7f: "chaos"}
E_ENERGIZE = 30


def norm(s):
    return re.sub(r"[^a-z0-9]", "", s.lower())


class Db2Source:
    """Spell facts from the DB2 tables (tools/db2.py layout). Swap this class to change the source."""

    def __init__(self, directory, spec_id=None):
        from db2 import Db2
        from effects import Spells
        self.db = Db2(directory)
        self.sp = Spells(self.db)
        self.spec_id = spec_id
        self._names = None
        self.spec_spells = set()
        self.talent_spells = set()      # the loadout's talents; set by main() once decoded
        if spec_id:
            self.spec_spells = {int(r["SpellID"]) for r in self.db.multi("SpecializationSpells", "SpecID").get(spec_id, [])}

    def index(self):
        if self._names is None:
            self._names = {}
            for r in self.db.rows("SpellName"):
                self._names.setdefault(norm(r["Name_lang"]), []).append(int(r["ID"]))
        return self._names

    def candidates(self, name):
        return [s for s in self.index().get(norm(name.replace("_", " ")), []) if self.sp.effects.get(s)]

    def pick(self, name, aura=False):
        """Best spell id for a SimC name: a castable (not passive) spell that does something,
        then one the loadout's talents grant, then the spec's own spells, and among equals
        the oldest id (newer same-named rows are procs and overrides)."""
        c = self.candidates(name)
        if not c:
            return None, []

        def score(s):
            misc = self.sp.misc.get(s)
            has_dur = misc is not None and num_(misc.get("DurationIndex")) > 0
            dmg = bool(self.sp.damage_indices(s))
            cd = bool(self.db.by("SpellCooldowns", "SpellID").get(s))
            passive = self.sp.passive(s)
            return (not passive and (has_dur if aura else (dmg or cd)), not passive, s in self.talent_spells,
                    s in self.spec_spells, dmg and cd, -s)
        c.sort(key=score, reverse=True)
        return c[0], c

    def castables(self, talent_spells=()):
        """Active (non-passive) spells the spec has that do something: [(snake_name, id)].
        Used when the profile has no APL to take the ability list from."""
        out, seen = [], set()
        for sid in sorted(self.spec_spells | set(talent_spells)):
            if sid in seen or not self.sp.effects.get(sid) or self.sp.passive(sid):
                continue
            has_cd = bool(self.db.by("SpellCooldowns", "SpellID").get(sid))
            if not (self.sp.damage_indices(sid) or has_cd or self.db.multi("SpellPower", "SpellID").get(sid)):
                continue
            seen.add(sid)
            out.append((re.sub(r"[^a-z0-9]+", "_", self.db.name(sid).lower()).strip("_"), sid))
        return out

    def _duration(self, sid):
        misc = self.sp.misc.get(sid)
        if not misc:
            return 0.0
        d = self.db.by("SpellDuration", "ID").get(int(num_(misc.get("DurationIndex"))), {})
        ms = d.get("Duration", list(d.values())[1] if len(d) > 1 else 0)
        return num_(ms) / 1000

    def ability(self, sid):
        """Engine-ish facts for a spell: a dict of toml fields plus a list of notes."""
        by = self.db.by
        f, notes = {}, []
        cd = by("SpellCooldowns", "SpellID").get(sid, {})
        cat = by("SpellCategories", "SpellID").get(sid, {})
        chg = by("SpellCategory", "ID").get(int(num_(cat.get("ChargeCategory"))), {})
        recov = max(num_(cd.get("RecoveryTime")), num_(cd.get("CategoryRecoveryTime"))) / 1000
        if chg and num_(chg.get("ChargeRecoveryTime")):
            # a one-charge spell on a charge category keeps its cooldown here too (Blade of
            # Justice, Wake of Ashes); RecoveryTime is only the lockout between casts
            if num_(chg.get("MaxCharges")) > 1:
                f["charges"] = int(num_(chg["MaxCharges"]))
            f["cooldown"] = num_(chg["ChargeRecoveryTime"]) / 1000
        elif recov:
            f["cooldown"] = recov
        gcd = num_(cd.get("StartRecoveryTime"))
        if cd and gcd != 1500:
            f["gcd"] = gcd / 1000
        m = self.sp.misc.get(sid, {})
        cast = by("SpellCastTimes", "ID").get(int(num_(m.get("CastingTimeIndex"))), {})
        if num_(cast.get("Base")):
            f["cast_time"] = num_(cast["Base"]) / 1000
        cost, gain = {}, {}
        for p in self.db.multi("SpellPower", "SpellID").get(sid, []):
            t = int(num_(p.get("PowerType")))
            v = num_(p.get("ManaCost")) / POWER_DIV.get(t, 1)
            if v and t in POWER:
                cost[POWER[t]] = v
        for e in self.sp.effs(sid):
            if int(e["Effect"]) == E_ENERGIZE and int(num_(e["EffectMiscValue_0"])) in POWER:
                t = int(num_(e["EffectMiscValue_0"]))
                if num_(e["EffectBasePointsF"]):
                    gain[POWER[t]] = num_(e["EffectBasePointsF"]) / POWER_DIV.get(t, 1)
        if cost:
            f["cost"] = cost
        if gain:
            f["gain"] = gain
        idx = self.sp.damage_indices(sid)
        if idx:
            index, periodic = idx[0]
            eff = {int(e["EffectIndex"]): e for e in self.sp.effs(sid)}[index]
            ap, spc = num_(eff["BonusCoefficientFromAP"]), num_(eff["EffectBonusCoefficient"])
            mask = int(num_(m.get("SchoolMask"))) or 1
            school = SCHOOL.get(mask)
            if not school:
                school = SCHOOL.get(mask & -mask, "physical")
                notes.append(f"multi-school mask {mask}: using {school}, TODO check")
            dmg = {"school": school}
            if ap:
                dmg["ap_coeff"] = ap
            elif spc:
                dmg["sp_coeff"] = spc
            f["_periodic"] = periodic
            if periodic:
                f["_dot"] = {"tick_time": num_(eff["EffectAuraPeriod"]) / 1000, "duration": self._duration(sid), **dmg}
            else:
                f["damage"] = dmg
        return f, notes

    def buff(self, sid):
        ao = self.db.by("SpellAuraOptions", "SpellID").get(sid, {})
        lines = []
        for e in self.sp.effs(sid):
            if int(e["EffectAura"]):
                lines.append(f"effect #{e['EffectIndex']}: aura {e['EffectAura']} base {num_(e['EffectBasePointsF']):g} "
                             f"misc {e['EffectMiscValue_0']}/{e['EffectMiscValue_1']}")
        return {"duration": self._duration(sid), "max_stacks": int(num_(ao.get("CumulativeAura"))) or 1, "effects": lines}


def num_(x, d=0.0):
    try:
        return float(x)
    except (TypeError, ValueError):
        return d


# --------------------------------------------------------------------------
# profile + output
# --------------------------------------------------------------------------

CLASSES = {"warrior", "paladin", "hunter", "rogue", "priest", "deathknight", "shaman", "mage", "warlock",
           "monk", "druid", "demonhunter", "evoker"}


def read_profile(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    info = {"text": text, "class": None, "spec": None, "talents": None, "player": None}
    for raw in text.splitlines():
        line = raw.strip()
        m = re.match(r'^([a-z_]+)="([^"]*)"$', line)
        if m and m[1] in CLASSES:
            info["class"], info["player"] = m[1], m[2]
        for key in ("spec", "talents"):
            m = re.match(rf"^{key}=(\S+)$", line)
            if m:
                info[key] = m[1]
    return info


def spec_id_of(talents):
    """Spec id from the loadout string header (version 8 bits, spec 16 bits)."""
    from talents import B64
    bits = "".join(f"{B64.index(c):06b}"[::-1] for c in talents[:6])
    return int(bits[8:24][::-1], 2)


def toml_val(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, float):
        return f"{v:.6g}"
    if isinstance(v, dict):
        return "{ " + ", ".join(f"{k} = {toml_val(x)}" for k, x in v.items()) + " }"
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    return str(v)


def emit(profile, rows, skipped, resources, db, hero, talents_known, extra=()):
    out = []
    w = out.append
    pname = profile["player"] or "profile"
    w(f"# DRAFT / WIP: generated by tools/draft_spec.py from {pname}. Not registered in src/api.rs,")
    w("# not calibrated, not validated. Tags: [db2 <spell id>] game tables, [simc] the profile's APL.")
    w("# Every TODO is a decision the tool could not make. See docs/draft-spec.md.")
    w(f"# talents known: {'yes' if talents_known else 'no (talent.* conditions are TODO)'}; hero tree: {hero or 'unknown'}")
    if skipped:
        w("# APL actions with no spec counterpart, skipped: " + ", ".join(f"{k} x{v}" for k, v in sorted(skipped.items())))
    todo = {}
    for r in rows:
        if r["status"] == "todo":
            todo[r["reason"]] = todo.get(r["reason"], 0) + 1
    if todo:
        w("# untranslated rotation entries by reason: " + "; ".join(f"{k} x{v}" for k, v in sorted(todo.items(), key=lambda kv: -kv[1])))
    w("")
    w(f'name = "{profile["class"].title()} {profile["spec"].title()}{" (" + hero + ")" if hero else ""} DRAFT"')
    w("")
    w("[player]   # TODO: stats, weapon, crit multiplier: see the reference observations")
    w("")
    for r in sorted(resources):
        w(f"[resources.{r}]   # TODO max/start/regen")
        w("")

    names = []
    for r in rows:
        if r["action"] not in names:
            names.append(r["action"])
    for n in extra:
        if n not in names:
            names.append(n)
    buff_names = []
    for r in rows:
        for b in re.findall(r"buff\.([a-z0-9_]+)\.", r.get("if") or r["simc"]):
            if b not in buff_names:
                buff_names.append(b)

    for n in names:
        w("[[abilities]]")
        w(f'name = "{n}"')
        if db is None:
            w("# TODO: no DB2 given (--db2): cooldown, cost, cast time, coefficient")
            w("")
            continue
        sid, cands = db.pick(n)
        if sid is None:
            w("# TODO: no spell named like this in SpellName (talent not taken, or the name differs)")
            w("")
            continue
        f, notes = db.ability(sid)
        w(f"spell_id = {sid}   # [db2]" + (f" (also: {', '.join(map(str, cands[1:6]))})" if len(cands) > 1 else ""))
        for k in ("gcd", "cooldown", "charges", "cast_time"):
            if k in f:
                w(f"{k} = {toml_val(f[k])}   # [db2 {sid}]")
        if "cooldown" in f:
            w("# TODO cooldown_hasted: not in DB2 (SimC: cooldown.hasted)")
        for k in ("cost", "gain", "damage"):
            if k in f:
                w(f"{k} = {toml_val(f[k])}   # [db2 {sid}]")
        if "_dot" in f:
            w(f"# periodic damage (no direct hit): TODO [abilities.dot] tick_time/duration/coefficient, [db2 {sid}]: "
              f"{toml_val(f['_dot'])}")
        if "damage" in f or "_dot" in f:
            w("# TODO mastery = true? (needs the spec's mastery)")
        for note in notes:
            w(f"# TODO {note}")
        w("")

    for b in buff_names:
        if b in names:
            continue
        w("[[buffs]]")
        w(f'name = "{b}"')
        if db is None:
            w("# TODO: duration, stacks")
            w("")
            continue
        sid, cands = db.pick(b, aura=True)
        if sid is None:
            w("duration = 0   # TODO: no spell named like this in SpellName")
            w("")
            continue
        f = db.buff(sid)
        w(f"duration = {toml_val(f['duration'])}   # [db2 {sid}]" + (f" (also: {', '.join(map(str, cands[1:6]))})" if len(cands) > 1 else ""))
        if f["max_stacks"] > 1:
            w(f"max_stacks = {f['max_stacks']}   # [db2 {sid}]")
        for line in f["effects"]:
            w(f"# TODO modifier, [db2 {sid}] {line}")
        w("")

    if not rows:
        w("# TODO rotation: the profile has no actions= lines; the abilities above are the spec's active spells (SpecializationSpells + talents)")
        w("")
    for r in rows:
        tag = f"# [simc] list {r['list']}"
        if r["status"] == "ok":
            w(tag)
            w("[[rotation]]")
            w(f'action = "{r["action"]}"')
            if r["if"]:
                w(f'if = "{r["if"]}"')
        else:
            w(f"# TODO untranslated ({r['reason']}); [simc] list {r['list']}: if={r['simc'] or '-'}")
            w("# [[rotation]]")
            w(f'# action = "{r["action"]}"')
        w("")
    return "\n".join(out)


# --------------------------------------------------------------------------
# structural diff against a hand-written spec
# --------------------------------------------------------------------------


def diff_specs(draft, hand):
    """Both are parsed spec dicts. -> list of text lines."""
    out = []
    da = {a["name"]: a for a in draft.get("abilities", [])}
    ha = {a["name"]: a for a in hand.get("abilities", [])}
    out.append(f"abilities: draft {len(da)}, hand {len(ha)}, common {len(set(da) & set(ha))}")
    out.append("  only in draft: " + ", ".join(sorted(set(da) - set(ha))))
    out.append("  only in hand:  " + ", ".join(sorted(set(ha) - set(da))))
    keys = ("cooldown", "charges", "cast_time", "gcd")
    for n in sorted(set(da) & set(ha)):
        for k in keys:
            d, h = da[n].get(k), ha[n].get(k)
            if d is not None and h is not None and abs(d - h) > 1e-6:
                out.append(f"  {n}.{k}: draft {d:g}, hand {h:g}")
            elif (d is None) != (h is None) and k in ("cooldown", "charges", "cast_time"):
                out.append(f"  {n}.{k}: draft {d}, hand {h}")
        for k in ("cost", "gain"):
            if da[n].get(k, {}) != ha[n].get(k, {}):
                out.append(f"  {n}.{k}: draft {da[n].get(k, {})}, hand {ha[n].get(k, {})}")
        dd, hd = da[n].get("damage", {}), ha[n].get("damage", {})
        for k in ("ap_coeff", "sp_coeff", "school"):
            if dd.get(k) != hd.get(k) and (dd.get(k) or hd.get(k)):
                out.append(f"  {n}.damage.{k}: draft {dd.get(k)}, hand {hd.get(k)}")
    db_ = {b["name"] for b in draft.get("buffs", [])}
    hb = {b["name"] for b in hand.get("buffs", [])}
    out.append(f"buffs: draft {len(db_)}, hand {len(hb)}, common {len(db_ & hb)}")
    out.append("  only in hand:  " + ", ".join(sorted(hb - db_)))
    dr = [r["action"] for r in draft.get("rotation", [])]
    hr = [r["action"] for r in hand.get("rotation", [])]
    out.append(f"rotation entries: draft {len(dr)}, hand {len(hr)}")
    first = lambda xs: [x for i, x in enumerate(xs) if x not in xs[:i]]
    out.append("  action order, draft: " + " > ".join(first(dr)))
    out.append("  action order, hand:  " + " > ".join(first(hr)))
    return out


# --------------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("profile")
    ap.add_argument("--db2", default=None, help="DB2 CSV directory (python3 tools/db2.py)")
    ap.add_argument("--hero", default=None, help="hero tree name (default: decoded from the loadout when --db2 is given)")
    ap.add_argument("--talent", action="append", default=[], metavar="NAME=0|1",
                    help="set a talent by hand (when no DB2 to decode the loadout)")
    ap.add_argument("--spec-id", type=int, default=None, help="ChrSpecialization id (default: from the talents= string)")
    ap.add_argument("--out", default=None)
    ap.add_argument("--apl", default=None, metavar="FILE",
                    help="SimC action list to translate (default: the profile's own actions= lines, "
                         "else <--simc>/ActionPriorityLists/default/<class>_<spec>.simc)")
    ap.add_argument("--simc", default=os.environ.get("SIMC_DIR"),
                    help="SimC checkout, for the default action list of profiles that have none (env SIMC_DIR)")
    ap.add_argument("--fight-length", type=float, default=300.0, help="expected_combat_length, seconds")
    ap.add_argument("--bare", action="store_true",
                    help="assume no trinkets or other equipped items (equipped.* and trinket.* fold to 0), as in the core-kit references")
    ap.add_argument("--apl-only", action="store_true", help="rotation only, no abilities/buffs")
    ap.add_argument("--diff", default=None, help="hand-written spec to compare the draft with")
    a = ap.parse_args()

    prof = read_profile(a.profile)
    if not prof["class"] or not prof["spec"]:
        sys.exit("profile has no class/spec line")
    hero = a.hero

    db, tspells = None, []
    talents = {t.split("=")[0]: t.split("=")[1] == "1" for t in a.talent}
    if a.db2 and not a.apl_only:
        db = Db2Source(a.db2, a.spec_id or (spec_id_of(prof["talents"]) if prof["talents"] else None))
        if prof["talents"]:
            from talents import decode
            _, _, sel = decode(db.db, prof["talents"])
            if not hero:
                subs = {t["subtree"] for t in sel if t["subtree"]}
                sub_rows = db.db.by("TraitSubTree")
                hero = next((sub_rows[i]["Name_lang"] for i in subs if i in sub_rows), None)
            tspells = [t["spell"] for t in sel if t["spell"]]
            db.talent_spells = set(tspells)
            have = {norm(db.db.name(t["spell"])) for t in sel if t["spell"]}
            every = {norm(db.db.name(int(num_(r["SpellID"])))) for r in db.db.rows("TraitDefinition")}
            known = lambda name: (talents[name] if name in talents else
                                  (norm(name.replace("_", " ")) in have) if norm(name.replace("_", " ")) in every else None)
        else:
            known = lambda name: talents.get(name)
    else:
        known = lambda name: talents.get(name)

    lists = parse_apl(prof["text"])
    apl_file = a.apl
    if not apl_file and "default" not in lists and a.simc:
        guess = os.path.join(a.simc, "ActionPriorityLists", "default", f"{prof['class']}_{prof['spec']}.simc")
        apl_file = guess if os.path.exists(guess) else None
    if apl_file and ("default" not in lists or a.apl):
        with open(apl_file, encoding="utf-8") as f:
            lists = parse_apl(f.read())
        print(f"rotation from {apl_file}", file=sys.stderr)
    rows, skipped, resources = build_rotation(lists, lambda act: Ctx(talent=known, hero=hero, action=act, bare=a.bare, fight_length=a.fight_length))
    rows = adjust_rows(rows, skipped, known)
    pruned = sum(r["status"] == "pruned" for r in rows)
    rows = [r for r in rows if r["status"] != "pruned"]
    if pruned:
        skipped["(pruned: never fires)"] = pruned
    extra = []
    if not rows and db is not None:
        extra = [n for n, _ in db.castables(tspells)]
    text = emit(prof, rows, skipped, resources, db, hero, bool(a.db2 and prof["talents"]) or bool(talents), extra)
    if a.out:
        with open(a.out, "w", encoding="utf-8") as f:
            f.write(text + "\n")
        print(f"wrote {a.out}: {len(rows)} rotation entries, {sum(r['status'] == 'todo' for r in rows)} TODO", file=sys.stderr)
    else:
        print(text)
    if a.diff:
        tomllib.loads(text)   # the draft must at least be valid TOML
        print("\n".join(diff_specs(tomllib.loads(text), tomllib.loads(open(a.diff, encoding="utf-8").read()))))


if __name__ == "__main__":
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    main()
