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
| The King's Unyielding Wind (Stormbound Emblem of Dazar) | on use: 2s channel, 5s of Haste per tick (20s) | a 2s cast, then a 20s Haste-rating buff |
| Freightrunner's Flask | on use: crit rating for 15s, 90s cooldown | on-use stat buff |
| Loa Worshiper's Band (ring) | real-PPM 2; each proc summons one loa: Capybara (primary stat, always), plus one per unique gem colour worn (Peridot Akil'zon haste, Amethyst Halazzi physical hit, Lapis Jan'alai fire hit, Garnet Nalorakk stacking crit) | one proc per option at rppm / options, `gem`-keyed templates |
| Bandolier embellishment (bonus 13454) | doubles the other embellishments' values | `:band` modifier on the templates' numbers |

Templates may carry a `gem = "garnet" | "peridot" | "amethyst" | "lapis"` key
(used only when that colour is socketed anywhere in the gear; a gem's colour
is the stat it carries first) and the placeholder modifiers `:band`, `:lvl`
(scale at the character's level, for spells SimC doesn't item-scale) and
`:per_option`. Effects an item *bonus* adds (embellishments) come from
`bonus_effects` in the game data.

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

A spec with a `[gear]` table must set `spec_id` (its ChrSpecialization): class set bonuses are keyed by it, and without it
they never match, so a swap that breaks the tier set is not reported as `unmodelled` (Fire, Arcane, Destruction and
Subtlety lacked it until now; a test checks every spec).

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

**Where it stands.** The trait ids are TraitNodeEntry ids; `python3
tools/omnium.py traits` maps them to their definition spells in DB2 (136814
Rune of Overload, 136815 Critical Power, 136816 Void-Tainted Shell, 136817
Lingering, 136818 Masterful Cunning, 136819 Self-Mending, 136820 the Versatile
Warrior, 136821 Burning Haste, 136822 Unleashed Fire, 136823 Lynxlike
Reflexes). The spells carry no damage numbers of their own: every one is an
aura-4 dummy or trigger, and SimC (`unique_gear_midnight.cpp`, namespace
`omnium`) implements only the two core runes (Unleashed Fire, Void-Touched
Orbs) as proc drivers. The others modify the driver: Overload multiplies its
damage by 1 + 100%, Lingering adds a DoT to its hit, Burning Haste, Critical
Power, Masterful Cunning and the Versatile Warrior add a rating buff on each
proc. Self-Mending, Void-Tainted Shell and Lynxlike Reflexes have no SimC
implementation. So a per-trait value cannot be read from a table; it is
measured with `tools/omnium.py ablate` (SimC with each trait removed).

Every spec with a `[gear]` table and an `omnium_talents` profile now carries
a per-trait table (Ret: [retribution.md](retribution.md)). Traits that are
not a flat damage multiplier would need a spec fragment like the item effects';
the flat value is exact for the full set and for none, and approximate in between
(and across gear: the worth is 3.6% at core kit, 3.1% with the full kit). Not
filled: Frost DK, Marksmanship, Survival, Windwalker and Outlaw have no `[gear]`
table yet, and Blood and Protection have no `ablate` run. A trait id has to be
spelled as the profile spells it (`136822` or `rune_of_unleashed_fire`).

