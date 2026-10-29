#!/usr/bin/env python3
"""Pull real combat data from the Warcraft Logs v2 API and turn it into an
observations file in the same format tools/observe.py makes from SimC.

  python3 tools/wcl.py survey  --zone 53 [--class Warlock --spec Demonology]
      top parses per encounter: how much of the player's damage hits the
      main target, to pick single-target fights
  python3 tools/wcl.py observe --encounter 3420 [-n 25] [--difficulty 5] --out reference/demo_wcl
      -> <out>_observations.json  pooled observations (observe.py schema)
         <out>_fights.json        per fight: dps, hero tree, talents, loadout string, /simc gear

Credentials are read at run time from 1Password (op-agent) and only kept in
memory. Every API response is cached under target/wcl/ (gitignored), so a rerun
costs no points; --refresh re-reads the rankings (fight events never change).
Needs Python 3.11+ (tomllib-era stdlib only) and, for talent names and loadout
strings, the DB2 tables in target/db2 (python3 tools/db2.py).
"""
import argparse
import base64
import hashlib
import json
import re
import statistics
import subprocess
import sys
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CACHE = ROOT / "target" / "wcl"
TOKEN_URL = "https://www.warcraftlogs.com/oauth/token"
API_URL = "https://www.warcraftlogs.com/api/v2/client"

# the permanent pet's actor carries the player's custom name (often not
# English); call it by its kind instead
PERMANENT_PET = {"Demonology": "felguard", "Affliction": "pet", "Destruction": "pet"}
SCHOOLS = {1: "physical", 2: "holy", 4: "fire", 8: "nature", 16: "frost", 32: "shadow", 64: "arcane",
           36: "shadowflame", 48: "shadowfrost", 40: "plague", 20: "frostfire", 33: "shadowstrike",
           124: "chromatic", 126: "magic", 127: "chaos", 3: "holystrike", 6: "holyfire", 5: "flamestrike"}
SLOTS = ["head", "neck", "shoulder", "shirt", "chest", "waist", "legs", "feet", "wrist", "hands",
         "finger1", "finger2", "trinket1", "trinket2", "back", "main_hand", "off_hand", "tabard"]
HIT, CRIT = 1, 2
# damage events that carry the player as source but are an Augmentation
# Evoker's (WCL credits them to the evoker)
EVOKER_ATTRIBUTED = {"breath_of_eons", "fate_mirror"}
EBON_MIGHT = 395152

_token = None
spent = {"queries": 0, "cached": 0}


# ---------------------------------------------------------------- API

def op_read(ref):
    return subprocess.run(["op-agent", "read", ref], check=True, capture_output=True, text=True).stdout.strip()


def token():
    global _token
    if _token is None:
        cid = op_read("op://Agents/WCL_CLIENT_ID/credential")
        secret = op_read("op://Agents/WCL_CLIENT_SECRET/credential")
        req = urllib.request.Request(
            TOKEN_URL, data=urllib.parse.urlencode({"grant_type": "client_credentials"}).encode(),
            headers={"Authorization": "Basic " + base64.b64encode(f"{cid}:{secret}".encode()).decode()})
        with urllib.request.urlopen(req, timeout=30) as r:
            _token = json.load(r)["access_token"]
    return _token


def gql(query, variables=None, cache=True):
    """Run a GraphQL query; cache the data by (query, variables) under target/wcl/q/."""
    variables = variables or {}
    key = hashlib.sha256(json.dumps([query, variables], sort_keys=True).encode()).hexdigest()[:24]
    path = CACHE / "q" / f"{key}.json"
    if cache and path.exists():
        spent["cached"] += 1
        return json.loads(path.read_text())
    body = json.dumps({"query": query, "variables": variables}).encode()
    req = urllib.request.Request(API_URL, data=body, headers={
        "Authorization": "Bearer " + token(), "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        resp = json.load(r)
    spent["queries"] += 1
    if "errors" in resp:
        raise RuntimeError(json.dumps(resp["errors"])[:2000])
    data = resp["data"]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data))
    return data


