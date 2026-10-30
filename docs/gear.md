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


## Item effects and set bonuses

With gear lines, every on-use, on-equip and (for non-trinkets) chance-on-hit
effect of an equipped item in any slot (DB2 `ItemEffect`: trinkets, rings such
as Loa Worshiper's Band, necks, weapons) and every set bonus the character reaches (`ItemSet`,
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

Slots are not special: a ring or weapon effect goes through the same lookup,
template and placeholders as a trinket's. The report row says the trigger
(`use`, `equip`, `proc`), and rating coefficients use the item's own
combat-rating column (jewelry for rings and necks, trinket, weapon, armor). To
model one, write its template in `data/item_effects.toml` like the trinkets'.
The committed `data/gamedata.json.gz` still holds `ItemEffect` rows for
trinkets only: `tools/gamedata.py` now keeps them for every equippable item
(proc effects of non-trinkets included), but the data hasn't been rebuilt
(needs DB2 access), so until it is, rings and weapons carry no effects and are
neither modelled nor listed as `unmodelled`. Enchants that work through a
spell aura, and embellishments (crafting-bonus spells, not `ItemEffect`) are
not read yet.

`--gear-stats` (and the Python `gear_stats`) lists every effect with its
status: `modelled`, `covered` (its values are read by another entry),
`unmodelled` (no template: the item counts by its stats alone) or `off`.
`unmodelled` also lists the sources on their own; the app sends a swap that
changes that list to SimC. `--set gear.trinket_effects=0` and
`--set gear.set_bonuses=0` switch the two off. Adding a trinket: write its
entry, rerun `tools/gamedata.py` (it keeps data for every spell the templates
name) and check it against SimC on the same character.

## Omnium traits

The profile's `omnium_talents=` line (`136822:1/136816:1/...`, or rune names such
as `rune_of_overload`) is parsed into trait ids, ranks dropped, and reported
under `omnium` in `--gear-stats`. A spec may add `[gear.omnium]`:

```toml
[gear.omnium]
baseline = ["136822", "136816", "136817", "136821", "136814"]  # on in the calibration reference
traits = [ { id = "136822", name = "...", global_mult = 1.0 } ]  # one entry per trait
```

With gear lines, `[player].global_mult` is multiplied by the product of the
character's modelled traits divided by the product over `baseline`: the
reference's own set changes nothing, a character with none loses the baseline's
worth, and an extra trait adds its own. A baseline trait without an entry is an
error; a character trait without one is listed as `unmodelled`. Specs without
`[gear.omnium]` and runs without gear lines (calibration, validation) are
untouched bit for bit.

**Not filled in yet.** No spec declares `[gear.omnium]`: each trait's value has
to come from the game data (the trait ids' spells in DB2) and be checked
against SimC with and without the traits (about +3.6% for Ret in total, see
[retribution.md](retribution.md)), and neither was available when this landed.
Traits that are not a flat damage multiplier (procs, rune effects) need a
spec fragment like the item effects' and aren't supported by this mechanism yet.


## Casters

Fire Mage, Destruction Warlock and Devourer Demon Hunter carry `[gear]` tables too. Casters have no
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

Assassination Rogue, Beast Mastery Hunter, Enhancement Shaman (agility) and
Unholy Death Knight (strength) have `[gear]` tables on the same pattern. Against
SimC's MID2 sample profiles without trinkets (`tools/check_gear.py`), every
rating and the final attack power, crit, haste, mastery and versatility match
(Enhancement is 1 rating point short on crit and haste; Beast Mastery's SimC
character has 27 more crit rating than its items and gems sum to, folded into
`crit_bonus`). Mastery coefficients are SimC's spell data (Potent Assassin 1.7,
Master of Beasts 1.9, Enhanced Elements 2.0, Dreadblade 1.8). Unholy's flat
1000 base attack power rides in `weapon_ap_per_dps`. DPS, 5,000 iterations:

| spec | SimC, no trinkets | azsim `--gear` | full profile (trinkets, no consumables) |
|---|---:|---:|---:|
| Assassination | 148,298 | +0.10% | -0.32% |
| Beast Mastery | 181,945 | -1.78% | -2.59% |
| Enhancement | 148,587 | -4.23% | -4.37% |
| Unholy | 148,581 | +0.95% | +1.49% |

The gaps are the specs' own (see their docs), not gear conversion: `--gear`
is within 0.2% of the same specs with hand-entered stats. None has a
`[gear.kit]` yet, so consumables are not modelled.


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