**Per-trait values for every spec with `[gear]`.** Filled from `tools/omnium.py
ablate` at 20,000 iterations (SimC core kit, each trait removed in turn); the
`traits` entries are the leave-one-out ratios rescaled so their product is
exactly all/none. Full-character DPS, each profile in `profiles/` as is
("omnium on", the reference's own traits: unchanged by construction) and with
its `omnium_talents` line removed ("none"). azsim 10,000 iterations seed 1,
SimC 10,000 iterations. The absolute gaps are the pieces azsim does not model
yet (trinket effects, tier sets, other item effects); the point is that "none"
moves from azsim-too-high relative to SimC to the same gap as "on".

| spec | on: azsim (before = after) | on: SimC | on: diff | none: azsim before | none: azsim now | none: SimC | diff before | diff now |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Death Knight Unholy | 218,394 | 239,918 | -9.0% | 218,394 | 210,960 | 233,817 | -6.6% | -9.8% |
| Demon Hunter Devourer | 216,649 | 253,967 | -14.7% | 216,649 | 210,659 | 247,763 | -12.6% | -15.0% |
| Demon Hunter Havoc | 225,732 | 260,979 | -13.5% | 225,732 | 217,464 | 253,628 | -11.0% | -14.3% |
| Druid Balance | 189,629 | 232,400 | -18.4% | 189,629 | 183,950 | 226,295 | -16.2% | -18.7% |
| Druid Feral | 235,739 | 251,565 | -6.3% | 235,739 | 228,094 | 244,133 | -3.4% | -6.6% |
| Hunter Beast Mastery | 246,094 | 262,831 | -6.4% | 246,094 | 238,932 | 255,771 | -3.8% | -6.6% |
| Mage Arcane | 179,300 | 203,552 | -11.9% | 179,300 | 173,777 | 197,973 | -9.4% | -12.2% |
| Mage Fire | 205,290 | 234,194 | -12.3% | 205,290 | 198,751 | 227,787 | -9.9% | -12.7% |
| Mage Frost | 208,444 | 227,580 | -8.4% | 208,444 | 200,770 | 221,507 | -5.9% | -9.4% |
| Paladin Retribution | 231,338 | 225,732 | +2.5% | 223,295 | 223,338 | 218,803 | +2.1% | +2.1% |
| Priest Shadow | 182,524 | 218,873 | -16.6% | 182,524 | 175,919 | 211,927 | -13.9% | -17.0% |
| Rogue Assassination | 212,557 | 237,490 | -10.5% | 212,557 | 205,708 | 230,943 | -8.0% | -10.9% |
| Rogue Subtlety | 254,245 | 282,888 | -10.1% | 254,245 | 246,935 | 275,824 | -7.8% | -10.5% |
| Shaman Elemental | 208,556 | 250,415 | -16.7% | 208,556 | 201,943 | 243,205 | -14.2% | -17.0% |
| Shaman Enhancement | 204,977 | 247,988 | -17.3% | 204,977 | 197,511 | 239,645 | -14.5% | -17.6% |
| Warlock Affliction | 227,754 | 236,925 | -3.9% | 227,754 | 219,853 | 229,764 | -0.9% | -4.3% |
| Warlock Demonology | 213,769 | 236,119 | -9.5% | 213,769 | 207,144 | 229,861 | -7.0% | -9.9% |
| Warlock Destruction | 203,963 | 231,589 | -11.9% | 203,963 | 197,480 | 225,468 | -9.5% | -12.4% |
| Warrior Arms | 226,242 | 240,922 | -6.1% | 226,242 | 217,882 | 233,448 | -3.1% | -6.7% |
| Warrior Fury | 227,964 | 257,228 | -11.4% | 227,964 | 220,651 | 250,176 | -8.9% | -11.8% |


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


## Raid-night kit against SimC (full gear, no trinkets)

`[gear.kit]` is each spec's food, rune, flask, weapon oil, potion and
Bloodlust. The flask is not the same for every spec (SimC picks it per spec in
`engine/class_modules/apl/*.cpp`) and some specs have a rating multiplier
(`[gear] rating_mult`, aura 405: Frost Mage's Winter's Blessing is +5% haste
rating), so a kit copied from another spec puts the wrong 165 rating on the
character. Measured on each sample profile (trinkets and tier set removed,
omnium traits kept) with SimC's default consumables, the kits are now:

| spec | flask (165) | oil | other |
|---|---|---|---|
| Fire | versatility | 16 crit/haste | |
| Frost Mage | crit | 16 crit/haste | `rating_mult` haste 1.05, mastery coefficient 1.6 (was 1.0), primary x1.1025 (was 1.071) |
| Arcane | haste | 16 crit/haste | |
| Affliction, Demonology | crit | 16 crit/haste | potion is Liquid Luster (versatility stacks), not Light's Potential |
| Destruction | mastery | 16 crit/haste | Potion of Recklessness (+1725 mastery, -232 versatility) |
| Elemental | mastery | none (staff) | |
| Fury | mastery | 32 crit/haste (two weapons) | |
| Devourer | mastery | 32 crit/haste | Blooming Feast +65 mastery, no food primary |
| Subtlety | crit | 32 crit/haste | |

Fire, Destruction and Subtlety had no kit at all, so a real character read
13-14% low (Fire, Destruction) and 11% low (Subtlety) before anything else.
Shadow and Frost Mage had the potion buff but no rotation entry that used it.
After the change every spec's crit, haste, mastery and versatility percent
and its spell or attack power match SimC's buffed stats on its sample
profile. DPS on the same profiles (no trinkets, set bonuses off, 300s
Patchwerk, SimC 60,000 iterations at 0.15% error, azsim 10,000):

| spec | SimC | azsim before | azsim after |
|---|---:|---:|---:|
| affliction_hellcaller | 183,142 | 185,168 (+1.1%) | 183,503 (+0.2%) |
| arcane_spellslinger | 151,973 | 151,754 (-0.1%) | 151,670 (-0.2%) |
| arms_slayer | 186,694 | 186,297 (-0.2%) | 186,297 (-0.2%) |
| balance_elunes_chosen | 174,911 | 171,444 (-2.0%) | 171,444 (-2.0%) |
| demonology_diabolist | 183,934 | 180,221 (-2.0%) | 178,893 (-2.7%) |
| destruction_hellcaller | 178,279 | 155,815 (-12.6%) | 178,414 (+0.1%) |
| devourer_voidscarred | 192,825 | 189,258 (-1.8%) | 189,885 (-1.5%) |
| elemental_farseer | 185,688 | 189,222 (+1.9%) | 188,124 (+1.3%) |
| feral_wildstalker | 193,526 | 189,519 (-2.1%) | 189,519 (-2.1%) |
| fire_mage | 172,909 | 148,456 (-14.1%) | 178,818 (+3.4%) |
| frost_mage_spellslinger | 172,161 | 137,961 (-19.9%) | 172,978 (+0.5%) |
| fury_slayer | 190,804 | 185,656 (-2.7%) | 186,934 (-2.0%) |
| havoc_fel_scarred | 185,105 | 177,807 (-3.9%) | 177,807 (-3.9%) |
| retribution_herald | 152,451 | 153,200 (+0.5%) | 153,200 (+0.5%) |
| shadow_voidweaver | 165,558 | 157,638 (-4.8%) | 163,897 (-1.0%) |
| subtlety_deathstalker | 202,849 | 181,258 (-10.6%) | 210,013 (+3.5%) |

What is left is the spec-level residue (Havoc -3.9%, Demonology -2.7%, Balance
and Feral about -2%) and Fire and Subtlety reading 3.5% high. That is not trinket
procs in `[gear.procs]`: `tools/proc_audit.py` (SimC's per-second stat
timelines, with and without the profile's trinkets) shows those values equal
the trinket-free character's own proc averages (Fire: +28 crit, +124 haste,
+223 mastery, +154 versatility rating against the spec's 0.45%, 2.78%, 2.14%,
2.91%; the trinkets add another +233 haste on top, which is not in the spec).
The excess comes from the raid kit's response: with consumables, potion and
Bloodlust split out (5,000 iterations), Fire's Bloodlust is worth +8.7% in
azsim against +6.2% in SimC, and Subtlety's consumables plus Bloodlust +20.3k
against +14.6k, while the no-kit baselines are within +0.8% and +1.1%.
Trinket and set effects are modelled separately.
A ring swap that only moves ratings is mapped exactly (`--gear-stats` deltas
equal SimC's to 0.01 points), so the remaining swap error is the engine's
response to the stats (Fire's haste over-response, below) and item effects the
ring carries (issue #19).

## Raid-night kit: Assassination, Beast Mastery, Enhancement, Unholy

These four had a `[gear]` table but no `[gear.kit]`, so gear runs read 13-19%
low against SimC. Each kit now carries the sample profile's consumables
(measured as the difference in SimC's buffed stats with and without them),
Bloodlust at the pull and the potion, used as in SimC's APL:

| spec | flask (165) | oil | potion | used |
|---|---|---|---|---|
| Assassination | crit | 16 crit/haste x2 daggers (32) | Light's Potential | Rupture up and Bloodlust / Deathmark / last 30s |
| Beast Mastery | haste | 16 crit/haste (bow) | Recklessness (+1725 haste, -232 versatility) | Bestial Wrath ready |
| Enhancement | haste | none (disabled in the profile) | Recklessness (+1725 mastery, -232 versatility) | Ascendance or Doom Winds up |
| Unholy | mastery | 16 crit/haste | Recklessness (+1725 mastery, -232 versatility) | Army ghouls / Forbidden Knowledge / Dark Transformation |

Recklessness raises the highest rating on the gear and lowers the lowest, so
the buff names them per spec. Every spec's buffed attack power, crit, haste,
mastery and versatility match SimC's to rounding. DPS on the sample profiles
(trinkets and tier set removed, SimC 20,000 iterations, azsim 10,000, seed 1):

| spec | SimC | azsim before | azsim after |
|---|---:|---:|---:|
| assassination_deathstalker | 173,631 | 148,534 (-14.5%) | 173,574 (-0.03%) |
| beast_mastery_pack_leader | 207,264 | 178,626 (-13.8%) | 206,903 (-0.17%) |
| enhancement_stormbringer | 174,833 | 142,230 (-18.7%) | 165,253 (-5.5%) |
| unholy_rider | 172,340 | 149,951 (-13.0%) | 173,509 (+0.68%) |

Enhancement's remaining gap is its known -4% rotation residue (hit counts, see
[enhancement-shaman.md](enhancement-shaman.md#known-gaps)) plus about 1.5
points from Bloodlust and the potion landing on a rotation that is already
GCD-bound. Core-kit runs (`--calib`, no gear) are bit-identical before and
after for all four.

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
  effect library has an entry (eleven trinket effects and the Midnight 12.0/12.1 class sets of Ret, Shadow and the three warlock specs so far,
  see above and [Fallbacks closed](#fallbacks-closed)); others are reported as `unmodelled`. Library procs are rolled
  on direct hits as real PPM, but which hits may trigger them isn't read from
  the proc flags. On-use items share no category cooldown, and Font of
  Venomous Rage is used on cooldown where SimC's APL waits for the other
  trinket. Item data is the full equippable item set of build 12.1.5.70575
  (871 KB gzip'd, with the effect and set tables); rebuild it per patch with `tools/gamedata.py`.


## Fallbacks closed

Effects the app used to send to SimC because azsim reported them `unmodelled`.
Each was validated by running the same profile on SimC and azsim with and
without it (10,000 SimC / 5,000 azsim iterations, Patchwerk 300s, one target,
the profile's own gear, kit on, other trinkets and the tier set removed;
`tools/validate_effects.py` for trinkets, set bonuses by SimC's
`set_bonus=midnight_season_N_{2,4}pc=0`). Contribution is the DPS the effect adds
over the same character without it, so the trinket rows include the item's own stats.

| effect | where | SimC | azsim |
|---|---|---:|---:|
| Vile Vial of Volatile Venom (1293316) | Demonology | +9.43% | +9.28% |
| | Affliction | +9.17% | +9.60% |
| | Shadow | +9.44% | +8.94% |
| | Destruction | +8.37% | +8.62% |
| | Retribution | +9.18% | +8.98% |
| | Fire (see below) | +8.99% | +6.80% |
| Vexhul's Everflowing Gland (1295833) | Demonology | +8.65% | +8.58% |
| | Affliction | +9.19% | +9.01% |
| | Shadow | +8.01% | +8.50% |
| | Destruction | +8.87% | +9.12% |
| | Retribution | +3.57% | +4.08% |
| Gebbo's Bottomless Bag (1292291) | Demonology | +8.99% | +8.70% |
| | Affliction | +9.36% | +8.60% |
| | Shadow | +8.93% | +8.78% |
| | Destruction | +8.86% | +8.94% |
| | Retribution | +8.29% | +8.10% |
| Shadow 2pc (set 2063, 1296579) | Shadow | +5.44% | +5.45% |
| Shadow 4pc (1296580) | Shadow | +6.4% (12.18% with the 2pc) | +5.5% (11.25%) |
| Affliction 2pc (set 2066, 1296568) | Affliction | +3.11% | +3.03% |
| Affliction 4pc (1296569) | Affliction | +4.3% (7.54%) | +4.4% (7.62%) |
| Demonology 2pc (1296573) | Demonology | +2.44% | +2.40% |
| Demonology 4pc (1296574) | Demonology | +4.7% (7.27%) | +4.8% (7.36%) |
| Destruction 2pc (1296571) | Destruction | +4.20% | +4.21% |
| Destruction 4pc (1296572) | Destruction | +5.7% (10.11%) | +5.7% (10.18%) |
| Retribution MID1 2pc (set 1985, 1264848) | Retribution | +0.51% | +0.49% |

The 4pc rows give the 4pc's own gain over the 2pc, and the total in brackets.

How they are modelled (`data/item_effects.toml`, class sets in the specs'
`[[gear.effects]]`):

- **Vile Vial**: on use (120s), one of the four secondary ratings at random for
  15s, then a smaller penalty to an independently rolled one for 15s (`choose`).
- **Vexhul's**: on use (180s), five nature hits 0.2s apart, each scaled by the
  target's remaining health. The kill cooldown reduction is not modelled (one target).
- **Gebbo's**: real PPM 3, each proc one of six artifacts at random: Salmon, Gralstone
  and Voidfin as flat buffs, the Scroll (12 mastery stacks that decay), the Seashell (crit
  stacks that build) and the Totem (versatility stacks spent by hits, 0.75s apart).
- **Shadow 4pc** gives Tentacle Slam a free Void Volley stack (Crushing Void raised to
  its 4 stacks), **Affliction 4pc** +2% damage per active Unstable Affliction (checked every
  0.5s), **Demonology 4pc** a 20% chance for an expiring Wild Imp to Implode at 350%, and
  **Destruction 4pc** a 6% damage-taken debuff on an Echo of Sargeras hit.

New generic pieces: `:divN` and `eN/eM` placeholder forms, `[[patch]]` on `pet =`
and `set = { field = value }`. Destruction's `[gear]` had no `spec_id`, so its class
set bonuses were never looked up; it has one now.

Known gaps: the Fire Mage spec has no `use_items` condition, so the Vial is used
on cooldown rather than inside Combustion as SimC's APL does (the +6.8%);
the Shadow 4pc is about 0.9 points light (the rotation spends the volleys slightly
differently); Retribution's MID1 4pc (set 1985) stays `unmodelled`: Final Verdict
re-applies Expurgation with its own base multiplier, which makes the DoT's ticks 3.8x
stronger in SimC (+6.9%). Wavecaller's Seastone (1295058, Shadow's other trinket)
is still `unmodelled`.

## Checking a trinket against SimC

`tools/validate_effects.py --simc <simc> --case profile.txt:<azsim spec>:trinket2`
runs a profile with no trinkets and with only the one under test, on SimC and on
azsim (effect modelled, and `gear.trinket_effects=0` for stats only), and prints
the DPS the item adds on each. Results (20,000 iterations, same raid buffs):

| item, spec | SimC gain | azsim gain, modelled | azsim gain, stats only |
|---|---:|---:|---:|
| King's Unyielding Wind, Fire Mage | +9.66% | +9.32% | +6.29% |
| King's Unyielding Wind, Demonology | +10.31% | +10.26% | +5.42% |
| Freightrunner's Flask, Shadow | +9.33% | +8.55% | +5.71% |

The trinket-free baselines differ from SimC by -23% (Fire), -6% (Demonology)
and -12% (Shadow). That gap is not the effect's, and it is not trinket procs
baked into `[player]` either (`tools/proc_audit.py` finds none): this
comparison leaves SimC's tier set on, which azsim does not model, and the
kit-less specs lack consumables. Switch the set bonuses off on the SimC side
(`set_bonus=midnight_season_2_2pc=0`, `..._4pc=0`) for baselines that mean
something.