def rate():
    return gql("{ rateLimitData { limitPerHour pointsSpentThisHour pointsResetIn } }", cache=False)["rateLimitData"]


def check_budget(margin=200):
    r = rate()
    if r["pointsSpentThisHour"] > r["limitPerHour"] - margin:
        sys.exit(f"rate limit nearly used ({r['pointsSpentThisHour']:.0f}/{r['limitPerHour']}); "
                 f"resets in {r['pointsResetIn']}s. Rerun then: cached fights cost nothing.")
    return r


RANKINGS = """query($id:Int!,$d:Int!,$c:String!,$s:String!,$p:Int!){ worldData { encounter(id:$id){ name
  characterRankings(className:$c, specName:$s, metric:dps, difficulty:$d, page:$p, includeCombatantInfo:true) } } }"""
REPORT = """query($c:String!,$f:[Int]!){ reportData { report(code:$c){
  masterData { actors { id name type subType petOwner gameID } abilities { gameID name type } }
  fights(fightIDs:$f){ id startTime endTime encounterID kill } } } }"""
EVENTS = """query($c:String!,$f:[Int]!,$s:Float!,$e:Float!,$t:EventDataType!,$src:Int,$tgt:Int){ reportData { report(code:$c){
  events(fightIDs:$f,startTime:$s,endTime:$e,dataType:$t,sourceID:$src,targetID:$tgt,limit:10000){ data nextPageTimestamp } } } }"""
TARGETS = """query($c:String!,$f:[Int]!,$s:Int!){ reportData { report(code:$c){
  table(dataType:DamageDone, fightIDs:$f, sourceID:$s, viewBy:Target) } } }"""


def rankings(enc, diff, cls, spec, n, refresh=False):
    out, page = [], 1
    while len(out) < n:
        r = gql(RANKINGS, {"id": enc, "d": diff, "c": cls, "s": spec, "p": page}, cache=not refresh)
        e = r["worldData"]["encounter"]
        cr = e["characterRankings"]
        out += cr.get("rankings", [])
        if not cr.get("hasMorePages"):
            break
        page += 1
    return e["name"], out[:n]


def events(code, fight, start, end, dtype, src=None, tgt=None):
    out, t = [], start
    while t is not None:
        ev = gql(EVENTS, {"c": code, "f": [fight], "s": t, "e": end, "t": dtype, "src": src, "tgt": tgt})
        ev = ev["reportData"]["report"]["events"]
        out += ev["data"]
        t = ev["nextPageTimestamp"]
    return out


def load_fight(rk):
    """All the raw data for one ranked parse."""
    code, fid = rk["report"]["code"], rk["report"]["fightID"]
    rep = gql(REPORT, {"c": code, "f": [fid]})["reportData"]["report"]
    fight = rep["fights"][0]
    actors = rep["masterData"]["actors"]
    me = [a for a in actors if a["type"] == "Player" and a["name"] == rk["name"]]
    if not me:
        raise ValueError(f"{rk['name']} not in report {code}")
    pid = me[0]["id"]
    s, e = fight["startTime"], fight["endTime"]
    return {
        "rank": rk, "code": code, "fight": fight, "pid": pid,
        "actors": {a["id"]: a for a in actors},
        "abilities": {a["gameID"]: a for a in rep["masterData"]["abilities"]},
        # sourceID on a player also returns its pets' events
        "damage": events(code, fid, s, e, "DamageDone", src=pid),
        "casts": events(code, fid, s, e, "Casts", src=pid),
        "buffs": events(code, fid, s, e, "Buffs", tgt=pid),
        "info": events(code, fid, s, e, "CombatantInfo", src=pid),
    }


# ---------------------------------------------------------------- per fight

def snake(s):
    return re.sub(r"[^a-z0-9]+", "_", s.lower().replace("'", "")).strip("_")


