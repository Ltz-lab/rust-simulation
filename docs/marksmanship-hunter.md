# Marksmanship Hunter (Sentinel)

`specs/marksmanship_sentinel.toml`, built like the other specs: coefficients,
costs, cooldowns, durations and talent values from DB2, SimC's hunter module
(`sc_hunter.cpp`) and default APL (`apl/hunter/mm.txt`) where a mechanic is
scripted, the rest observed in SimC's log. Residual multipliers live in
`calibration/marksmanship_sentinel.toml` (tools/calibrate.py), the
DB2-derived ones in `derived/marksmanship_sentinel.toml` (tools/effects.py).

## Results

Reference: SimC's Midnight S2 sample Marksmanship profile (Sentinel), core
kit only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk.

| | DPS |
|---|---:|
| reference | 155,473 |
| azsim (`-n 10000 --seed 1`, with calibration) | 156,903 |
| difference | **+0.92%** |

Hit counts are within 5% of the reference on every source except Steady Shot
(see the known gaps).

## Held-out validation: stat sensitivity

Difference between azsim's and the reference's DPS gain from each probe, in
percentage points: crit -1.89, haste -0.15, mastery +0.47, vers +0.14,
agility +0.40. Probe references are `reference/mm_probe_*.json`.

## What is modelled

- Focus (125 cap, 5/s hasted), Steady Shot +25, Rapid Fire +3 a tick.
- SimC's Sentinel single-target APL: Explosive Shot on cooldown (Unstable
  Trigger's free second cast), Volley and Salvo, Trueshot and its cast-time
  cut, Moonlight Chakram, Rapid Fire's 10-tick channel (Quick Draw), Kill
  Shot and Arcane Shot spending Precise Shots, Aimed Shot charges with hasted
  recharge, Steady Shot as filler, free instant Aimed Shot under Lock and Load.
- Rapid Fire suppresses Auto Shot while it channels
  (`channel.interrupt_auto_attack`, SimC's rule for ranged auto attacks).
- Sentinel's Mark at 40% (30% Spotter's Mark + 10% Moon's Blessing), 90%
  during Trueshot, consumed by the next Aimed Shot to call down Lunar Storm.
- Focused Aim (1s off Aimed Shot's recharge per Precise Shots spend), Moon's
  Blessing at 1s per mark, Stargazer with independent per-stack timers,
  Critical Precision, Bulletstorm, Lock and Load.
- Mastery: Sniper Training at the reference's measured value per point, with
  Master Marksman and Sanctified Armaments (banked DoTs) scaling with it;
  Blood Fletching procs on every hit. Crit multiplier 2.41, from the
  reference's own crit/noncrit ratio.

## Known gaps

- Steady Shot is cast ~35 times a fight against the reference's ~25. The
  rotation is GCD-locked in both sims, and SimC leaves ~0.2s of slack between
  an action and the next, where this engine starts the next action
  immediately. That is ~14s a fight, which this spec spends on its filler.
  Fixing it needs a generic "time between actions" knob in the engine.
- No `[gear]` section yet, so gear lines can't be turned into stats for this
  spec.
- The crit probe is the weakest (-1.89 points).
