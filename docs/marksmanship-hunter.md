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
- SimC counts 40 crit rating more than the sample profile's items and gems
  sum to (1005 against 1045; every item's own ratings match), folded into
  `crit_bonus` as Beast Mastery does with its 27. The cause is not found.
- The crit probe is the weakest (-1.89 points).

## Gear

`[gear]`, `[gear.kit]` and `[gear.omnium]` turn `profiles/MID2_Hunter_Marksmanship.simc`
into stats. Measured on SimC (trinkets and tier set removed, one iteration for
stats):

| constant | value | from |
|---|---|---|
| base agility, multiplier | 618, x1.08145 | unbuffed agility 2526 on 2336 (same as Beast Mastery) |
| attack power multiplier | 1.05 | Battle Shout, 2526 agility -> 2652 |
| crit bonus | 0.148696 | 36.72% at 1045 rating (items sum to 1005) |
| haste multiplier | 1.02 | 11.55% at 412 rating |
| mastery | 10 points, 0.625 each | +92 and +460 rating moved mastery 0.0125 and 0.0615; 8 base + 2 buff |
| versatility bonus | 0.03 | Mark of the Wild |
| weapon attack power per DPS | 6.0 | 934.77 at 155.8 DPS, as `[player]` |
| spec id | 254 | ChrSpecialization |

Kit, from SimC's stats with each consumable alone: food +54 agility, augment
rune +27, Flask of the Shattered Sun +172 crit, Thalassian Phoenix Oil +17 crit
and +16 haste (the three together +188 crit, SimC reports 1233 against 1045),
then Bloodlust at the pull. Potion of Recklessness is +1725 crit and -232 haste
for 30s, used as the APL does: `buff.trueshot.up&(buff.bloodlust.up|...)`.
Buffed stats, azsim against SimC: attack power 2737 / 2737, crit 40.80% /
40.80%, haste 11.92% / 11.92%, mastery 19.77% / 19.77%, versatility 14.63% /
14.63%.

Omnium (`tools/omnium.py ablate`, 20,000 iterations): 155,386 with all five
traits, 149,630 with none, +3.85%; Unleashed Fire 1.02016, Critical Power
1.01498, Overload 1.00174, Lingering 1.00112, Self-Mending 1.00006.

DPS (300s Patchwerk, no trinkets, tier set off):

| | SimC | azsim |
|---|---:|---:|
| core kit (`check_gear.py --dps --no-trinkets`, 5,000) | 155,380 | 157,037 (+1.07%) |
| full kit (SimC 20,000, azsim 10,000, seed 1) | 180,080 | 177,053 (-1.68%) |