def school(t):
    try:
        return SCHOOLS.get(int(t), str(t))
    except (TypeError, ValueError):
        return ""


def pet_prefix(actor, spec):
    name = actor["name"]
    if actor["gameID"] > 10**6:  # a player-named permanent pet: gameID is its own guid, not a creature id
        return PERMANENT_PET.get(spec, "pet")
    name = re.sub(r"\s*\(.*\)", "", name)  # "Wild Imp (Hand of Gul'dan)" -> "Wild Imp", like SimC
    return snake(name) if name.isascii() and snake(name) else f"pet{actor['gameID']}"


def observe_fight(f):
    pid, spec = f["pid"], f["rank"]["spec"]
    t0, t1 = f["fight"]["startTime"], f["fight"]["endTime"]
    length = (t1 - t0) / 1000
    abil = f["abilities"]

    def aname(gid):
        return abil.get(gid, {}).get("name") or str(gid)

    casts = Counter(e["abilityGameID"] for e in f["casts"] if e["type"] == "cast" and e["sourceID"] == pid)
    casts_by_name = Counter()
    for gid, n in casts.items():
        casts_by_name[snake(aname(gid))] += n

    acc = defaultdict(lambda: {"spell_id": 0, "school": "", "dmg": 0.0, "dh": 0, "dc": 0, "dhit_dmg": 0.0,
                               "th": 0, "tc": 0, "thit_dmg": 0.0, "pet": None})
    for e in f["damage"]:
        if e["type"] != "damage":
            continue
        src = e["sourceID"]
        name = snake(aname(e["abilityGameID"]))
        if name in EVOKER_ATTRIBUTED:
            continue
        pet = None
        if src != pid:
            pet = pet_prefix(f["actors"].get(src, {"name": "pet", "gameID": 0}), spec)
            name = f"{pet}_{name}"
        a = acc[name]
        a["spell_id"] = e["abilityGameID"]
        a["school"] = school(abil.get(e["abilityGameID"], {}).get("type"))
        a["pet"] = pet
        amt = e.get("amount", 0) + e.get("absorbed", 0)
        a["dmg"] += amt
        crit = e.get("hitType") == CRIT
        if e.get("tick"):
            a["tc" if crit else "th"] += 1
            if e.get("hitType") == HIT:
                a["thit_dmg"] += amt
        else:
            a["dc" if crit else "dh"] += 1
            if e.get("hitType") == HIT:
                a["dhit_dmg"] += amt
    total = sum(a["dmg"] for a in acc.values())
    sources = {}
    for name, a in acc.items():
        if a["dmg"] <= 0:
            continue
        dn, tn = a["dh"] + a["dc"], a["th"] + a["tc"]
        sources[name] = {
            "spell_id": a["spell_id"], "school": a["school"], "pet": a["pet"],
            "damage": a["dmg"],
            "executes": casts_by_name.get(name, 0) or dn or tn,
            "direct_hits": dn, "direct_crits": a["dc"], "direct_hit_damage": a["dhit_dmg"], "direct_noncrits": a["dh"],
            "ticks": tn, "tick_crits": a["tc"], "tick_hit_damage": a["thit_dmg"], "tick_noncrits": a["th"],
        }

    buffs = observe_buffs(f, t0, t1, aname)
    info = f["info"][0] if f["info"] else {}
    return {
        "length": length, "damage": total, "dps": total / length,
        "sources": sources, "buffs": buffs,
        "casts": {snake(aname(g)): {"spell_id": g, "count": n} for g, n in casts.items()},
        "idle": idle_time(f["casts"], pid, t0, t1),
        "aug": ebon_might_share(f["damage"]) > 0.05,
        "ebon_might_share": ebon_might_share(f["damage"]),
        "info": info,
    }


