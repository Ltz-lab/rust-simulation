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


## Trinkets and set bonuses

With gear lines, every on-use and on-equip effect of an equipped trinket
(DB2 `ItemEffect`) and every set bonus the character reaches (`ItemSet`,
`ItemSetSpell`, for the spec's `spec_id` or any spec) is looked up by its
spell id in an effect library: `data/item_effects.toml` for trinkets and
non-class sets, `[[gear.effects]]` in the spec for class set bonuses. Each
entry is a fragment in the spec format (abilities, buffs, procs, rotation
entries, and `patch`es that append effects to the spec's own abilities). Its
numbers come from the game data through `${...}` placeholders, scaled to the
item's level the way SimC's `spelleffect_data_t::average(item)` does: an
effect's coefficient times the epic budget (`-1`), that times the trinket
combat-rating multiplier (`-7`), or the damage budgets (`-8`, `-9`), else its
base points; plus the spell's real PPM (with its haste flag), duration,
internal cooldown, cooldown and stack cap. So a trinket at a different item
level, or a different trinket with the same shape, needs no code.

Most Midnight trinkets keep their numbers in dummy auras that SimC scripts
(`unique_gear_midnight.cpp`); the library follows those scripts. It covers
the app's Ret character and its bag:

| item | effect | modelled as |
|---|---|---|
| Zul'jin's Guillotine Technique | real-PPM 3 (hasted) physical hit, +2% per 1% target health missing | on-hit proc, `damage.scale` on `target.health.pct` |
| Bite of Zul'jan 2pc | Guillotine also applies Venomfang | patch on the Guillotine (where the spec has Venomfang) |
| Voracious Heart of Ula'tek | on use: primary stat 20s; meanwhile hits proc Devour Morsel (real-PPM 18) stacking more primary | on-use ability in the rotation, an on-hit proc gated by the buff |
| Gaze of the Alnseer | real-PPM 2 Alnsight; meanwhile hits (0.75s ICD) stack primary stat | two on-hit procs, stacks with their own timers |
| Twisted Horror's Tendril | real-PPM 20 stacks of crit rating, 3 at the pull | on-hit proc, stacks with their own timers |
| Font of Venomous Rage | on use: 2s channel of nature damage | a 2s cast dealing the total |

Buff stats (`stats = { primary = .., crit_rating = .. }` per stack) turn into
the character's own percent bonuses at the margin: primary stat as a share of
attack or spell power, ratings through the diminishing-returns curve from the
character's current rating. On-use items go into the rotation after the
spec's `use_after` action, under its `use_items` condition for stat buffs
(Ret: SimC's "in a fresh Avenging Wrath"). Proc damage from the library takes
the spec's `effect_mult`: Mystic Touch, Chaos Brand and Hunter's Mark, which
calibrated sources carry in their fitted multipliers but a new source would
miss. Against SimC, per hit: Guillotine 79,557 vs 79,456, Devour Morsel
12,708 vs 12,734; per fight: 24.6 vs 23.9 Guillotines, 35.8 vs 35.3 Devour
Morsels.

`--gear-stats` (and the Python `gear_stats`) lists every effect with its
status: `modelled`, `covered` (its values are read by another entry),
`unmodelled` (no template: the item counts by its stats alone) or `off`.
`unmodelled` also lists the sources on their own; the app sends a swap that
changes that list to SimC. `--set gear.trinket_effects=0` and
`--set gear.set_bonuses=0` switch the two off. Adding a trinket: write its
entry, rerun `tools/gamedata.py` (it keeps data for every spell the templates
name) and check it against SimC on the same character.

## Casters

Fire Mage and Destruction Warlock carry `[gear]` tables too. Casters have no
weapon attack power; `spell_power_per_primary` turns the buffed intellect
into spell power (1 per point, observed) and `attack_power_mult` is left out.
Their `[player]` stats are the buffed snapshot plus time-averaged gear stat
procs, so `[gear.procs]` adds those same averages back on top (in `[player]`
units). `--gear-stats` reports both: `buffed` is what SimC reports, `player`
is what the sim runs with. Against SimC's MID2 sample profiles without
trinkets (`tools/check_gear.py`, which picks the spec from the profile's
`spec=` line), every item level and rating matches:

| stat | Fire azsim | Fire SimC | Destro azsim | Destro SimC |
|---|---:|---:|---:|---:|
| crit rating | 170 | 170 | 1,149 | 1,149 |
| haste rating | 1,204 | 1,204 | 771 | 771 |
| mastery rating | 1,069 | 1,069 | 1,002 | 1,002 |
| versatility rating | 595 | 595 | 123 | 123 |
| spell power | 3,341 | 3,341 | 3,375 | 3,375 |
| crit | 10.70% | 10.70% | 35.98% | 35.98% |
| haste | 29.91% | 29.91% | 19.87% | 19.87% |
| mastery | 17.50% | 17.50% | 73.57% | 73.57% |
| versatility | 14.02% | 14.02% | 5.28% | 5.28% |

DPS from `--gear` vs the hand-entered stats (5,000 iterations, same seed):
Fire 148,405 vs 148,397 (+0.01%; SimC 147,351), Destruction 155,828 vs
155,835 (-0.00%; SimC 157,039).


## Agility

Subtlety Rogue's `[gear]` works like Ret's: `attack_power_mult` (Battle
Shout) on the buffed agility, and `weapon_ap_per_dps` for the main hand's
share of ability damage (702 at 117.6 DPS, fitted to the agility probe). All
14 items, every rating and the buffed stats match SimC without trinkets; see
[subtlety-rogue.md](subtlety-rogue.md#gear).


## Known gaps

- The `[gear]` constants are per spec and race (all dwarf
  today). The casters' `[gear.procs]` averages were observed on the sample
  gear and stay fixed when the gear changes; racial stat bonuses such as Human's +2% secondaries, enchants that
  work through a spell aura (ring crit-damage, weapon procs) and
  embellishments aren't read from data yet. Two rare item-level bonuses are
  off: type 53 (the Aln'hara Lantern, 331 vs 337) and one Myth bonus (12856,
  340 vs 341). Trinket effects and set bonuses count only where the
  effect library has an entry (six trinket effects and two sets so far,
  see above); others are reported as `unmodelled`. Library procs are rolled
  on direct hits as real PPM, but which hits may trigger them isn't read from
  the proc flags. On-use items share no category cooldown, and Font of
  Venomous Rage is used on cooldown where SimC's APL waits for the other
  trinket. Item data is the full equippable item set of build 12.1.5.70575
  (871 KB gzip'd, with the effect and set tables); rebuild it per patch with `tools/gamedata.py`.
