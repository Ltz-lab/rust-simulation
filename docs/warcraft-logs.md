# Warcraft Logs ingestion

`tools/wcl.py` pulls real fights from the Warcraft Logs v2 GraphQL API and
writes an observations file in the same format `tools/observe.py` makes from
SimC, so the `observe → calibrate → validate` loop can run on real logs.

## Usage

```sh
python3 tools/db2.py   # once: DB2 tables for talent names and loadout strings

# which bosses are single target? top parses per encounter, share of the
# player's damage that hits the main target
python3 tools/wcl.py survey --zone 53 --class Warlock --spec Demonology

# pull the top N parses of one encounter and pool them
python3 tools/wcl.py observe --encounter 3420 -n 25 --skip-aug --out reference/demo_wcl
```

Python 3.11+, stdlib only. The client ID and secret are read at run time with
`op-agent read op://Agents/WCL_CLIENT_ID/credential` (and `WCL_CLIENT_SECRET`)
and kept in memory only.

Every API response is cached under `target/wcl/q/` (gitignored), keyed by the
query and its variables, so a rerun costs no points. Rankings are cached too;
`--refresh` re-reads them. The limit is 3,600 points an hour: one fight costs
about 6 points (report metadata, combatantinfo, damage/cast/buff events paged
by `nextPageTimestamp`), the 25-fight run above with ~90 Aug fights skipped
cost about 400. The tool prints points used, and stops before the limit; rerun
after the reset and cached fights are free.

## Output

`<out>_observations.json`, the `observe.py` schema, pooled over all fights:

- `dps`, `fight_length` (the mean length). Damage is `amount + absorbed` of
  every damage event whose source is the player or one of its pets, over the
  fight's start to end.
- `sources`: per ability, `dps`, `share`, `executes`, `direct_hits`,
  `direct_noncrit_avg`, `direct_crit_rate`, `ticks`, `tick_noncrit_avg`,
  `tick_crit_rate`, `spell_id`, `school`. Counts are rates over the total time
  times the mean fight length; averages and crit rates are pooled over all
  hits. Pets are `<pet>_<spell>`, as in `observe.py`: the pet is the actor's
  name in snake case with the parenthetical dropped (`Wild Imp (Hand of
  Gul'dan)` → `wild_imp`, as SimC names it); the permanent pet, which carries
  the player's own name for it, is `felguard` for Demonology. Extra keys: `pet`,
  `seen_in_fights`.
- `buffs`: buffs on the player, from apply/refresh/remove events: `uptime`,
  `starts`, `refreshes`, `interval`, `duration`, plus `external` (cast by
  someone else) and `seen_in_fights`. Auras up at the pull come from
  combatantinfo.
- `buffed_stats`: the median pre-pull character sheet from combatantinfo:
  primary stats and secondary ratings. These are ratings, not the percentages
  SimC's buffed stats give, so `validate_stats.py` can't use this file as is.
- Extra: `casts` (every cast, damaging or not, per fight and per minute),
  `damage_share_by_actor`, `idle_fraction`, and `wcl` (encounter, fight count,
  hero trees).

`<out>_fights.json`, per fight: report link, the player's DPS (ours and WCL's),
length, pet share, Ebon Might share, idle fraction, external buffs,
character-sheet stats, hero tree, talents (entry, node, rank, spell, name), a
loadout string, and the gear as `/simc` lines (`head=,id=...,bonus_id=...,
enchant_id=...,gem_id=...,ilevel=...`) that `--gear` reads. On top:
`talent_pick_rate` across fights and the most common loadout.

The loadout string is encoded from the talent entries with the inverse of
`tools/talents.py`'s decoder and checked by decoding it back. The tree hash is
zeroed: SimC ignores it, the game may refuse the import.

## How sources map

| WCL | observations |
|---|---|
| damage event, `tick: true` | `ticks`, `tick_*` |
| damage event, otherwise | `direct_hits`, `direct_*` |
| `hitType` 2 | crit; `hitType` 1 feeds the non-crit average |
| `cast` event, same ability name as the damage | `executes` |
| no matching cast (procs, pets) | `executes` = hits (direct, else ticks) |
| `sourceID` = a pet of the player | `<pet>_<spell>` |
| `applybuff` / `removebuff` on the player | buff `starts`, uptime |

Cast and damage spell IDs often differ (Hand of Gul'dan is cast as 105174 and
hits as 86040), hence the match by name. Pet casts aren't pulled.

## Caveats

- **External buffs.** Real raids have Power Infusion, Augmentation Evokers,
  Bloodlust, raid buffs. Most top Demonology parses carry an Aug Evoker, and
  ~96% of their damage lands under Ebon Might. WCL credits that support damage
  to the Evoker, so its DPS for such a fight is ~4-5% lower than the damage the
  warlock's events carry. `--skip-aug` drops fights where more than 5% of the
  damage was done under Ebon Might; with it our DPS equals WCL's. The damage
  WCL credits to the Evoker but logs with the warlock as source (Breath of
  Eons, Fate Mirror) is always dropped. Power Infusion stays in (15 of the 25
  committed fights); it shows as an external buff.
- **Trinkets and tier sets** are in the logs as their own sources and buffs
  (e.g. `rune_of_unleashed_fire`, `void_eruption`) and in every hit through
  stat buffs. The engine models neither yet, so calibrating against these
  numbers folds them into the residuals.
- **Multi-target.** Damage on adds counts. Use `survey` to pick bosses: for
  Sszorak every top Demonology parse hits one target only; most other bosses
  put 30-50% of the damage on adds.
- **Movement and downtime.** `idle_fraction` is time with no cast in progress
  beyond one unhasted GCD after the last action, a rough proxy. Sszorak's
  top parses idle ~2.5% of the fight.
- **Stats** are the character sheet at the pull (flask and food included, no
  combat procs), not SimC's buffed stats.
- **Pooling.** Fights of different lengths are pooled as rates; Tyrant
  windows and cooldown alignment differ per fight length, so per-fight cast
  counts don't map exactly onto a fixed-length sim.
- **Selection bias.** Top parses are the best pulls of the best players:
  good RNG, good externals, kill-time-tuned cooldowns.

## Demonology on Sszorak (committed)

`reference/demo_wcl_observations.json` / `demo_wcl_fights.json`: the top 25
Mythic Sszorak parses without an Aug Evoker (patch 12.1.5). All 25 are
Diabolist. `reference/demo_wcl_top25_observations.json` is the plain top 25,
18 of them with an Aug Evoker, for comparison.

Pets do ~71% of the damage (range 69-73% per fight): wild imps 20%, Demonic
Tyrant 9%, Imp Lord 6%, Gloomhound 6%, Dreadstalkers 6%, Felguard 6%, the
Diabolist demons (Antoran Inquisitor and Jailer, Alythess and Sacrolash,
Mother of Chaos, Overlord, Pit Lord) 17%. The warlock's own spells are 29%.
