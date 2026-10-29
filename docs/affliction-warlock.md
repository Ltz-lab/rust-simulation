# Affliction Warlock (Hellcaller)

The Midnight Season 2 sample profile uses Hellcaller. In this version of the
profile, Wither replaces Corruption, Dark Harvest replaces Soul Rot, and
Unstable Affliction is the shard spender; Malefic Grasp is used during
Darkglare. Mechanics and coefficients come from DB2 and SimC's class module
and APL, with the remaining per-hit differences fitted to the committed core
observations. Hit counts are not fitted.

## Results

300-second Patchwerk, core kit without consumables, Bloodlust, trinkets or
set bonuses. The reference has 5,000 iterations; azsim has 10,000, seed 1.

| | DPS |
|---|---:|
| SimC | 157,898 |
| azsim | 157,573 ± 115 |
| difference | -0.21% |

| source | SimC hits/fight | azsim hits/fight |
|---|---:|---:|
| unstable_affliction | 255.1 | 249.1 |
| blackened_soul | 222.9 | 207.6 |
| agony | 286.5 | 284.9 |
| shadow_bolt | 107.6 | 113.1 |
| wither | 285.4 | 284.0 |
| haunt | 18.8 | 18.7 |
| darkglare_eye_beam | 55.3 | 54.8 |
| malefic_grasp | 69.9 | 70.2 |
| dark_harvest_tick | 21.5 | 21.5 |

## Held-out stat probes

The same per-hit calibration is used for all probes. Gains are relative to
each simulator's own baseline, using 20,000 paired azsim iterations.

| probe | SimC gain | azsim gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.70% | +8.40% | -0.30 points |
| +500 haste | +8.97% | +8.38% | -0.60 points |
| +500 mastery | +7.06% | +7.10% | +0.05 points |
| +500 versatility | +8.76% | +8.71% | -0.06 points |
| +200 intellect | +6.45% | +6.58% | +0.13 points |

## Gear and engine mechanics

The `[gear]` conversion matches every item level, rating and final stat in
the sample profile against SimC with trinkets, tier and consumables disabled.
`[gear.kit]` adds food, flask, rune, weapon oil, Bloodlust and a potion to
runs using gear lines. The shared item effect library handles supported
trinkets; Affliction-specific tier effects are not implemented.

The generic engine additions are channels that start their DoT at cast start
and snapshot speed, overlapping DoT stacks with independent expiry and expiry
effects, and an option to preserve a free-cast buff gained during a hard cast.
Their details and limitations are in [spec-format.md](spec-format.md).

## Known gaps

- Overlapping Unstable Affliction applications share one tick cadence; real
  applications have independent cadences. The model reports a combined tick
  scaled by active stacks, so per-hit calibration also reflects that grouping.
- Blackened Soul has about 7% fewer hits, Shadow Bolt about 5% more, and
  Shadow of Nathreza about 4% more. These differences affect source shares even
  though aggregate DPS is close. The Haunt periodic clock ignores travel time.
- Agony's shard accumulator, Nightfall and gear procs are approximated by
  independent rolls or rates. Sayaad energy and haste inheritance are reduced
  to observed average attack intervals. Gear stat procs are time-averaged in
  core comparisons. Haste probe response is 0.60 points below SimC.
- The model omits Affliction-specific tier effects; results with the full
  sample gear therefore are not a validated full-kit DPS comparison.
