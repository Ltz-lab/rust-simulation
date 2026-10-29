# Balance Druid (Elune's Chosen)

The single-target MID2 profile is from Nokomanida (US Zul'jin), rank 1 in the
Heroic Sszorak survey, [fight 26](https://www.warcraftlogs.com/reports/thAPjTMWaqcZJCn4#fight=26).
`profiles/MID2_Druid_Balance.simc` contains the combatantinfo gear and talents;
an inactive Guardian hero-selection node was removed from the WCL export.
The committed `bal` references were generated with `tools/make_reference.sh`,
with trinkets, set bonuses, consumables and Bloodlust disabled. Enchant and
embellishment damage remains in the core reference.

For 300s Patchwerk (20% duration variation), 10,000 azsim iterations, seed 1:
SimC **150,502 DPS**, azsim **148,765 DPS** (−1.15%). The principal hits per
fight are Starsurge 85.8 vs 85.2, Starfire 93.3 vs 94.9, Umbral Starfire 19.8
vs 20.2, and Moonfire/Sunfire 290.4/292.3 vs 295.6/297.7. Shooting Stars
are 95.5/95.8 vs 99.6/99.2. Calibration fits damage per hit, not counts.

Held-out probes (20,000 paired azsim iterations, seed 42):

| stat probe | SimC DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +9.59% | +9.31% | −0.27 pts |
| +500 haste | +8.46% | +8.52% | +0.07 pts |
| +500 mastery | +8.90% | +9.26% | +0.36 pts |
| +500 versatility | +8.04% | +8.07% | +0.03 pts |
| +200 intellect | +6.50% | +6.73% | +0.24 pts |

The gear check matches every item level and rating in the core profile; spell
power differs by 1. Its SimC snapshot is in caster form, whereas the engine
uses Moonkin form: remove Lycara's 6% haste and add six mastery points. With
gear and the raid kit disabled, azsim gives 148,771 DPS vs 150,386 SimC in a
separate 5,000-iteration check. The gear kit adds approximate standard
consumables, Bloodlust and potion; see `docs/gear.md` for effect coverage.

## Known gaps

- This is the selected Elune's Chosen single-target build, not a general talent
  or multi-target simulator. Stellar Flare and Force of Nature are not selected
  in this profile, and its single-target APL does not cast Starfall; those
  abilities and Keeper of the Grove are not modeled. The Denizen summon is
  modeled as a pet.
- Eclipse scheduling and opener are simplified; Astral Smolder uses a rolling
  bank, and Shooting Stars uses an approximated heartbeat. DoT, proc and pet
  counts differ slightly as shown above. Harmony duration, denizen cast delay,
  and gear proc rates use approximations documented in the spec.
- Core gear enchant and embellishment effects and time-averaged stat procs
  follow this reference character even when gear changes. Class tier bonuses
  and some trinket effects are not modeled; inspect `--gear-stats` for effect
  coverage. The full raid-kit configuration has not been calibrated against a
  full-kit SimC run.
