#!/usr/bin/env python3
"""Tests for tools/draft_spec.py:  python3 tools/test_draft_spec.py

The DB2 half runs on a tiny SYNTHETIC table set written to a temp directory
(made-up spells, columns as in the wago.tools exports), because the real
tables were not reachable when this was written. It checks the plumbing,
not the game's numbers. The APL half runs on the real profiles in profiles/.
"""
import csv
import glob
import os
import sys
import tempfile
import tomllib
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import draft_spec as ds  # noqa: E402

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")


def tr(cond, **kw):
    return ds.translate(cond, ds.Ctx(**kw))


class Translate(unittest.TestCase):
    def test_basic(self):
        self.assertEqual(tr("buff.a.up&!buff.b.up|cooldown.c.remains<gcd"), "buff.a.up&!buff.b.up|cooldown.c.remains<gcd")
        self.assertEqual(tr("buff.a.down"), "!buff.a.up")
        self.assertEqual(tr("cooldown.c.up&buff.a.react"), "cooldown.c.ready&buff.a.stack")
        self.assertEqual(tr("holy_power.deficit<=2"), "holy_power.max-holy_power<=2")
        self.assertEqual(tr("target.time_to_die<25"), "fight_remains<25")

    def test_precedence_kept(self):
        self.assertEqual(tr("(buff.a.up|buff.b.up)&buff.c.up"), "(buff.a.up|buff.b.up)&buff.c.up")

    def test_constants(self):
        t = lambda n: {"yes": True, "no": False}.get(n)
        self.assertEqual(tr("talent.yes&buff.a.up", talent=t), "buff.a.up")
        with self.assertRaises(ds.NeverFires):
            tr("talent.no&buff.a.up", talent=t)
        with self.assertRaises(ds.Untranslatable):
            tr("talent.unknown", talent=t)
        self.assertEqual(tr("active_enemies>1|buff.a.up"), "buff.a.up")
        self.assertEqual(tr("hero_tree.herald_of_the_sun", hero="Herald of the Sun"), "")

    def test_variables_and_self(self):
        cx = ds.Ctx(variables={"v": "buff.a.up&rage>50"}, action="hammer")
        self.assertEqual(ds.translate("variable.v|full_recharge_time<gcd", cx),
                         "buff.a.up&rage>50|cooldown.hammer.full_recharge_time<gcd")

    def test_unsupported_is_flagged_not_guessed(self):
        for c in ("trinket.1.has_use_buff", "a%%2=0", "x>?3", "debuff.x.up", "equipped.foo", "@buff.a.stack>1"):
            with self.assertRaises(ds.Untranslatable, msg=c):
                tr(c)


class Apl(unittest.TestCase):
    APL = """
actions.precombat=snapshot_stats
actions=auto_attack
actions+=/call_action_list,name=cds
actions+=/run_action_list,name=aoe,if=active_enemies>2
actions+=/judgment,if=buff.x.up
actions.cds=avenging_wrath,if=cooldown.y.ready
actions.cds+=/potion
actions.aoe=divine_storm
"""

    def test_flatten(self):
        rows, skipped, _ = ds.build_rotation(ds.parse_apl(self.APL), lambda a: ds.Ctx(action=a))
        live = [(r["action"], r.get("if"), r["status"]) for r in rows]
        self.assertEqual(live[0], ("avenging_wrath", "cooldown.y.ready", "ok"))
        self.assertEqual(live[1][2], "pruned")     # divine_storm: 3+ targets never happens
        self.assertEqual(live[2], ("judgment", "buff.x.up", "ok"))   # the run_action_list guard folds to true
        self.assertIn("potion", skipped)

    def test_real_profiles(self):
        """Every profile with an APL flattens, and the output re-parses in the engine grammar."""
        total = bad = 0
        for p in glob.glob(f"{ROOT}/profiles/*.simc"):
            prof = ds.read_profile(p)
            rows, _, _ = ds.build_rotation(ds.parse_apl(prof["text"]), lambda a: ds.Ctx(action=a))
            for r in rows:
                if r["status"] == "ok":
                    ds.parse(r["if"] or "0")
            total += sum(r["status"] != "pruned" for r in rows)
            bad += sum(r["status"] == "todo" for r in rows)
        self.assertGreater(total, 300)
        print(f"\n  real profiles: {total} rotation entries, {bad} TODO without talent info")


COLS = {
    "SpellName": "ID,Name_lang",
    "SpellEffect": "SpellID,EffectIndex,DifficultyID,Effect,EffectAura,EffectBasePointsF,BonusCoefficientFromAP,"
                   "EffectBonusCoefficient,EffectMiscValue_0,EffectMiscValue_1,EffectAuraPeriod,EffectTriggerSpell,"
                   "EffectSpellClassMask_0,EffectSpellClassMask_1,EffectSpellClassMask_2,EffectSpellClassMask_3",
    "SpellMisc": "SpellID,DurationIndex,CastingTimeIndex,Attributes_0,SchoolMask,Speed",
    "SpellCooldowns": "SpellID,RecoveryTime,CategoryRecoveryTime,StartRecoveryTime",
    "SpellCategories": "SpellID,ChargeCategory",
    "SpellCategory": "ID,MaxCharges,ChargeRecoveryTime",
    "SpellPower": "SpellID,ManaCost,PowerType",
    "SpellAuraOptions": "SpellID,CumulativeAura",
    "SpellDuration": "ID,Duration",
    "SpellCastTimes": "ID,Base",
    "SpellClassOptions": "SpellID,SpellClassSet,SpellClassMask_0,SpellClassMask_1,SpellClassMask_2,SpellClassMask_3",
    "SpellLabel": "SpellID,LabelID",
    "SpecializationSpells": "SpecID,SpellID",
}


