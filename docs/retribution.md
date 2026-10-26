# Retribution Paladin (Herald of the Sun)

## Results

Reference: SimC's Midnight S2 sample Ret profile, core kit only (no trinkets,
tier set or consumables), 300s Patchwerk, 5,000 iterations.

| | DPS |
|---|---:|
| reference | 131,766 |
| azsim | 132,884 (±60) |
| difference | **+0.89%** |

## Structure: hit counts per fight

Calibration only adjusts damage *per hit*, so hit counts have to come out of
the engine's own mechanics and rotation. They match closely:

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| final_verdict | 40,061 | 41,061 | +2.5% | 160.1 | 164.2 |
| dawnlight | 17,830 | 17,931 | +0.6% | 211.8 | 213.1 |
| crusading_strike | 13,078 | 13,396 | +2.4% | 230.7 | 236.5 |
| hammer_of_wrath | 6,540 | 6,528 | -0.2% | 33.4 | 33.3 |
| execution_sentence | 5,915 | 5,979 | +1.1% | 5.2 | 5.3 |
| blade_of_justice_walk_into_light | 5,066 | 5,055 | -0.2% | 38.7 | 38.7 |
| suns_avatar | 4,447 | 4,473 | +0.6% | 211.8 | 213.1 |
| light_within_boj | 4,377 | 4,168 | -4.8% | 16.0 | 15.3 |
| highlords_judgment | 4,125 | 4,139 | +0.4% | 27.9 | 28.0 |
| light_within_boj_wil | 3,934 | 3,946 | +0.3% | 11.4 | 11.4 |
| blade_of_justice | 3,690 | 3,507 | -4.9% | 30.8 | 29.3 |
| expurgation | 3,151 | 3,140 | -0.4% | 138.9 | 138.4 |
| judgment | 2,611 | 2,600 | -0.4% | 27.2 | 27.2 |
| hammer_of_wrath_divine_resonance | 2,049 | 2,065 | +0.8% | 15.8 | 15.9 |
| truths_wake | 1,963 | 1,935 | -1.4% | 76.1 | 75.1 |
| hammer_of_wrath_divine_toll | 1,731 | 1,738 | +0.4% | 5.3 | 5.4 |
| venomfang | 1,298 | 1,315 | +1.3% | 167.1 | 169.4 |
| wake_of_ashes | 1,229 | 1,233 | +0.3% | 10.3 | 10.3 |
| sun_sear | 1,069 | 1,172 | +9.7% | 179.0 | 196.5 |
| execution_sentence_init | 1,097 | 1,138 | +3.8% | 5.2 | 5.5 |
| divine_storm | 1,100 | 1,099 | -0.0% | 5.4 | 5.4 |
| consecration_tick_blade_of_justice | 1,070 | 955 | -10.7% | 409.8 | 365.8 |
| rune_of_unleashed_fire | 1,028 | 1,028 | -0.0% | 50.3 | 50.4 |
| seething_flames_1 | 863 | 868 | +0.5% | 10.3 | 10.3 |
| seething_flames_0 | 864 | 868 | +0.5% | 10.3 | 10.3 |
| venomfang_burst | 561 | 561 | -0.1% | 38.7 | 38.7 |
| hammer_of_wrath_second_sunrise | 442 | 434 | -1.9% | 6.7 | 6.6 |
| rune_of_unleashed_fire_lingering | 324 | 323 | -0.2% | 242.0 | 241.6 |
| divine_storm_tempest | 147 | 147 | -0.2% | 5.4 | 5.4 |
| divine_storm_second_sunrise | 81 | 84 | +3.0% | 1.1 | 1.1 |
| divine_storm_second_sunrise_tempest | 32 | 33 | +2.5% | 1.1 | 1.1 |

Holy Power economy also matches: 502.9 generated per fight vs 500.1 in the
reference (81 vs 95 lost to overcapping).

## Held-out validation: stat sensitivity

The real test, since none of these runs were used for fitting. Each probe adds
stats in the reference; we apply the same change in final stats to our spec,
keep the calibration fixed, and compare DPS gains.

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.50% | +8.20% | -0.30 pts |
| +500 haste | +9.00% | +9.86% | +0.86 pts |
| +500 mastery | +9.33% | +9.25% | -0.08 pts |
| +500 vers | +8.79% | +8.82% | +0.03 pts |
| +200 strength | +6.05% | +6.10% | +0.05 pts |

Mastery, versatility and strength are within 0.1 points, crit within 0.3.
**Haste over-responds by 0.9 points**, enough to swap haste and mastery at the
top of the stat priority. See [Known gaps](#known-gaps).


## Speed

Same machine (2 cores), 10,000 iterations: azsim **1.6s**, SimC **25.3s**.
Treat this as indicative only, since azsim models less than SimC does.
Per-purpose random streams cost nothing measurable: on a 4-core container,
10,000 iterations take a median 0.73s with them and 0.73s with the old single
RNG (2.85s vs 2.82s single-threaded, medians of 8 runs).

## What the raw data alone gets you

With coefficients straight from the game tables and base mechanics only, the
spec reached **~35% of the reference DPS**. Adding the mechanics observed in
logs (crit damage, the weapon component of attack power, Execution Sentence
storing damage, and so on) brings the uncalibrated spec to **~50%**. The rest
is talent and aura multipliers encoded in effect/aura types that still need
interpreting. That gap is what the calibration overlay covers for now, and what
a proper effect-interpretation layer (or log-driven fitting at scale) has to close.

Several mechanics weren't visible in the base data at all and were found by
observing logs: Judgment's second charge, Dawnlight extending rather than
resetting, mastery doubling as the Highlord's Judgment proc chance, Avenging
Wrath shortening the GCD and speeding up Hammer of Wrath's recharge, Truth's
Wake extending on crit ticks, and DoTs critting harder than direct hits. The
stat probes caught three of these, which is exactly what held-out validation
is for.


## Known gaps

- **Haste over-response (+0.7 pts).** Concentrated in the Hammer of Wrath
  chain inside Avenging Wrath (it competes with Final Verdict for GCDs and
  Holy Power) and in Expurgation, whose damage *per tick* rises with haste in
  the reference, probably a rolling DoT that carries damage over on refresh.
- **Fitted rather than derived:** Avenging Wrath's haste/GCD/recharge effects,
  Judgment's cooldown, and the always-on haste procs are fitted to observed
  cadence. Blessing of An'she isn't modelled individually.
- **Small sources:** Consecration overlap (-11%) and Sun Sear (+10%) are each
  under 1% of damage.
