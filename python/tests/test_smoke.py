"""Smoke test for the azsim Python module (run after `maturin develop --release`)."""
import pathlib

import azsim

ROOT = pathlib.Path(__file__).resolve().parents[2]
GEAR = [l.strip() for l in (ROOT / "profiles/MID2_Paladin_Retribution.simc").read_text().splitlines() if ",id=" in l]


def test_specs():
    assert "retribution_herald" in azsim.spec_names()
    assert azsim.supports_gear("retribution_herald")


def test_sim():
    r = azsim.sim("retribution_herald", gear_lines=GEAR, iterations=500, seed=3)
    assert 100_000 < r["dps"] < 300_000   # full kit: trinkets, tier, consumables, Bloodlust
    assert r["dps_error_95"] > 0 and r["iterations"] == 500
    assert r["dps"] == azsim.sim("retribution_herald", gear_lines=GEAR, iterations=500, seed=3)["dps"]


def test_sim_batch_paired():
    mh = next(l for l in GEAR if l.startswith("main_hand="))
    r = azsim.sim_batch(
        "retribution_herald", GEAR,
        [{"name": "low", "gear": [mh.replace("ilevel=344", "ilevel=320")]}, {"name": "same"}],
        iterations=500, seed=3,
    )
    low, same = r["variants"]
    assert low["delta"] < 0 and same["delta"] == 0
    assert low["delta_error_95"] < r["base"]["dps_error_95"]


def test_bad_spec():
    try:
        azsim.sim("nope")
    except ValueError:
        return
    raise AssertionError("expected ValueError")


def test_target_error_pct():
    r = azsim.sim_batch("retribution_herald", GEAR, [{"name": "same"}], target_error_pct=0.1, max_iterations=20_000)
    assert r["iterations"] == 2_000


def test_gear_stats():
    rep = azsim.gear_stats("retribution_herald", GEAR)
    assert {i["slot"] for i in rep["items"]} >= {"head", "main_hand"}
    assert rep["gear"]["crit_rating"] > 0


def test_breakdown():
    r = azsim.sim_batch("retribution_herald", GEAR, [], iterations=300, seed=3)
    src = r["sources"]
    assert src[0]["damage"] >= src[-1]["damage"] > 0
    assert abs(sum(s["dps"] for s in src) / r["base"]["dps"] - 1) < 0.01
    fv = next(s for s in src if s["id"] == "final_verdict")
    assert fv["name"] == "Final Verdict" and fv["casts"] > 0 and 0 < fv["crit_pct"] < 1
    aw = next(b for b in r["buffs"] if b["id"] == "avenging_wrath")
    assert aw["spell_id"] == 31884 and 0 < aw["uptime"] < 1 and aw["count"] > 0
    assert 250 < r["fight_length"] < 350
