"""Synthetic checks of effects.py's debuff and rank-curve paths (no DB2 needed):
  python3 tools/test_effects.py
The spell ids and numbers are made up; they test the mechanics, not game data."""
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import effects as fx


def eff(i, aura, pts, mask=(0, 0, 0, 0), effect=6):
    d = {"EffectIndex": i, "Effect": effect, "EffectAura": aura, "EffectBasePointsF": pts, "EffectMiscValue_0": 0,
         "EffectMiscValue_1": 0, "EffectTriggerSpell": 0}
    d.update({f"EffectSpellClassMask_{k}": m for k, m in enumerate(mask)})
    return d


class Stub(fx.Spells):
    def __init__(self):
        self.e = {1: [eff(0, 3, 0, effect=2)], 2: [eff(0, 3, 0, effect=2)],
                  9: [eff(0, 271, 20, (0x2, 0, 0, 0)), eff(1, 271, 5)]}
        self.fam = {1: (53, (0x2, 0, 0, 0)), 2: (53, (0x4, 0, 0, 0)), 9: (53, (0, 0, 0, 0))}
        self.labels = defaultdict(set)

    def effs(self, sid):
        return self.e.get(sid, [])

    def family(self, sid):
        return self.fam.get(sid, (0, (0, 0, 0, 0)))


sp = Stub()
mods, unres = fx.debuff_mods(sp, [9])
assert len(mods) == 1 and len(unres) == 1                      # empty-mask effect is listed, not applied
assert abs(fx.derive(sp, mods, 1, False)[0] - 1.2) < 1e-9      # masked spell gets +20%
assert fx.derive(sp, mods, 2, False)[0] == 1.0                 # other family member does not


class C(fx.RankCurves):
    def __init__(self):
        self.by_def = {7: {0: (0, 1), 1: (1, 2)}}
        self.points = {1: [(0, 1, 10.0), (1, 3, 30.0)], 2: [(0, 1, 1.0), (1, 2, 2.0)]}


c = C()
assert c.value(7, 0, 2, 5) == 20.0 and c.value(7, 0, 9, 5) == 30.0   # interpolate, clamp
assert c.value(7, 1, 2, 5) == 10.0 and c.value(7, 2, 1, 5) is None   # multiply op; no curve
assert fx.scaled(c, {5: 7}, 5, {"EffectIndex": 0}, 2, 5) == 20.0
assert fx.scaled(c, {5: 7}, 6, {"EffectIndex": 0}, 2, 5) == 10     # no definition: linear

# scripted dummies: the number is the effect's points, rank-scaled; the sources come from the data file
import tempfile
sp.e[20] = [eff(0, 4, 75)]
with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as f:
    f.write("[[effect]]\nspell = 20\nindex = 0\nsources = [1]\nsimc = 'test'\n"
            "[[effect]]\nspell = 21\nindex = 0\nsources = [1]\n"          # not in effect
            "[[effect]]\nspell = 20\nindex = 3\nsources = [1]\n")          # no such effect
sc = fx.scripted_effects(sp, {20: (1, "talent")}, f.name)
assert len(sc) == 1 and sc[0]["value"] == 75 and sc[0]["sources"] == {1}
assert fx.scripted_effects(sp, {20: (2, "talent")}, f.name)[0]["value"] == 150
assert fx.dummy_report(sp, {20: (1, "talent")}, sc) == []           # applied ones leave the "not derived" list
assert fx.dummy_report(sp, {20: (1, "talent")}) == [(20, 0, 75.0)]
os.unlink(f.name)
print("ok")
