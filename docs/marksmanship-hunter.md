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

## Gear: trinket timing and raid debuffs

With `--gear` the spec follows SimC's trinket lines (`use_after = "trueshot"`,
`use_items`): Voracious Heart goes up with Trueshot and Font of Venomous Rage
only when Trueshot is 20s+ away. Left on cooldown, Font landed outside
Trueshot's crit window (49.7% crit against SimC's 68.8%) and Voracious Heart
was used 2.0 times a fight against 2.8. Library gear procs also take the raid
debuffs (`effect_mult`, as the caster specs do), which calibrated sources
already carry in their fitted multipliers. Sample profile, SimC 20,000
iterations, azsim 10,000 (seed 1):

| | SimC | azsim before | azsim after |
|---|---:|---:|---:|
| full profile, tier set off | 226,199 | 216,241 (-4.40%) | 222,670 (-1.56%) |
| full profile, tier set on (needs the Hunter tier branch) | 242,381 | 231,842 (-4.35%) | 238,016 (-1.80%) |

Font of Venomous Rage now reads high (23.9k against 19.3k DPS, crit 74% against
69%), covering a shortfall elsewhere (Aimed Shot, Master Marksman, Moonlight
Chakram and Lunar Storm are each 5-15% light, Steady Shot 40% heavy).

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

## Tier set (Midnight season 2, set 2059)

Modelled with gear lines, as the spec's `[[gear.effects]]` (`--set gear.set_bonuses=0`
turns it off); numbers from the set spells through the game data.

| piece | what it does | modelled as |
|---|---|---|
| 2pc (1296633) | Explosive Shot lasts 1s longer, its ticks (and direct damage) +5% | `add = { duration = 1 }` on its DoT, `ability_pct` buff (the direct +5% has nothing to act on) |
| 4pc (1296634) | Aimed Shot and Rapid Fire +5%; each Explosive Shot tick takes 0.5s off Aimed Shot's and Rapid Fire's cooldowns | `ability_pct` buff, `reduce_cooldown` on the DoT's tick |

Profile `MID2_Hunter_Marksmanship` (300s Patchwerk, one target, full kit, seed 1; SimC 600,000 iterations
cap with target error 0.1%, azsim 100,000):

| | SimC | azsim |
|---|---:|---:|
| tier gain (full / set bonuses off) | +7.13% | +7.24% |
| 2pc alone / 4pc on top (partial runs, +-0.2) | +3.52% / +3.46% | +2.69% / +4.40% |
| full-profile DPS | 242,273 | 231,896 (-4.28%; off: -4.38%) |

The two pieces are off by about 0.8 points each, in opposite directions. In SimC the 2pc's extra
Explosive Shot tick is worth more than an average tick: per-tick damage rises 12.7% (azsim +5.7%, the spell's
5%), and its debug log shows a per-tick multiplier that grows along a cast (1.0, 1.4, 1.92 in one sample), which
azsim's flat per-tick calibration can't follow. The 4pc's cooldown reduction buys azsim more Rapid Fire casts than
SimC (+10.8% against +8.4%).