def observe_buffs(f, t0, t1, aname):
    """Buffs on the player: uptime, applications, refreshes, from apply/remove events."""
    pid = f["pid"]
    st = defaultdict(lambda: {"up": None, "time": 0.0, "starts": 0, "refreshes": 0, "durs": [], "ext": False})
    # auras already up at the pull (combatantinfo lists them) start at t0
    pre = {a["ability"]: a.get("source") for e in f["info"] for a in e.get("auras", [])}
    seen = set()
    for e in f["buffs"]:
        if e.get("targetID") != pid:
            continue
        g, ty, t = e["abilityGameID"], e["type"], e["timestamp"]
        b = st[g]
        if e.get("sourceID") != pid:
            b["ext"] = True
        if ty == "applybuff":
            if b["up"] is None:
                b["up"] = t
            b["starts"] += 1
        elif ty == "refreshbuff":
            b["refreshes"] += 1
        elif ty == "removebuff":
            start = b["up"]
            if start is None and g not in seen:
                start = t0  # was up before the fight
            if start is not None:
                b["time"] += t - start
                b["durs"].append(t - start)
            b["up"] = None
        seen.add(g)
    for g in set(pre) - seen:
        st[g]["ext"] = pre[g] != pid
        st[g]["time"] = t1 - t0
        st[g]["starts"] = 0
        st[g]["durs"] = [t1 - t0]
    out = {}
    for g, b in st.items():
        if b["up"] is not None:
            b["time"] += t1 - b["up"]
            b["durs"].append(t1 - b["up"])
        out[snake(aname(g))] = {"spell_id": g, "time": b["time"] / 1000, "starts": b["starts"],
                                "refreshes": b["refreshes"], "dur_sum": sum(b["durs"]) / 1000,
                                "dur_n": len(b["durs"]), "external": b["ext"]}
    return out


def ebon_might_share(damage):
    """Share of damage done with an Augmentation Evoker's Ebon Might up (damage
    events list the source's buffs as "id.id.")."""
    tot = em = 0
    tag = f"{EBON_MIGHT}."
    for e in damage:
        if e["type"] == "damage":
            amt = e.get("amount", 0) + e.get("absorbed", 0)
            tot += amt
            if f".{tag}" in "." + e.get("buffs", ""):
                em += amt
    return em / tot if tot else 0.0


def idle_time(casts, pid, t0, t1, gcd=1.5):
    """Seconds with no cast in progress and no recent instant: gaps between
    player actions beyond one (unhasted) GCD. A rough movement/downtime proxy."""
    ts = sorted({e["timestamp"] for e in casts if e["sourceID"] == pid and e["type"] in ("cast", "begincast")})
    ts = [t0] + ts + [t1]
    return sum(max(0.0, (b - a) / 1000 - gcd) for a, b in zip(ts, ts[1:]))


# ---------------------------------------------------------------- talents and gear

def talent_info(db, info):
    """Talent tree entries -> names, hero tree and a loadout string (None without DB2)."""
    tree = info.get("talentTree", [])
    if db is None or not tree:
        return {"entries": {t["id"]: t["rank"] for t in tree}}
    from db2 import num
    entries, defs = db.by("TraitNodeEntry"), db.by("TraitDefinition")
    nodes = db.by("TraitNode")
    subtrees = db.by("TraitSubTree")
    names, hero = [], Counter()
    for t in tree:
        e = entries.get(t["id"])
        if not e:
            continue
        d = defs.get(int(e["TraitDefinitionID"]), {})
        sid = int(num(d.get("SpellID")))
        if not sid:
            continue  # the hero tree selection node
        nm = d.get("OverrideName_lang") or db.name(sid)
        sub = int(num(nodes.get(t["nodeID"], {}).get("TraitSubTreeID")))
        if sub:
            hero[sub] += 1
        names.append({"entry": t["id"], "node": t["nodeID"], "rank": t["rank"], "spell": sid, "name": nm, "hero": bool(sub)})
    hero_id = hero.most_common(1)[0][0] if hero else 0
    out = {"hero_tree": subtrees.get(hero_id, {}).get("Name_lang", ""), "talents": names}
    try:
        out["loadout"] = encode_loadout(db, info["specID"], tree, hero_id)
    except Exception as ex:  # the string is a convenience; the entry list is the record
        out["loadout_error"] = str(ex)
    return out


