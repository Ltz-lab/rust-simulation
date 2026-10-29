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

## Gear and known gaps

`[gear]` computes stats from item lines; `[gear.kit]` adds a raid consumable
approximation, potion and Bloodlust when gear is supplied. The core reference
excludes trinkets and tier. The MID2 profile includes an unmodelled
Freightrunner's Flask on-use effect and Arcane tier effects are not yet
implemented, so full-gear DPS is not validated against SimC. Use the reported
unmodelled effects when interpreting gear comparisons.

The rotation is a single-target approximation: SimC's more elaborate queuing,
reaction timing and APL branches are simplified. Splinter hits are 3% low,
and the Rune of Unleashed Fire lingering ticks are 17% high; Mirror Image
Frostbolt is omitted (9 reference DPS). Gear stat procs are time-averaged.
Evocation is modeled but is not used in the core reference's usual mana
trajectory. The gear kit and other talent builds have not been separately
calibrated or validated. See [spec-format.md](spec-format.md#channels-and-resource-expressions)
for the generic channel and resource features.
