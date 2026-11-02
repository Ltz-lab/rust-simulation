# Arcane Mage (Spellslinger)

`specs/arcane_spellslinger.toml` models the Midnight Season 2 sample profile's
Spellslinger talent tree. Its core-kit reference is SimC, 300-second Patchwerk,
without trinkets, tier, consumables or Bloodlust. Spell data comes from DB2,
talent and rotation behavior from SimC, and the calibration fits damage per hit
only. Charges, mana, clearcasting, channels and rotation determine counts.

## Results

At 10,000 iterations, seed 1, azsim gives **133,980 DPS** (±108 at 95%)
against **133,260 DPS** in the reference: **+0.54%**.

| Source | Reference hits | azsim hits |
|---|---:|---:|
| Arcane Barrage | 57.3 | 58.2 |
| Arcane Missiles | 393.4 | 395.1 |
| Prismatic Bolt | 22.3 | 22.7 |
| Arcane Blast | 57.4 | 57.9 |
| Arcane Splinter | 886.2 | 859.3 |
| Orb Barrage | 45.3 | 45.8 |
| Touch of the Magi explosion | 6.6 | 6.7 |

## Held-out stat probes

Changes are relative to each simulator's base DPS; validation uses 20,000
paired iterations and the same per-hit calibration for all probes.

| Probe | SimC gain | azsim gain | Difference |
|---|---:|---:|---:|
| +500 crit | +8.50% | +8.54% | +0.04 pts |
| +500 haste | +7.90% | +8.21% | +0.31 pts |
| +500 mastery | +7.66% | +8.17% | +0.51 pts |
| +500 versatility | +8.36% | +8.21% | -0.15 pts |
| +200 intellect | +6.56% | +6.63% | +0.08 pts |

## Tier set (Midnight season 2, set 2060)

With gear lines, `[[gear.effects]]` at the end of the spec model both bonuses; `--set gear.set_bonuses=0`
removes them, and runs without gear lines never see them.

| bonus | what it does | modelled as |
|---|---|---|
| 2pc (1296581) | Arcane Missiles +5% damage, one more missile | `ability_pct` +5 [db2 e2]; the channel gets 7+1 ticks [db2 e1] and 15% shorter ticks [db2 e3: a spell-data modifier SimC applies, the tooltip omits it] |
| 4pc (1296582) | each Missiles tick adds Cumulative Power: +3% (8 stacks, 12 s) to the next Arcane Blast or Prismatic Bolt | buff 1296930 stacked from the Missiles dot's `on_tick`, spent in Blast's and Bolt's `on_cast` [simc `arcane_missiles_tick_t::execute`] |

**One rotation condition** is new, because SimC's APL has it: with the 4pc the salvo-driven Prismatic Bolt
(`buff.arcane_salvo.stack>13`) also waits for `buff.cumulative_power.stack>=6` [simc `apl/mage.cpp`]. It is applied
as a `[[patch]] rotation =` entry of the 4pc, so the no-tier rotation is unchanged. Not modelled: Arcane Pulse
(not in the spec) and Echo's Cumulative Power rule for it.

Measured on `profiles/MID2_Mage_Arcane.simc` as it is (trinkets included), against the same profile with the set bonuses off (SimC: `set_bonus=midnight_season_2_2pc=0`, `..._4pc=0`, `bite_of_zuljan_2pc=0`; azsim: `--set gear.set_bonuses=0`). SimC at target error 0.3%, seed 1; azsim 10,000 iterations, seed 1 (95% error about 0.1%). Tier gain = full / off - 1.

| | SimC | azsim | diff |
|---|---:|---:|---:|
| 2pc alone | +8.47% | +8.03% | -0.44 pts |
| 4pc on top of the 2pc | +4.28% | +4.43% | +0.15 pts |
| **tier gain** | **+13.12%** | **+12.82%** | **-0.30 pts** |
| full-profile DPS | 203,139 | 202,279 | -0.42% |
| set bonuses off, DPS | 179,584 | 179,300 | -0.16% |

Counts per fight, off to full (SimC 10,000 iterations, azsim 10,000): Arcane Missiles casts 62.9 to 63.9 (azsim
62.9 to 64.4), Arcane Barrage 61.2 to 65.1 (62.3 to 66.2), Prismatic Bolt 23.9 to 25.4 (24.3 to 25.9), Arcane Blast
61.1 to 55.9 (63.1 to 57.0). Missiles ticks per cast in azsim: 6.7 to 7.6.

## Gear and known gaps

`[gear]` computes stats from item lines; `[gear.kit]` adds a raid consumable
approximation, potion and Bloodlust when gear is supplied. The core reference
excludes trinkets and tier. The MID2 profile includes an unmodelled
Freightrunner's Flask on-use effect. The tier set is modelled with gear lines
(see [Tier set](#tier-set-midnight-season-2-set-2060)), which gives the full-profile DPS against SimC. Use the reported
unmodelled effects when interpreting gear comparisons.

The rotation is a single-target approximation: SimC's more elaborate queuing,
reaction timing and APL branches are simplified. Splinter hits are 3% low,
and the Rune of Unleashed Fire lingering ticks are 17% high; Mirror Image
Frostbolt is omitted (9 reference DPS). Gear stat procs are time-averaged.
Evocation is modeled but is not used in the core reference's usual mana
trajectory. The gear kit and other talent builds have not been separately
calibrated or validated. See [spec-format.md](spec-format.md#channels-and-resource-expressions)
for the generic channel and resource features.