def encode_loadout(db, spec, tree, hero_id):
    """Inverse of talents.decode: the in-game / SimC talents= string (tree hash zeroed)."""
    import talents
    class_trees = {int(r["TraitTreeID"]) for r in db.rows("SkillLineXTraitTree")}
    trees = {int(r["TraitTreeID"]) for r in db.multi("TraitTreeLoadout", "ChrSpecializationID").get(spec, [])} & class_trees
    (tid,) = trees
    node_ids = sorted(int(r["ID"]) for r in db.rows("TraitNode") if int(r["TraitTreeID"]) == tid)
    node_entries = db.multi("TraitNodeXTraitNodeEntry", "TraitNodeID")
    entries = db.by("TraitNodeEntry")
    nodes = db.by("TraitNode")
    picked = defaultdict(list)
    for t in tree:
        picked[t["nodeID"]].append(t)
    bits = []

    def put(v, n):
        bits.extend((v >> i) & 1 for i in range(n))

    put(2, 8)
    put(spec, 16)
    put(0, 128)
    for node in node_ids:
        ents = [int(r["TraitNodeEntryID"]) for r in sorted(node_entries.get(node, []), key=lambda r: int(r["_Index"]))]
        sel = [i for i, e in enumerate(ents) if int(entries.get(e, {}).get("TraitSubTreeID", 0) or 0) == hero_id] if hero_id else []
        ntype = int(nodes[node]["Type"])
        if node in picked:
            ts = picked[node]
            idx = ents.index(ts[0]["id"]) if ts[0]["id"] in ents else 0
            if ntype in (2, 3):  # choice / hero selection: one entry
                rank, maxr = ts[0]["rank"], int(entries[ts[0]["id"]]["MaxRanks"])
            else:  # ranked or tiered: ranks add up over the node's entries
                rank = sum(t["rank"] for t in ts)
                maxr = sum(int(entries[e]["MaxRanks"]) for e in ents) or rank
        elif ntype == 3 and sel:
            idx, rank, maxr = sel[0], 1, 1
        else:
            put(0, 1)
            continue
        put(1, 1)  # selected
        put(1, 1)  # purchased
        if rank < maxr:
            put(1, 1)
            put(rank, 6)
        else:
            put(0, 1)
        if ntype in (2, 3):
            put(1, 1)
            put(idx, 2)
        else:
            put(0, 1)
    while len(bits) % 6:
        bits.append(0)
    s = "".join(talents.B64[sum(b << i for i, b in enumerate(bits[k:k + 6]))] for k in range(0, len(bits), 6))
    # round trip: every entry we encoded must come back
    _, _, back = talents.decode(db, s)
    got = {t["node"] for t in back}
    want = {t["nodeID"] for t in tree if int(nodes[t["nodeID"]]["Type"]) != 3}
    if want - got:
        raise ValueError(f"loadout round trip lost nodes {sorted(want - got)[:5]}")
    got_e = {t["entry"] for t in back}
    choice = {t["id"] for t in tree if int(nodes[t["nodeID"]]["Type"]) == 2}
    if choice - got_e:
        raise ValueError(f"loadout round trip lost choice entries {sorted(choice - got_e)[:5]}")
    return s


def simc_gear(info):
    lines = []
    for i, g in enumerate(info.get("gear", [])):
        if not g.get("id") or i >= len(SLOTS) or SLOTS[i] in ("shirt", "tabard"):
            continue
        parts = [f"{SLOTS[i]}=", f"id={g['id']}"]
        if g.get("bonusIDs"):
            parts.append("bonus_id=" + "/".join(map(str, g["bonusIDs"])))
        if g.get("permanentEnchant"):
            parts.append(f"enchant_id={g['permanentEnchant']}")
        if g.get("gems"):
            parts.append("gem_id=" + "/".join(str(x["id"]) for x in g["gems"]))
        parts.append(f"ilevel={g['itemLevel']}")
        lines.append(",".join(parts))
    return lines


