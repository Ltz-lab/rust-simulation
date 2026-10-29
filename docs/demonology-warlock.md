# Demonology Warlock (Diabolist)

`specs/demonology_diabolist.toml` models the Midnight S2 sample profile against
`reference/demo_core_observations.json`, a 300-second single-target SimC core
kit run. DB2 build 12.1.5.70575, the SimC warlock module and APL, and observed
SimC per-hit damage supply the inputs. Calibration fits damage per hit, not
counts. The 25-fight Warcraft Logs sample is a cross-check, not the fit target;
its different encounters and gear are not directly comparable.

| | DPS |
|---|---:|
| SimC | 160,301 |
| azsim, 10,000 iterations, seed 1 | 158,236 ± 70 |
| Difference | -1.29% |

| Source | SimC hits/fight | azsim hits/fight |
|---|---:|---:|
| Wild Imp Fel Firebolt | 1,908.7 | 1,850.2 |
| Demonic Tyrant Burning Cleave | 65.6 | 65.2 |
| Demonbolt | 60.4 | 60.2 |
| Hand of Gul'dan impact | 87.1 | 89.5 |
| Dreadbite | 96.2 | 88.6 |
| Vilefiend Gloom Slash | 188.2 | 174.6 |
| Implosion | 81.4 | 79.5 |

Held-out probes use the same calibration and 20,000 paired iterations:

| Probe | SimC gain | azsim gain | Difference |
|---|---:|---:|---:|
| +500 crit | +8.63% | +8.05% | -0.58 points |
| +500 haste | +8.20% | +8.94% | +0.74 points |
| +500 mastery | +7.44% | +7.51% | +0.07 points |
| +500 versatility | +8.86% | +8.59% | -0.27 points |
| +200 intellect | +6.53% | +6.46% | -0.08 points |

Gear parsing matches all item levels and ratings in the sample profile; its
unprocced spell power differs by one point and the four percentages match to
two decimal places. The gear kit supplies consumables, Bloodlust and supported
item effects; it is separate from the core comparison above.

## Model and known gaps

The shard economy, summons, pet casts and swings, energy consumption, Implosion,
Tyrant, Diabolic Ritual and Dominion of Argus are represented in data. The
engine additions are generic pet energy regeneration, per-summon energy and
damage overrides, sampled dismissal, start-of-cast pet cooldowns, swing pauses,
and shared buff timers that can be preserved or shortened.

This is an approximation of the full SimC implementation. The guardians'
action timing and selection produce 68.6 Inquisitor hits versus 45.4, 41.1
Jailer hits versus 56.4, and roughly 28 hits each for Alythess and Sacrolash
versus 34. Diabolic Gaze's shared clock overcounts its first two variants;
Dreadstalker and Vilefiend activity is somewhat low. Imp boss allocation,
initial ritual stage, Oculi timing, and some Tyrant interactions are simplified.
These differences, together with time-averaged stat procs, contribute to the
haste and crit probe gaps. Felstorm is represented as a single six-hit action
at the end of its cast. Per-hit residuals include effects not separately
modelled, especially Overlord's coefficient. The WCL sample is not a validation
of this particular profile or its single-target rotation.
