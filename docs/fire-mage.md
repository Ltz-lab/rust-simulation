# Fire Mage (Sunfury)


`specs/fire_mage.toml`, built the same way: coefficients, cast times, charges,
durations and talent values from DB2 (plus the `Spell` table's tooltip text for
what each talent does), the rest observed in SimC's log. Fire is a very
different shape from Ret: hardcasts, an off-GCD spell, crit chains, a rolling
DoT that banks damage, and a pet, so it was a real test of keeping mechanics in
data. Every mechanic below is generic engine data, not mage code.

## Results

Reference: SimC's Midnight S2 sample Fire profile (Sunfury), core kit only (no
trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000 iterations.
Pet damage (Arcane Phoenix) is included, as `arcane_phoenix_*` sources. The
phoenix is a `[[pets]]` entry (see [spec-format.md](spec-format.md#pets-and-summons)),
summoned by Combustion and dismissed when it ends.

| | DPS |
|---|---:|
| reference | 147,374 |
| azsim | 148,454 (±139) |
| difference | **+0.73%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| pyroblast | 60,712 | 62,610 | +3.1% | 145.5 | 149.9 |
| ignite | 33,949 | 33,951 | +0.0% | 299.5 | 299.7 |
| intensifying_flame | 9,600 | 9,601 | +0.0% | 299.5 | 299.7 |
| fire_blast | 7,802 | 7,716 | -1.1% | 98.0 | 97.0 |
| scorch | 7,603 | 7,296 | -4.0% | 30.9 | 29.7 |
| fireball | 5,953 | 5,488 | -7.8% | 48.9 | 45.2 |
| meteor_impact | 5,854 | 5,859 | +0.1% | 9.6 | 9.7 |
| meteorite_impact | 4,874 | 4,867 | -0.1% | 115.5 | 115.4 |
| meteor_burn | 3,327 | 3,320 | -0.2% | 76.8 | 76.8 |
| arcane_phoenix_pyroblast | 1,450 | 1,447 | -0.2% | 26.9 | 26.9 |
| rune_of_unleashed_fire | 1,378 | 1,376 | -0.1% | 50.2 | 50.3 |
| arcane_phoenix_arcane_barrage | 1,179 | 1,175 | -0.4% | 26.9 | 26.8 |
| arcane_phoenix_greater_pyroblast | 867 | 860 | -0.8% | 5.1 | 5.0 |
| arcane_phoenix_meteorite_bug_impact | 807 | 799 | -1.0% | 15.1 | 15.0 |
| acidbrood_scourge | 776 | 771 | -0.7% | 22.8 | 22.9 |
| arcane_phoenix_arcane_surge | 578 | 577 | -0.1% | 5.1 | 5.1 |
| rune_of_unleashed_fire_lingering | 363 | 426 | +17.5% | 243.5 | 287.1 |
| arcane_phoenix_meteorite_impact | 311 | 308 | -0.9% | 5.0 | 5.0 |
| frostbolt (Mirror Image) | 10 | 0 | -100.0% | 27.7 | 0.0 |

The crit economy matches: 122 Heating Ups and 118 Hot Streaks per fight against
125 and 115 in the reference, Fire Blast 97 vs 98, Combustion 14.9s vs 14.7s
long. Split by phase (Combustion, execute, filler), Pyroblast, Fire Blast,
Fireball and Scorch counts are within a few casts of the reference in each.

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +4.61% | +4.20% | -0.41 pts |
| +500 haste | +8.24% | +10.43% | +2.19 pts |
| +500 mastery | +6.53% | +6.44% | -0.08 pts |
| +500 vers | +7.96% | +7.92% | -0.04 pts |
| +200 intellect | +6.49% | +6.61% | +0.13 pts |

Mastery, versatility and intellect are within 0.15 points, crit within 0.45.
Mastery landing this close matters most: it runs entirely through Ignite, so
it checks the banked-DoT model directly. **Haste over-responds by 2.2 points**.
Both stat priorities still put haste first, then vers; mastery and intellect
swap at fourth and fifth. See [Known gaps](#known-gaps).

## Speed

Same machine (10 cores), 10,000 iterations: azsim **0.24s**, SimC **3.6s**.

## What the raw data alone gets you

With DB2 coefficients and the modelled mechanics but no calibration, the spec
reaches **62% of the reference** (90.8k). Ignite is the standout: a bank of
exactly mastery% of direct damage (×2.25 from Hot Streak Pyroblasts, per the
Hot Streak and Inflame tooltips) lands within 20% of the reference with no
fitting at all. The larger residuals sit on direct hits (Meteor ×2.6, Scorch
×2.5, Fire Blast ×2.2), where the Fire spec aura and talent multipliers live.

Mechanics read from the log rather than the tables: Hot Streak being spent as
Pyroblast is cast, so its own crit can start the next Heating Up; non-crits
breaking both Heating Up and the Fevered Incantation streak; the Combustion
pull delay (Firestarter) and Hyperthermia following the phoenix; the phoenix
casting once a second for Combustion's whole duration; and Fire Blast's
recharge budget. The tooltips list From the Ashes (every direct hit −0.5s),
Pyrocosm (every meteorite −0.5s), Fired Up (−2.5s) and Fiery Rush (+50% in
Combustion), but Fire Blast still came up 10% short inside Combustion until the
phoenix's meteorites were counted too.

## Engine features added for Fire

All generic, data-driven, and Ret's results are bit-for-bit unchanged:

- **spell power** and `sp_coeff` alongside attack power;
- **cast times** (hasted): the spell resolves at the end of the cast, the GCD
  runs from its start;
- **banked (rolling) DoTs**: `feed` a share of each hit (`pct` plus
  `per_mastery`) into a DoT that spreads its bank over the time left and resets
  its duration on each feed, keeping its tick cadence (Ignite, Intensifying Flame);
- **conditional effects** (`if` with the rotation condition language) and
  `on_noncrit` / `on_start` effect lists (crit chains, effects at cast time
  before the hit lands);
- **buffs**: overlapping stacks with their own timers, health-gated buffs
  (`below_health` / `above_health`, `target.health.pct` in conditions), per-ability
  crit, crit damage, mastery and power modifiers, `extend_buff`, `on_expire`;
- cooldown-rate buffs now also speed up a recharge already in progress;
- **`source`**: several ability variants (hardcast / Hot Streak / Hyperthermia
  Pyroblast) report under one result row and share one calibration entry.


## Known gaps

- **Haste over-response (+2.2 pts).** Fire Blast's charge economy reacts too
  strongly: +500 haste raises Fire Blast casts 15% in azsim against 12% in the
  reference, and Hot Streak Pyroblasts (and the Ignite they feed at ×2.25)
  follow. Base counts match, so it's the loop gain that's off: each Fire Blast
  makes a Hot Streak, which feeds spheres → meteorites, Fired Up and more direct
  hits back into Fire Blast's recharge. Candidates still to test against the
  probe: how recharge reductions interact with a hasted recharge, Fire Blast's
  0.5s lockout between casts, and casting Fire Blast mid-cast (tried; it
  starves Combustion with this rotation paraphrase, so it's left out).
- **Crit −0.4 pts.** Burn It All (Combustion damage +20% of crit chance) is a
  fixed 16% here, read from the buff's value in the log.
- **Fitted rather than derived [obs]:** the gear stat procs, folded into final
  stats as their time-averaged ratings; the Fired Up chance inside Combustion
  (35%), Heat Shimmer per-tick chance, the phoenix's cast mix, the phoenix's
  meteorites counting for Pyrocosm, crit multiplier and Fevered Incantation
  per stack, Meteor's 2.25s landing time.
- **Calibration nudges the crit chain.** Per-source crit offsets are averages
  across phases, so after calibration filler Pyroblast crits slightly more
  often: Pyroblast +3% hits and Fireball −8%. Uncalibrated, they're 146.4
  and 47.8 against the reference's 145.5 and 48.9.
- **Simplified:** no travel time; Fire Blast fires between casts rather than
  mid-cast; Rune of Unleashed Fire's lingering DoT overlaps too long (+17%
  ticks, 0.3% of damage); Mirror Image isn't modelled.