def char_stats(info):
    return {
        "intellect": info.get("intellect"), "strength": info.get("strength"), "agility": info.get("agility"),
        "crit_rating": info.get("critSpell"), "haste_rating": info.get("hasteSpell"),
        "mastery_rating": info.get("mastery"), "versatility_rating": info.get("versatilityDamageDone"),
    }


# ---------------------------------------------------------------- pooling

def pool(obs, player):
    """Pool per-fight observations into one, at the mean fight length:
    counts and damage are rates over the total time, averages and crit rates
    are pooled over all hits."""
    T = sum(o["length"] for o in obs)
    L = T / len(obs)
    scale = L / T
    src = defaultdict(lambda: defaultdict(float))
    meta = {}
    for o in obs:
        for n, s in o["sources"].items():
            for k, v in s.items():
                if isinstance(v, (int, float)) and not isinstance(v, bool) and k != "spell_id":
                    src[n][k] += v
            src[n]["fights"] += 1
            meta[n] = (s["spell_id"], s["school"], s["pet"])
    total = sum(s["damage"] for s in src.values())
    sources = {}
    for n, s in sorted(src.items(), key=lambda kv: -kv[1]["damage"]):
        sid, school, pet = meta[n]
        sources[n] = {
            "spell_id": sid, "school": school,
            "dps": s["damage"] / T,
            "share": s["damage"] / total,
            "executes": s["executes"] * scale,
            "direct_hits": s["direct_hits"] * scale,
            "direct_noncrit_avg": s["direct_hit_damage"] / s["direct_noncrits"] if s["direct_noncrits"] else 0.0,
            "direct_crit_rate": s["direct_crits"] / s["direct_hits"] if s["direct_hits"] else 0.0,
            "ticks": s["ticks"] * scale,
            "tick_noncrit_avg": s["tick_hit_damage"] / s["tick_noncrits"] if s["tick_noncrits"] else 0.0,
            "tick_crit_rate": s["tick_crits"] / s["ticks"] if s["ticks"] else 0.0,
            "pet": pet,
            "seen_in_fights": int(s["fights"]),
        }
    bacc = defaultdict(lambda: defaultdict(float))
    for o in obs:
        for n, b in o["buffs"].items():
            for k in ("time", "starts", "refreshes", "dur_sum", "dur_n"):
                bacc[n][k] += b[k]
            bacc[n]["spell_id"] = b["spell_id"]
            bacc[n]["external"] = max(bacc[n]["external"], b["external"])
            bacc[n]["fights"] += 1
    buffs = {}
    for n, b in sorted(bacc.items()):
        starts = b["starts"] * scale
        buffs[n] = {
            "spell_id": int(b["spell_id"]),
            "uptime": b["time"] / T,
            "starts": starts,
            "refreshes": b["refreshes"] * scale,
            "interval": L / starts if starts else 0.0,
            "duration": b["dur_sum"] / b["dur_n"] if b["dur_n"] else 0.0,
            "external": bool(b["external"]),
            "seen_in_fights": int(b["fights"]),
        }
    cacc = defaultdict(float)
    cid = {}
    for o in obs:
        for n, c in o["casts"].items():
            cacc[n] += c["count"]
            cid[n] = c["spell_id"]
    stats = [char_stats(o["info"]) for o in obs if o["info"]]
    med = {k: statistics.median(s[k] for s in stats if s[k] is not None) for k in stats[0]} if stats else {}
    pets = defaultdict(float)
    for n, s in sources.items():
        pets[s["pet"] or "player"] += s["share"]
    return {
        "player": player,
        "dps": total / T,
        "fight_length": L,
        "buffed_stats": {"attributes": {k: med.get(k) for k in ("intellect", "strength", "agility")},
                         "stats": {k: v for k, v in med.items() if k.endswith("_rating")}},
        "sources": sources,
        "buffs": buffs,
        "casts": {n: {"spell_id": cid[n], "per_fight": v * scale, "cpm": v / T * 60}
                  for n, v in sorted(cacc.items(), key=lambda kv: -kv[1])},
        "damage_share_by_actor": dict(sorted(pets.items(), key=lambda kv: -kv[1])),
        "idle_fraction": sum(o["idle"] for o in obs) / T,
    }


