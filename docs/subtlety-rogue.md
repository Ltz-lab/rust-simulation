# Subtlety Rogue (Deathstalker)


`specs/subtlety_deathstalker.toml`, built the same way as the others:
coefficients, costs, cooldowns, durations and talent values from DB2, SimC's
rogue module (`sc_rogue.cpp`) and default APL where a mechanic is scripted,
and the rest observed in SimC's log. Subtlety is the first spec on regenerating
energy, combo-point finishers and dual-wield auto attacks, and most of its
damage is copies of other hits (Shadow Clones, Shadow Blades, Goremaw's Bite).
Every mechanic below is generic engine data, not rogue code.

## Results

Reference: SimC's Midnight S2 sample Subtlety profile (Deathstalker), core
kit only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk,
5,000 iterations.

| | DPS |
|---|---:|
| reference | 179,254 |
| azsim | 181,350 (±110) |
| difference | **+1.17%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| eviscerate | 36,689 | 37,600 | +2.5% | 127.2 | 130.2 |
| eviscerate_shadow_clone | 24,838 | 25,346 | +2.0% | 49.5 | 50.4 |
| eviscerate_bonus | 19,657 | 20,086 | +2.2% | 126.2 | 128.8 |
| secret_technique_clones | 14,010 | 14,194 | +1.3% | 24.8 | 25.1 |
| deathstalkers_mark | 11,859 | 12,131 | +2.3% | 113.8 | 116.4 |
| shadow_blades_attack | 9,621 | 9,682 | +0.6% | 585.1 | 588.3 |
| goremaws_bite_finisher | 9,650 | 9,470 | -1.9% | 130.3 | 127.8 |
| shadowstrike | 9,323 | 9,055 | -2.9% | 47.2 | 45.8 |
| auto_attack_mh | 6,334 | 6,337 | +0.1% | 418.1 | 417.9 |
| shadowstrike_shadow_clone | 5,172 | 5,159 | -0.3% | 21.5 | 21.4 |
| hunt_them_down | 4,871 | 4,838 | -0.7% | 683.5 | 678.3 |
| secret_technique_shadow_clone | 4,605 | 4,387 | -4.7% | 7.6 | 7.2 |
| goremaws_bite | 3,807 | 3,899 | +2.4% | 67.4 | 69.0 |
| backstab | 3,238 | 3,341 | +3.2% | 72.6 | 74.8 |
| auto_attack_oh | 3,086 | 3,087 | +0.0% | 417.7 | 417.5 |
| lingering_shadow | 2,567 | 2,574 | +0.3% | 108.5 | 108.7 |
| secret_technique_player | 2,294 | 2,320 | +1.1% | 12.4 | 12.6 |
| breath_of_corruption | 2,145 | 2,139 | -0.3% | 18.0 | 18.0 |
| backstab_shadow_clone | 1,816 | 1,920 | +5.8% | 38.6 | 40.8 |
| instant_poison | 1,483 | 1,509 | +1.7% | 530.9 | 539.6 |
| rune_of_unleashed_fire | 911 | 910 | -0.0% | 41.5 | 41.5 |
| shuriken_storm | 850 | 892 | +4.9% | 9.1 | 9.6 |
| rune_of_unleashed_fire_lingering | 302 | 376 | +24.5% | 220.3 | 274.0 |
| goremaws_bite_shadow_clone | 33 | 0 | -100.0% | 0.3 | 0.0 |

`goremaws_bite` counts its direct hit and its ticks together, as SimC
reports them (`calibrate.py` and `compare.py` now add a source's direct hits
and ticks).

The economy matches closely: 16.3 Shadow Dances, 12.6 Secret Techniques and
6.7 Goremaw's Bites a fight as in the reference; 495 Shadow Techniques procs
(480), 403 combo points from Ancient Arts over 57.6 conversions (399 over
57.1), 103 from Premeditation (103), and 10,900 energy spent (10,855). Per
Shadow Dance, 8.4 actions against 8.7.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +7.59% | +7.19% | -0.40 pts |
| +500 haste | +8.78% | +9.10% | +0.33 pts |
| +500 mastery | +8.25% | +8.39% | +0.14 pts |
| +500 vers | +8.23% | +8.20% | -0.03 pts |
| +200 agility | +6.56% | +6.58% | +0.02 pts |

The stat priority (haste > mastery > vers > crit > agility) is the
reference's exactly. Haste works through energy, both swing timers (twice:
Swift Slasher adds haste to Slice and Dice's attack speed), Shadow
Techniques procs, hasted Secret Technique and Goremaw's Bite, and Shadow
Dance's length (+1.5 × haste rating); before Swift Slasher was modelled the
probe was 1.4 points low, and auto attacks the main miss.

## Gear

`[gear]` turns /simc gear lines into these stats. Against SimC without
trinkets (`tools/check_gear.py`), all 14 items' item levels and every rating
match, and so do attack power (2,640), crit (19.98%), haste (26.25%), mastery
(90.60%) and versatility (11.00%). DPS from `--gear` is 181,056 against
181,091 from the hand-entered stats (-0.02%, same seed; SimC 179,298).
Weapon attack power comes from the main hand's DPS (702 at 117.6).

## Speed

Same machine (10 cores), 10,000 iterations: azsim **1.2s**, SimC **4.7s**.

## What the raw data alone gets you

Uncalibrated, with DB2 coefficients and the modelled mechanics, the spec
reaches about a quarter of the reference: Subtlety's multipliers are deep
(Executioner's 91% mastery on finishers, Shadow Dance, Darkest Night,
Lingering Darkness, Unshakeable Drive, the shadow residual multipliers).
The derived layer finds ×1.3-3.0 per source from talents; the buffs it can
read (Shadow Focus, Find Weakness, Lingering Darkness, Unshakeable Drive)
become `[buffs.*]` patches.

Mechanics read from SimC's source or log rather than the tables:
- **Shadow Techniques**: a stack every 3rd auto-attack hit, or the 2nd with
  50%, every 2nd inside Shadow Dance (Shadowcraft, which also doubles the
  stacks), a 0.45s internal cooldown during which the count carries on, and a
  14-stack cap; every Shadow Clone grants a stack too. Builders take the
  stacks as combo points.
- **Ancient Arts**: 20% per stack taken for a Shadow Clone of the builder (the
  15% tooltip + 5%); 5+ stacks left over arm the next finisher, which takes
  them again right after it lands and is always cloned.
- **Deathstalker's Mark**: 3 stacks from Shadowstrike, one spent per 5+
  point finisher; the last grants Darkest Night (+30 energy, the next finisher
  +35% and a crit), whose finisher re-applies the Mark. Quietus Celeris spends
  one at once 30% of the time. Hunt Them Down hits on every auto attack while
  it's up.
- **Supercharger**: Shadow Dance's two supercharged points add 3 each to a
  finisher's damage and Relentless Strikes.
- **Copies**: Shadow Blades copies 20% of each rogue hit (27.4% of physical
  ones), Goremaw's Bite 20% of finisher hits with target multipliers reversed,
  Lingering Shadow stacks% of Backstab after Shadow Dance.
- **Shadow Dance**: 6s × (1 + 1.5 × haste rating), so 8.0s without a haste
  proc and 8.8s with Venomcursed: one GCD more or less. Shadowstrike waits for
  the stance a moment after the Dance (SimC's stance delay), so a Dance opens
  with Shuriken Storm, Goremaw's Bite or a finisher.
- **Goremaw's Bite** takes Premeditation and Shadow Techniques like a builder
  (a SimC bug flag).

## Engine features added for Subtlety

All generic and data-driven; Ret's, Fire's and Destruction's results are
bit-for-bit unchanged (same seed, with and without gear lines):

- **regenerating resources**: `regen` per second, optionally hasted;
  continuous, settled whenever a resource or buff changes, integrated across
  haste buffs running out; an idle player wakes when regeneration pays for
  something, plus `fight.idle_poll`;
- **dual wield**: `player.off_hand` swings on its own timer, first swing at
  half its swing time, via `auto_attack.off_hand_replace_with`;
- **attack speed buffs**: `attack_speed_pct`, `attack_speed_per_haste`
  (Swift Slasher), and swings in progress resized when attack speed changes;
  `auto_attack.wakes = false` for actors that don't re-check on swings;
- **finishers**: `cost_max` spends up to a cap, `spent.<resource>` in
  expressions is what the last cast paid;
- **echoes**: a buff's `echo` copies a share of each hit as another ability's
  damage, per school, from listed sources, before or after armor;
- `amount_expr` on `gain`, `chance_expr` on `trigger`, `haste_duration` on
  buffs, `cooldown.X.duration` in conditions.

## Known gaps

- **Finishers +2.5%, so DPS +1.2%.** About 25 more combo points a fight: out
  of Shadow Dance the reference hits 2% less often and in it 4% more (its haste
  procs line up with Dances), so a few more Shadow Techniques stacks reach
  Backstab here. Builder and energy totals match.
- **Fitted rather than derived [obs]:** gear stat procs as time averages,
  except Venomcursed haste and Hasty Ritual, which are Poisson buffs; the
  1.0045s action spacing, which decides whether a Dance fits 8 or 9 GCDs;
  Skyfury's 16% repeats and the 16.4% dual-wield misses; Instant Poison's
  chance per source; the weapon's 702 attack power, from the agility probe.
- **Simplified:** Shadow Clones hit at once rather than 0.2-0.5s later (their
  Shadow Techniques stacks do come late); Find Weakness's armor reduction,
  Cold Blood's crit and Momentum of Despair sit in the residual; no Vanish
  auto-attack reset; the Goremaw's Bite clone (0.3 a fight) isn't modelled;
  Rune of Unleashed Fire's lingering DoT overlaps too long, as for Fire.
