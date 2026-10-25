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
    assert 100_000 < r["dps"] < 200_000
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