# ---------------------------------------------------------------- commands

def cmd_survey(a):
    zone = gql("query($z:Int!){ worldData { zone(id:$z){ name encounters { id name } } } }", {"z": a.zone})["worldData"]["zone"]
    print(f"{zone['name']}: top {a.top} {a.spec} {a.cls} parses per encounter, difficulty {a.difficulty}")
    print(f"{'encounter':28s} {'id':>5s} {'top dps':>8s} {'dur':>5s} {'targets':>7s} {'main target share':>18s}")
    for enc in zone["encounters"]:
        _, rk = rankings(enc["id"], a.difficulty, a.cls, a.spec, a.top, a.refresh)
        shares, ntg, durs = [], [], []
        for r in rk:
            code, fid = r["report"]["code"], r["report"]["fightID"]
            rep = gql(REPORT, {"c": code, "f": [fid]})["reportData"]["report"]
            me = [x["id"] for x in rep["masterData"]["actors"] if x["type"] == "Player" and x["name"] == r["name"]]
            if not me:
                continue
            ents = gql(TARGETS, {"c": code, "f": [fid], "s": me[0]})["reportData"]["report"]["table"]["data"]["entries"]
            tot = sum(e["total"] for e in ents) or 1
            shares.append(max(e["total"] for e in ents) / tot)
            ntg.append(len(ents))
            durs.append(r["duration"] / 1000)
        if shares:
            print(f"{enc['name'][:28]:28s} {enc['id']:5d} {rk[0]['amount'] / 1e3:7.0f}k {statistics.median(durs):4.0f}s "
                  f"{statistics.median(ntg):7.0f} {statistics.median(shares):17.0%}")
    report_points()


