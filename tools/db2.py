"""Shared DB2 loading for the tools: CSV exports from wago.tools/db2/<Table>/csv.

  python3 tools/db2.py [--build 12.1.5.70575] [--cache target/db2]   fetch what talents.py/effects.py read

  fetch_tables(cache, build, names)  download whatever is missing
  Db2(dir)                           lazy table access, rows by key
"""
import csv
import os
import urllib.request
from collections import defaultdict

csv.field_size_limit(10**9)


def fetch_tables(cache, build, names):
    os.makedirs(cache, exist_ok=True)
    for t in names:
        path = f"{cache}/{t}.csv"
        if os.path.exists(path):
            continue
        url = f"https://wago.tools/db2/{t}/csv?build={build}"
        print(f"fetching {url}")
        req = urllib.request.Request(url, headers={"User-Agent": "azsim-db2"})
        with urllib.request.urlopen(req, timeout=300) as r, open(path + ".tmp", "wb") as f:
            f.write(r.read())
        os.replace(path + ".tmp", path)


def num(x, default=0.0):
    try:
        return float(x)
    except (TypeError, ValueError):
        return default


class Db2:
    def __init__(self, d):
        self.d = d
        self._cache = {}

    def rows(self, table):
        key = ("rows", table)
        if key not in self._cache:
            with open(f"{self.d}/{table}.csv", newline="", encoding="utf-8") as f:
                self._cache[key] = list(csv.DictReader(f))
        return self._cache[key]

    def by(self, table, field="ID"):
        """field value (int) -> first row."""
        key = ("by", table, field)
        if key not in self._cache:
            out = {}
            for r in self.rows(table):
                out.setdefault(int(num(r[field])), r)
            self._cache[key] = out
        return self._cache[key]

    def multi(self, table, field):
        """field value (int) -> all rows."""
        key = ("multi", table, field)
        if key not in self._cache:
            out = defaultdict(list)
            for r in self.rows(table):
                out[int(num(r[field]))].append(r)
            self._cache[key] = out
        return self._cache[key]

    def name(self, spell_id):
        return self.by("SpellName").get(spell_id, {}).get("Name_lang", "?")


TABLES = [
    "SpellName", "SpellEffect", "SpellMisc", "SpellClassOptions", "SpellLabel", "SpecializationSpells",
    "ChrSpecialization", "TraitTreeLoadout", "SkillLineXTraitTree", "TraitNode", "TraitNodeXTraitNodeEntry",
    "TraitNodeEntry", "TraitDefinition", "TraitSubTree",
]

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", default="12.1.5.70575")
    ap.add_argument("--cache", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "db2"))
    a = ap.parse_args()
    fetch_tables(a.cache, a.build, TABLES)
