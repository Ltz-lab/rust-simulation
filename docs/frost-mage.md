# Frost Mage (Spellslinger)

`specs/frost_mage_spellslinger.toml`, built the same way as the other specs:
coefficients, cast times, charges and durations from DB2 (`tools/spelldb.py`),
the rotation from SimC's default Spellslinger APL, and the rest observed in
SimC's log. Frost is a shatter spec: Frostbolt, Flurry, Glacial Spike, Ray of
Frost and splinters stack **Freezing** on the target, and Ice Lance spends it
(6 stacks, 12 while Thermal Void is up) for a hit several times its own size.
The Spellslinger splinter chain (Force of Will, Augury Abounds, Splinterstorm)
is most of the hit count and about a sixth of the damage.

## Results

Reference: SimC's Midnight S2 sample Frost profile (Spellslinger), core kit only
(no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations.

| | DPS |
|---|---:|
| reference | 149,542 |
| azsim | 151,256 (±74) |
| difference | **+1.15%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| shatter_ice_lance | 46,448 | 48,193 | +3.8% | 124.4 | 129.4 |
| frost_splinter | 21,383 | 21,526 | +0.7% | 1300.3 | 1309.9 |
| ray_of_frost | 20,149 | 20,276 | +0.6% | 63.5 | 64.0 |
| frozen_orb_bolt | 13,294 | 13,238 | -0.4% | 338.2 | 336.9 |
| flurry_bolt | 12,801 | 12,282 | -4.1% | 161.1 | 154.7 |
| glacial_spike | 11,974 | 11,933 | -0.3% | 14.7 | 14.7 |
| ice_lance | 8,258 | 8,584 | +3.9% | 124.4 | 129.5 |
| hand_of_frost | 6,876 | 6,919 | +0.6% | 62.2 | 62.5 |
| frostbolt | 5,406 | 5,382 | -0.4% | 35.6 | 35.4 |
| rune_of_unleashed_fire | 1,033 | 1,034 | +0.0% | 50.3 | 50.4 |
| void_eruption | 771 | 769 | -0.3% | 28.9 | 28.9 |
| acidbrood_scourge | 767 | 765 | -0.2% | 21.6 | 21.6 |
| rune_of_unleashed_fire_lingering | 313 | 287 | -8.2% | 243.9 | 224.1 |
| **total** | **149,542** | **151,256** | **+1.15%** | | |

Cast counts behind those hits: Flurry 38.7 vs 40.3, Frozen Orb 11.5 vs 11.6,
Ray of Frost 8.0 vs 8.0, Glacial Spike 14.7 vs 14.7, Frostbolt 35.4 vs 34.6,
Hand of Frost 62.5 vs 62.5, splinters 1310 vs 1304. Fingers of Frost procs
34.3 (45.9 charges with Frozen Orb's), Brain Freeze 29.5, Augury Abounds 32
procs. Crit rates match source by source (Shatter 50.4% vs 50.5%, Ice Lance
33.5% vs 33.6%, Flurry 33.6% vs 33.6%), which is the point of modelling the
crit chain rather than averaging it.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +10.35% | +10.10% | -0.26 pts |
| +500 haste | +7.25% | +7.90% | +0.65 pts |
| +500 mastery | +9.03% | +9.73% | +0.71 pts |
| +500 versatility | +8.68% | +8.79% | +0.11 pts |
| +200 intellect | +6.31% | +6.66% | +0.35 pts |

The stat priority order (crit > mastery > vers > haste > int) matches the
reference.

## What's modelled

- **Freezing / Shatter.** Freezing is a 20-stack, 15 s debuff. Frostbolt (1),
  each Flurry bolt (1), Glacial Spike (3), each Ray of Frost tick (1), splinters
  (15%) and Hand of Frost (1) add stacks; Frostbolt and Glacial Spike crits add
  one more (Frostbite). Ice Lance's shatter resolves on impact, 0.212 s after
  the cast (the observed travel time), so stacks that land in that window are
  consumed too. It shatters `min(6 x (1 + Thermal Void), stacks)` stacks as one
  hit and consumes them; with Fingers of Frost it shatters 6 (12) and consumes
  nothing. Shatter's crit chance is 1.5x normal (Improved Shatter), which
  needed one generic engine field, `crit_chance_multiplier`.
- **Fingers of Frost** (2 stacks, 15 s): Frostbolt 18.75% (15% increased by
  Frozen Touch's 25%), Glacial Spike 100% (Flash Freeze) + 18.75%, Frozen Orb
  100% on cast (Everlasting Frost) + 3% per bolt.
- **Brain Freeze** (15 s): 36% of Frostbolts and Glacial Spikes in the
  reference (the talent value is 25%, raised by Frozen Touch), Frozen Orb 100%
  (Wintertide). A proc resets one Flurry charge (2 charges, 30 s recharge).
  Flurry with Brain Freeze is +50% damage and grants Thermal Void (8 s), which
  Ice Lance consumes for the double shatter.
- **Icicles / Glacial Spike**: one Icicle every 5 s hasted, 5 stacks unlock
  Glacial Spike; the cast spends them for a 13.0 SP hit and 3 Freezing.
- **Ray of Frost**: 8 ticks of 4.5 SP over 4 s (hasted), 1 Freezing and 1
  splinter per tick, Hand of Frost on every second tick, Splinterstorm for
  10 s, two charges (Hand of Frost's talent) on a 60 s cooldown less 0.1 s per
  shatter stack (Glaciate).
- **Hand of Frost**: a 5 SP hit, 1 Freezing and a 5-stack +1% damage buff;
  summoned by 20% of shatter stacks (20% + 0.5% per stack) and by Ray of
  Frost's second, fourth, sixth and eighth ticks.
- **Frozen Orb**: 30 bolts of 0.777 SP every 0.5 s, 3% Fingers of Frost each,
  four Splintering Orbs splinters, 60 s cooldown less 0.3 s per splinter
  (Spellfrost Teachings).
- **Splinters**: 0.453 SP each, 15% to add a Freezing stack (Infused
  Splinters), one per Frostbolt/Flurry/Ray tick, 3 per Glacial Spike, 2 per
  Brain Freeze (Shifting Shards), 4 per Frozen Orb, and `shatter stacks / 2`
  per Ice Lance (Force of Will). Splinterstorm doubles every conjure. Each
  splinter is an Augury Abounds attempt: 10% bad-luck-protected roll with a
  0.5 s internal cooldown, 8 more splinters.
- **Buffs**: Hand of Frost (5 x 1% damage, 8 s, from every Hand of Frost hit),
  Permafrost Lances (+10% Shatter, 15 s, from Frozen Orb), Overflowing Energy
  (+10% Frostbolt/Glacial Spike crit per stack, spent on a crit), Brainstorm
  (+1% intellect per Brain Freeze, overlapping).
- **Gear procs**: Rune of Unleashed Fire (direct hit plus an 8 s, 1 s-tick
  lingering DoT), Acidbrood Scourge, Void Eruption, and the time-averaged
  secondary-stat procs folded into `[player]`.

## Tier set (Midnight season 2, set 2060)

With gear lines, `[[gear.effects]]` at the end of the spec model both bonuses (`--set gear.set_bonuses=0`
removes them; runs without gear lines never see them).

| bonus | what it does | modelled as |
|---|---|---|
| 2pc (1296585) | Glacial Spike +20%; each stack of Freezing shattered has a 4% chance to make an Icicle | `ability_pct` 20 [db2 e1]; a one-roll helper triggered once per shattered stack from the Ice Lance shatters' `on_start`, 4% [db2 e2] each, adding an Icicle (and the Glacial Spike marker at 5) [simc `trigger_shatter`] |
| 4pc (1296586) | Shatter +5%; casting Glacial Spike can start Rapid Refreezing (5 Icicles over 1 s) | `ability_pct` 5 on the Ice Lance shatter [db2 e1]; the chance is SimC's hardcoded accumulated RNG, 20% on the first try and +2.0116% for each miss since the last proc [simc `init_rng`], then 5 Icicles 0.2 s apart [db2 1310248] |

Measured on `profiles/MID2_Mage_Frost.simc` as it is (trinkets included), against the same profile with the set bonuses off (SimC: `set_bonus=midnight_season_2_2pc=0`, `..._4pc=0`, `bite_of_zuljan_2pc=0`; azsim: `--set gear.set_bonuses=0`). SimC at target error 0.3%, seed 1; azsim 10,000 iterations, seed 1 (95% error about 0.1%). Tier gain = full / off - 1.

| | SimC | azsim | diff |
|---|---:|---:|---:|
| 2pc alone | +5.85% | +6.12% | +0.27 pts |
| 4pc on top of the 2pc | +3.76% | +4.07% | +0.31 pts |
| **tier gain** | **+9.83%** | **+10.44%** | **+0.61 pts** |
| full-profile DPS | 227,208 | 230,204 | +1.32% |
| set bonuses off, DPS | 206,868 | 208,444 | +0.76% |

Counts per fight, off to full (SimC 10,000 iterations, azsim 10,000): Glacial Spike 15.3 to 26.6 (azsim 15.3 to
27.3), Frostbolt 34.9 to 20.8 (37.0 to 21.3), Rapid Refreezing procs 6.5 (6.7). Icicles from the set: SimC 38.7
from the 2pc plus 6.5 x 5 from the 4pc, 71.1 in all; azsim 73.6.

Not modelled: Winter's End (not in the spec, so the 4pc's +5% reaches only the Ice Lance shatter) and the part of
SimC's APL that lets Glacial Spike be queued for 1.5 s after a Rapid Refreezing proc
(`time-buff.rapid_refreezing.last_trigger<1.5`); the spec casts it when the Icicles are in.

## Known gaps

- **Ice Lance 129.4 vs 124.4 casts (+3.8%) and Flurry 38.7 vs 40.3 (-4.1%).**
  The engine is fully GCD-locked; SimC spends a few seconds per fight idle and
  its shatters land on bigger stacks (6.5 per cast on average, against 6.2
  here), so it casts fewer, larger Ice Lances. The two errors partly cancel;
  the total is +1.15%.
- **rune_of_unleashed_fire_lingering** ticks 224 vs 244 per fight. SimC
  resolves the refresh from the next tick (DOT_REFRESH_DURATION); the engine's
  `reset` restarts from now, and its `rolling` mode inflates the per-tick
  damage for this DoT, so `reset` is the closer fit.
- The damage spells' derived multipliers are missing for Frostbolt, Ice Lance
  and Glacial Spike because the observations record the cast spell id (116,
  30455, 199786) rather than the damage spell (228597, 228598, 228600), so the
  talent modifiers (Frost Mage's +19%, Signature Spell's +25% for Frostbolt and
  Glacial Spike) land in the calibration residual instead (Frostbolt x1.91,
  Glacial Spike x1.93).
