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

## Full kit: a real character against SimC

The app sims real characters with Raidbots-style settings: 300s Patchwerk, all
raid buffs, Bloodlust, and SimC's default consumables. The core-kit
calibration models none of the extras, so on the app's test character
(`validation/cases/retribution_paladin.txt` in dpsmaxxing, the MID2 sample
gear with its trinkets and tier set) azsim gave **144,343** where SimC gives
**218,843**. With consumables, Bloodlust, trinket effects and the tier set it
now gives 227,081 against a like-for-like 225,566 (below). SimC numbers below are 20,000 iterations, seed 1, ±~95 DPS.

### Where SimC's DPS comes from

Each extra switched off on its own, from the app's full SimC run:

| switched off | SimC DPS | contribution | share of full |
|---|---:|---:|---:|
| nothing (app settings) | 218,843 | | |
| Bloodlust (`override.bloodlust=0`) | 207,195 | 11,648 | 5.3% |
| potion (Light's Potential) | 211,471 | 7,372 | 3.4% |
| flask (Flask of the Magisters) | 212,376 | 6,467 | 3.0% |
| food (Royal Roast) | 216,147 | 2,696 | 1.2% |
| augment rune (Void-Touched) | 217,572 | 1,271 | 0.6% |
| weapon oil (Thalassian Phoenix Oil) | 217,550 | 1,293 | 0.6% |
| trinkets, stats and effects | 177,798 | 41,045 | 18.8% |
| — trinket effects only (stats kept) | 192,710 | 26,133 | 11.9% |
| tier set, 2pc + 4pc | 183,709 | 35,134 | 16.1% |
| — 4pc only | 197,326 | 21,517 | 9.8% |
| Bite of Zul'jan 2pc (the two trinkets) | 217,556 | 1,287 | 0.6% |
| everything above (core kit) | 127,156 | 91,687 | 41.9% |

The same steps stacked, from core kit up (they interact, so these sum exactly
to the total but differ from the one-at-a-time numbers):

| step | SimC DPS | gain |
|---|---:|---:|
| core kit, gear only | 127,156 | |
| + trinket stats | 138,232 | +11,076 |
| + trinket effects | 158,980 | +20,748 |
| + tier set | 190,529 | +31,549 |
| + flask, food, augment rune, oil | 201,307 | +10,778 |
| + potion | 207,244 | +5,937 |
| + Bloodlust | 218,843 | +11,599 |

**Setting differences.** The app's raid-buff overrides are SimC's Patchwerk
defaults, so they change nothing (core kit with them 131,738, the reference
without them 131,766). Fight length (300s ±20%) and Patchwerk agree. One
difference does matter: the app's test character has **no `omnium_talents`
line**, while the calibration reference (SimC's sample profile) has them.
Those traits are worth +3.6% in SimC (core kit 127,156 → 131,738; full
218,843 → 225,641) and are baked into azsim's calibrated multipliers. So for
this character azsim reads ~3.6% high before anything else, and the
like-for-like SimC target is 225,641.

### What azsim now models

`[gear.kit]` in the spec adds SimC's default consumables and Bloodlust on top
of the gear. It applies only when gear lines are given (the app always sends
them), is on by default, and switches off with
`--set gear.kit.enabled=0` or `set: {"gear.kit.enabled": 0}` in a batch
request. Runs without gear lines (calibration, validation, stat probes) never
see it, so their results are unchanged bit for bit.

| piece | how | source |
|---|---|---|
| food: +50 strength | flat primary stat before multipliers | [db2 1294727] |
| augment rune: +25.1 strength | flat primary stat | [db2 1264426] |
| flask: +165 mastery rating | rating before conversion | [db2 1235108], item-level scaled |
| weapon oil: +16 crit, +16 haste rating | rating before conversion | [obs] SimC buffed stats |
| potion: +695 strength for 30s, 300s cooldown | buff, cast in Avenging Wrath or the last 30s (1.5 per fight, as in SimC) | [db2 1236616], item-level scaled |
| Bloodlust: +30% haste for 40s at the pull | buff cast before the pull | [db2 2825] |

With the kit, azsim's stats for this character match SimC's buffed sheet
exactly (3,008 AP, 35.43% crit, 21.29% haste, 54.91% mastery).

### Before and after

| | azsim | SimC like-for-like | diff |
|---|---:|---:|---:|
| core kit + trinket stats (kit off) | 144,391 | 143,164 | +0.86% |
| + consumables and Bloodlust (kit on) | 165,924 | 165,025 | +0.54% |
| kit gain | +14.9% | +15.3% | |

"SimC like-for-like" is SimC with the same pieces azsim models: trinkets
replaced by their stats, no tier set, and `omnium_talents` on to match the
calibration. Through the app's batch path (seed 1, 0.2% target error):
**144,343 → 165,849**, against SimC's 218,843 for the full character.

### Trinkets and tier set

With gear lines azsim now also models the character's trinket effects and set
bonuses (see [gear.md](gear.md#trinkets-and-set-bonuses) for the effect
library):

- *Zul'jin's Guillotine Technique*: real-PPM physical proc, +2% damage per 1%
  of target health missing; with the *Bite of Zul'jan* 2pc (the trinket plus
  the main hand) it also applies Venomfang.
- *Voracious Heart of Ula'tek*: on use, +480 strength for 20s, used as SimC's
  APL does (in a fresh Avenging Wrath); meanwhile hits proc Devour Morsel,
  each stacking +54 strength.
- *MID2 2pc* (spec data in `[[gear.effects]]`, keyed by set bonus 1296660):
  consuming Divine Purpose grants Divine Power (+10% Holy damage, 12s), and
  Divine Purpose procs on 20% of spenders instead of 10% [db2 1296660 effect
  1]. Free casts run the spender's new `on_free` effects.
- *MID2 4pc* (1296661): consuming Divine Purpose arms a Divine Arbiter for
  the other spender; that spender (or Empyrean Legacy's free Divine Storm)
  spends it for 10.125 AP of Holy damage. With one armed by Final Verdict the
  rotation hard-casts Divine Storm ahead of Final Verdict, as SimC's APL.

Per piece, on the app's test character (20,000 iterations, seed 1; SimC
like-for-like = `omnium_talents` on, pieces azsim doesn't run replaced by
their stats or switched off with `set_bonus=...=0`):

| | azsim | SimC like-for-like | diff |
|---|---:|---:|---:|
| kit, trinkets as stats, no set bonuses (before) | 165,910 | 164,959 | +0.58% |
| + trinket effects | 189,914 | 188,312 | +0.85% |
| + set bonuses (trinkets as stats) | 199,841 | 198,708 | +0.57% |
| + both (the character as the app sends it) | **227,081** | 225,566 | +0.67% |
| gain from trinket effects | +24,004 | +23,353 | |
| gain from set bonuses | +33,931 | +33,749 | |

`--set gear.trinket_effects=0` and `--set gear.set_bonuses=0` give the rows
above; set bonuses here include Bite of Zul'jan's. Through the app's batch
path (seed 1, 0.2% target error) the character goes **165,849 → 226,932**,
against SimC's 218,846 for it as sent: +3.7%, the `omnium_talents` the
character lacks but the calibration bakes in (+3.6% in SimC, see above).

Against SimC's per-source breakdown, the new pieces land on the counts and
per-hit damage: Divine Arbiter 36.3 hits a fight (SimC 36.3) at 122,114 per
non-crit hit (121,950), Divine Storm 38.0 casts (38.3), Guillotine 24.6
(23.9) at 79,557 (79,456), Devour Morsel 35.8 (35.3) at 12,708 (12,734).
Two numbers are fitted, both `[obs]`: Divine Arbiter's residual multiplier
(1.54, the role the calibration plays for every other source) and its crit
damage (x2.22). Hard-cast Divine Storm takes the calibrated `divine_storm`
(Empyrean Legacy's) divided by Empyrean Legacy's +25%.

The bag trinkets, each swapped into either slot, against SimC scaled by the
same +3.76% `omnium_talents` offset: Gaze of the Alnseer and Twisted Horror's
Tendril within 0.1%, Font of Venomous Rage +0.7% to +1.1% (SimC's APL holds
it until the other trinket is used: 1.8 uses a fight against 2.0).

### What's left

- The character's `omnium_talents` (+3.6%) aren't passed through.
- **Non-trinket item effects count by their stats alone.** On this character
  Loa Worshiper's Band (finger2, a scripted proc that picks a loa by the
  socketed gem colours) is worth about 3,000 DPS in SimC; azsim keeps its
  stats only, and keeps the Ret spec's built-in Venomfang and Rune of
  Unleashed Fire procs whatever the gear. That is now most of the
  difference in Top Gear: the app's `scripts/compare_engines.py` runs this
  character's whole Top Gear on azsim (480 combinations, 4.3s against
  SimC's 6.7s) but picks Providence Watch plus Sickening Signet in place of
  that ring (+2,437; SimC −1,353), where SimC picks Providence Watch alone
  (SimC +1,028, azsim +1,859). Re-simmed precisely in both engines, the
  combinations that keep the ring rank alike.
- Divine Arbiter's multiplier is fitted to this character's log; a
  reference with the tier set on (or a log) would let the calibration fit it
  with the other sources.