def cmd_observe(a):
    db = None
    if (ROOT / "target" / "db2" / "TraitNode.csv").exists():
        sys.path.insert(0, str(ROOT / "tools"))
        from db2 import Db2
        db = Db2(str(ROOT / "target" / "db2"))
    else:
        print("no target/db2: talents recorded as raw entry ids (run tools/db2.py for names and loadout strings)")
    enc_name, rk = rankings(a.encounter, a.difficulty, a.cls, a.spec, a.n + a.spare, a.refresh)
    print(f"{enc_name}: {len(rk)} ranked {a.spec} {a.cls} parses")
    check_budget()
    obs, fights = [], []
    for i, r in enumerate(rk):
        if len(obs) >= a.n:
            break
        if spent["queries"] and spent["queries"] % 20 == 0:
            check_budget()
        try:
            f = load_fight(r)
        except Exception as ex:
            print(f"  skip {r['name']} {r['report']['code']}#{r['report']['fightID']}: {ex}")
            continue
        o = observe_fight(f)
        if o["aug"] and a.skip_aug:
            print(f"  skip {r['name']}: buffed by an Augmentation Evoker")
            continue
        if not o["info"]:
            print(f"  skip {r['name']}: no combatantinfo")
            continue
        obs.append(o)
        tal = talent_info(db, o["info"])
        pet_share = sum(s["damage"] for s in o["sources"].values() if s["pet"]) / o["damage"]
        fights.append({
            "rank": i + 1, "name": r["name"], "server": r["server"]["name"], "region": r["server"]["region"],
            "report": f["code"], "fight": f["fight"]["id"],
            "url": f"https://www.warcraftlogs.com/reports/{f['code']}#fight={f['fight']['id']}",
            "wcl_dps": r["amount"], "dps": o["dps"], "length": o["length"],
            "pet_share": pet_share, "ebon_might_share": o["ebon_might_share"], "idle_fraction": o["idle"] / o["length"],
            "external_buffs": [x["name"] for x in r.get("externalBuffs", [])],
            "stats": char_stats(o["info"]),
            **{k: v for k, v in tal.items()},
            "gear": simc_gear(o["info"]),
        })
        print(f"  {len(obs):2d}. {r['name'][:14]:14s} {o['dps'] / 1e3:6.1f}k (wcl {r['amount'] / 1e3:.1f}k) {o['length']:4.0f}s "
              f"pets {pet_share:4.0%}  {tal.get('hero_tree', '')}{'  aug' if o['aug'] else ''}"
              f"{'' if tal.get('loadout') else '  ' + str(tal.get('loadout_error'))}")
    if not obs:
        sys.exit("no fights")
    out = pool(obs, f"wcl_{snake(a.spec)}_{snake(enc_name)}")
    out["wcl"] = {
        "encounter": enc_name, "encounter_id": a.encounter, "difficulty": a.difficulty,
        "fights": len(obs), "class": a.cls, "spec": a.spec,
        "hero_trees": dict(Counter(f.get("hero_tree", "") for f in fights)),
        "note": "pooled over fights at the mean fight length; stats are the pre-pull character sheet (ratings), not buffed",
    }
    Path(a.out + "_observations.json").write_text(json.dumps(out, indent=1) + "\n")
    common = Counter(f.get("loadout") for f in fights if f.get("loadout")).most_common(1)
    Path(a.out + "_fights.json").write_text(json.dumps({
        "encounter": enc_name, "most_common_loadout": common[0][0] if common else None,
        "most_common_loadout_count": common[0][1] if common else 0,
        "talent_pick_rate": talent_rates(fights), "fights": fights}, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {a.out}_observations.json and {a.out}_fights.json ({len(obs)} fights)")
    print("damage share by actor: " + ", ".join(f"{k} {v:.1%}" for k, v in list(out["damage_share_by_actor"].items())[:8]))
    report_points()


def talent_rates(fights):
    c = Counter()
    for f in fights:
        c.update({t["name"] for t in f.get("talents", [])})
    return {k: v / len(fights) for k, v in sorted(c.items(), key=lambda kv: (-kv[1], kv[0]))}


def report_points():
    r = rate()
    print(f"API: {spent['queries']} queries, {spent['cached']} from cache; "
          f"{r['pointsSpentThisHour']:.0f}/{r['limitPerHour']} points used this hour (resets in {r['pointsResetIn']}s)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("survey", "observe"):
        p = sub.add_parser(name)
        p.add_argument("--class", dest="cls", default="Warlock")
        p.add_argument("--spec", default="Demonology")
        p.add_argument("--difficulty", type=int, default=5, help="5 mythic, 4 heroic")
        p.add_argument("--refresh", action="store_true", help="re-read rankings instead of the cached copy")
    s = sub.choices["survey"]
    s.add_argument("--zone", type=int, required=True)
    s.add_argument("--top", type=int, default=3)
    o = sub.choices["observe"]
    o.add_argument("--encounter", type=int, required=True)
    o.add_argument("-n", type=int, default=25, help="fights to use")
    o.add_argument("--spare", type=int, default=10, help="extra ranked parses to fall back on when one fails")
    o.add_argument("--skip-aug", action="store_true",
                   help="drop fights with Ebon Might on the player (external damage the warlock's hits carry)")
    o.add_argument("--out", required=True, help="output prefix, e.g. reference/demo_wcl")
    a = ap.parse_args()
    {"survey": cmd_survey, "observe": cmd_observe}[a.cmd](a)


if __name__ == "__main__":
    main()