def write_fixture(d):
    def w(table, rows):
        with open(f"{d}/{table}.csv", "w", newline="") as f:
            f.write(COLS[table] + "\n")
            csv.writer(f).writerows(rows)
    for t in COLS:
        w(t, [])
    w("SpellName", [(1, "Big Hit"), (2, "Big Hit Buff"), (3, "Charged Shot")])

    def e(sid, i, eff, aura, base, ap, sp, m0=0):
        return (sid, i, 0, eff, aura, base, ap, sp, m0, 0, 0, 0, 0, 0, 0, 0)
    w("SpellEffect", [e(1, 0, 2, 0, 0, 2.5, 0), e(1, 1, 30, 0, 1, 0, 0, 9), e(2, 0, 6, 108, 20, 0, 0), e(3, 0, 2, 0, 0, 0, 1.2)])
    w("SpellMisc", [(1, 0, 0, 0, 2, 0), (2, 7, 0, 0, 1, 0), (3, 0, 5, 0, 4, 0)])
    w("SpellCooldowns", [(1, 0, 6000, 1500), (3, 0, 0, 1500)])
    w("SpellPower", [(1, 3, 9), (3, 200, 1)])
    w("SpellAuraOptions", [(2, 3)])
    w("SpellDuration", [(7, 12000)])
    w("SpellCastTimes", [(5, 2500)])
    w("SpellCategories", [(3, 4)])
    w("SpellCategory", [(4, 2, 8000)])
    w("SpecializationSpells", [(70, 1), (70, 3)])


class Db2Layer(unittest.TestCase):
    def test_synthetic(self):
        with tempfile.TemporaryDirectory() as d:
            write_fixture(d)
            db = ds.Db2Source(d)
            f, _ = db.ability(1)
            self.assertEqual(f["cooldown"], 6.0)
            self.assertEqual(f["cost"], {"holy_power": 3.0})
            self.assertEqual(f["gain"], {"holy_power": 1.0})
            self.assertEqual(f["damage"], {"school": "holy", "ap_coeff": 2.5})
            g, _ = db.ability(3)
            self.assertEqual((g["charges"], g["cooldown"], g["cast_time"]), (2, 8.0, 2.5))
            self.assertEqual(g["cost"], {"rage": 20.0})        # rage is stored in tenths
            self.assertEqual(g["damage"], {"school": "fire", "sp_coeff": 1.2})
            self.assertEqual(db.pick("big_hit_buff", aura=True)[0], 2)
            b = db.buff(2)
            self.assertEqual((b["duration"], b["max_stacks"]), (12.0, 3))

    def test_emit_is_valid_toml(self):
        with tempfile.TemporaryDirectory() as d:
            write_fixture(d)
            prof = {"player": "X", "class": "paladin", "spec": "retribution"}
            lists = ds.parse_apl("actions=big_hit,if=buff.big_hit_buff.up&holy_power>=3\nactions+=/charged_shot\n")
            rows, sk, res = ds.build_rotation(lists, lambda a: ds.Ctx(action=a))
            text = ds.emit(prof, rows, sk, res, ds.Db2Source(d), None, False)
            spec = tomllib.loads(text)
            self.assertEqual([a["name"] for a in spec["abilities"]], ["big_hit", "charged_shot"])
            self.assertEqual(spec["buffs"][0]["duration"], 12.0)
            self.assertEqual(spec["rotation"][0]["if"], "buff.big_hit_buff.up&holy_power>=3")
            out = ds.diff_specs(spec, spec)
            self.assertFalse([x for x in out if ".cooldown" in x or ".damage" in x])


class EndToEnd(unittest.TestCase):
    def test_cli_without_apl_uses_spec_spells(self):
        import subprocess
        with tempfile.TemporaryDirectory() as d:
            write_fixture(d)
            prof = os.path.join(d, "p.simc")
            with open(prof, "w") as f:
                f.write('paladin="Fake_Herald"\nspec=retribution\n')
            out = os.path.join(d, "out.toml")
            r = subprocess.run([sys.executable, os.path.join(ROOT, "tools", "draft_spec.py"), prof, "--db2", d, "--spec-id", "70", "--out", out],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            with open(out, "rb") as f:
                spec = tomllib.load(f)
            self.assertEqual(sorted(a["name"] for a in spec["abilities"]), ["big_hit", "charged_shot"])


class OutputIsToml(unittest.TestCase):
    def test_no_db2_output_parses(self):
        prof = ds.read_profile(f"{ROOT}/profiles/MID2_Warrior_Fury.simc")
        rows, sk, res = ds.build_rotation(ds.parse_apl(prof["text"]), lambda a: ds.Ctx(action=a))
        tomllib.loads(ds.emit(prof, rows, sk, res, None, None, False))


if __name__ == "__main__":
    unittest.main()
