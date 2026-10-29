# Gear → stats


`--gear profile.simc` computes the character's stats from their gear (the
`/simc` addon export's slot lines) instead of taking final stats as input.
Item stats come from the game's item tables: each item's stat allocation, its
item level from bonus IDs (upgrade tracks, crafting quality), the stat budget
for that level and slot, combat-rating and stamina multipliers, plus gems and
enchants. Ratings turn into percent at the character's level with diminishing
returns. On top sit a few per-character constants in the spec's `[gear]`
table (base strength, passives, raid buffs), each observed from the
reference's reported stats.

Against SimC's MID2 sample Ret profile (`tools/check_gear.py`), all 15 items'
item levels and every rating match exactly, with and without trinkets:

| stat | azsim | reference | diff |
|---|---:|---:|---:|
| crit rating | 1,051 | 1,051 | 0 |
| haste rating | 816 | 816 | 0 |
| mastery rating | 1,062 | 1,062 | 0 |
| versatility rating | 107 | 107 | 0 |
| attack power | 2,696 | 2,696 | 0 |
| crit | 31.85% | 31.85% | 0 |
| haste | 20.92% | 20.92% | 0 |
| mastery | 50.07% | 50.07% | 0 |
| versatility | 4.98% | 4.98% | 0 |

DPS from `--gear` is 132,876 vs 132,902 from the hand-entered final stats
(-0.02%, same seed; SimC 131,771). Real Raidbots Top Gear exports
(Havoc DH, BM Hunter, Demonology Warlock) load too: Havoc matches every
rating exactly, Demonology is within 2 (one off-hand, see [Known gaps](#known-gaps)).
Loading the item data costs about 30ms once per run.

### Raid-night kit

An optional `[gear.kit]` table adds what a raid night brings on top of the
gear: flat `stats` (primary stat and ratings from food, flask, augment rune and
weapon oil, added before multipliers and rating conversion), `precombat`
abilities appended to the spec's (Ret: Bloodlust, and a marker that unlocks the
potion), and `primary_buffs` (buff name → flat primary stat, written into that
buff's `power_pct` from the computed attack power). It is on by default and
only ever applied with gear lines; `--set gear.kit.enabled=0` (or the same key
in a batch request's `set`) turns it off. `check_gear.py` does that, since its
SimC side is core kit. Ret's numbers and the gap they close are in
[retribution.md](retribution.md#full-kit-a-real-character-against-simc).


## Known gaps

- The `[gear]` constants are per spec and race (Ret dwarf
  today); racial stat bonuses such as Human's +2% secondaries, enchants that
  work through a spell aura (ring crit-damage, weapon procs), embellishments,
  and set bonuses aren't read from data yet. Two rare item-level bonuses are
  off: type 53 (the Aln'hara Lantern, 331 vs 337) and one Myth bonus (12856,
  340 vs 341). On-equip primary stat from trinkets counts; their effects
  don't. Item data is the full equippable item set of build 12.1.5.70575
  (793 KB gzip'd); rebuild it per patch with `tools/gamedata.py`.
