# Havoc live overlay (combat-log driven)

`overlay/havoc_fel_scarred.toml` is a Havoc Demon Hunter (Fel-Scarred) priority list for a
Hekili-style "press this next" overlay on retail 12.1.5, fed only by the combat-log stream that
wow-forever rebuilds from the world connection (`protocol/combatlog`, see its `docs/RETAIL.md`).
`src/overlay.rs` turns those events into state and evaluates the list; nothing in it is
Havoc-specific.

```bash
azsim overlay overlay/havoc_fel_scarred.toml --demo 45 --haste 0.17            # stand-in fight, 1 target
azsim overlay overlay/havoc_fel_scarred.toml --demo 45 --haste 0.17 --targets 4
azsim overlay overlay/havoc_fel_scarred.toml --replay events.jsonl --next 3 [--player <guid>]
```

## In the live pipeline

```bash
combatfeed -run <capture-live run> -spelldata <wago CSV dir> -follow \
  | azsim overlay overlay/havoc_fel_scarred.toml --feed --next 3 \
  | auraoverlay
```

`--feed` reads wow-forever's `tools/combatfeed` lines on stdin, writes every line back unchanged
(flushed per line), and adds one line whenever the suggestion changes:

```json
{"type":"next","t":36032,"spell":198013,"name":"eye_beam","in_ms":0,"off_gcd":false,
 "queue":[{"spell":370965,"name":"the_hunt"},{"spell":258860,"name":"essence_break"}],"fury":42,"enemies":1}
```

`spell` is the id to show (the in-Metamorphosis form when that is what to press: Death Sweep
210152, not Blade Dance), `in_ms` how long until it can go out (the GCD or a cooldown), `queue` the
look-ahead. How each combatfeed line is used:

| line | used for | replaces |
|---|---|---|
| `event` | everything in the table below, with `mine` / `me` as the source/dest test | — |
| `auras` `unit: player` | the player's auras with `remaining_ms` and `stacks` (a full snapshot: an aura missing from it is gone) | the event-based aura timers |
| `auras` `unit: target` | the player's (`mine`) debuffs on the selected target | same |
| `cooldowns` | everything recovering (`spell`, or `category` + `spells`, with `charges`); absent = ready; `gcd_ms` | the cast-started timers, including Cycle of Hatred's Eye Beam cuts |
| `power` type 17 | Fury | energizes minus costs |

Between snapshots the estimates from events carry on, so the rules' inputs stay the same whether a
snapshot exists or not. Run combatfeed with `-spelldata` so its cooldown list knows Havoc's
cooldowns: once a `cooldowns` line arrives, a spell missing from it counts as ready.

Tested on a real retail capture (a Vengeance DH on a dummy, 6,398 lines): every line passed through
byte for byte, Fury followed the `power` lines, and the picks followed the casts. Vengeance shares
Immolation Aura, Throw Glaive and Felblade with Havoc, so that run checks the plumbing, not Havoc's
rules; `tests/overlay_havoc.rs` checks the rules on combatfeed-shaped lines.

## Replaying raw events

`--replay` reads one event a line, as Go's `json.Marshal(combatlog.Event)` writes it
(`TimeMs, Sub, Source, Dest, SpellID, Amount, School`; snake_case also works), and prints
`{"t", "fury", "enemies", "next": [3 presses]}` whenever the suggestion changes. Without
`--player` the player is whoever casts one of the list's spells or energizes themselves with Fury.
Tests: `cargo test --release --test overlay_havoc`.

## Format

Same rotation language as the specs (`[[rotation]]` `action` + `if`, `src/expr.rs`). The other
tables say where each name comes from:

| table | what | from the log |
|---|---|---|
| `[[spells]]` | button: cast `ids`, `cooldown` (`hasted`, `charges`, shared `group`), Fury `cost`, `gcd`, form (`requires`/`excludes` an aura), `on_cast` resets | `SPELL_CAST_SUCCESS` by the player |
| `[[auras]]` | buff (`on = "player"`) or debuff the player put on its target (`on = "target"`) | `SPELL_AURA_APPLIED/REFRESH` (Amount = remaining ms), `_APPLIED_DOSE/_REMOVED_DOSE` (Amount = stacks), `_REMOVED` |
| `[[markers]]` | state with no aura: set and cleared by the player's casts, optionally only while an aura is up (`ends_with`) | `SPELL_CAST_SUCCESS` |
| `resource` | Fury, power type 17 | `SPELL_ENERGIZE` / `SPELL_PERIODIC_ENERGIZE` to the player, minus each cast's `cost`; a `POWER_UPDATE` event (not CLEU: an exact SMSG_POWER_UPDATE reading, if the stream adds one) overwrites it |
| `active_enemies` | units the player damaged in the last `aoe_window` (3s) | `SWING_*`, `SPELL_DAMAGE`, `SPELL_PERIODIC_DAMAGE`, `SPELL_MISSED`; `UNIT_DIED` drops a unit |
| target | the unit the player swings at (else last cast target) | `SWING_DAMAGE` dest, `SPELL_CAST_SUCCESS` dest |
| `gcd`, haste | 1.5s / (1 + haste), 0.75s floor; haste = `--haste` (gear, set once) x Metamorphosis 20% x Furious Gaze 8% | the two haste auras |

A rule fires when its spell is off cooldown at the end of the current GCD, affordable, in the right
form, and its `if` holds. `recommend(n)` looks ahead by applying each pick to a copy of the state
(cost, cooldown, markers, plus the spell's `gain` and `applies`, which live play reports as events).

## Spell and aura ids (retail 12.1.5.70783)

| name | cast id(s) | aura id | cooldown in the list |
|---|---|---|---|
| Chaos Strike / Annihilation | 162794 / 201427 | — | — (40 Fury) |
| Blade Dance / Death Sweep | 188499 / 210152 | — | shared: 15s / 9s, hasted (35 Fury) |
| Eye Beam / Abyssal Gaze | 198013 / 452497 | — | shared 20s (30 Fury) |
| Immolation Aura / Consuming Fire | 258920 / 452487 | 258920 | shared 30s |
| Essence Break | 258860 | 320338 (on target) | 40s |
| Metamorphosis | 191427 | 162264 | 120s; resets Eye Beam, Blade Dance, Immolation Aura |
| The Hunt | 370965 | — | 60s |
| Felblade | 232893 | — | 12s |
| Vengeful Retreat | 198793 | Initiative 391215 | 25s, off the GCD |
| Throw Glaive | 185123 | — | 9s (25 Fury) |
| Furious Gaze | — | 343312 | |
| Demonsurge (stacking) | — | 452416 | |
| Eternal Hunt | — | 1271092 | Blade Dance/Death Sweep consume it and reset |
| Empowered Eye Beam | — | 1271144 | |
| Demon Blades (Fury) | energize 203796 | — | |

Ids checked by name against `SpellName` at build 12.1.5.70783; cooldowns, costs and durations from
the Spell tables (70575) via `tools/spelldb.py`. The in-Metamorphosis replacements come from
Metamorphosis 162264 effects #5/#6 and Demonsurge 452489 effects #1/#3.

## The priority, and what each rule reads

Sources: SimC's Havoc APL (`profiles/MID2_Demon_Hunter_Havoc.simc`, line numbers below), Method's
Havoc guide for 12.1.5, and `specs/havoc_fel_scarred.toml`. Talent checks in the APL are resolved
for that profile (Fel-Scarred, Chaotic Transformation, Eternal Hunt).

| # | press | when | state it needs | source |
|---|---|---|---|---|
| 1 | Metamorphosis | Blade Dance/Death Sweep on cooldown, Eye Beam > 8s away, no Annihilation/Death Sweep empowerment left | cooldowns (casts), markers (casts) | SimC 42 |
| 2 | The Hunt | Eye Beam < 10s and Meta > 15s away, or Eye Beam down while Meta is ready | cooldowns | SimC 44 |
| 3 | Vengeful Retreat (off GCD) | Eye Beam about to come up, Meta not ready, no Initiative; or before a hardcast Meta | cooldowns, aura 391215, aura 1271092 | SimC 45 |
| 4 | Essence Break | a Demonsurge Death Sweep or Annihilation is waiting | markers | SimC 50 |
| 5 | Death Sweep | Essence Break on the target, or empowered | debuff 320338 on target, marker | SimC 51 |
| 6 | Annihilation | Essence Break on the target, or empowered | debuff 320338, marker | SimC 52 |
| 7 | Abyssal Gaze | in Meta, once both empowered spenders are gone | aura 162264, markers | SimC 55 |
| 8 | Eye Beam | out of Meta | aura 162264, cooldown | SimC 56 |
| 9 | Consuming Fire | empowered, and the Demonsurge stacks run out within a GCD | marker, aura 452416 remaining | SimC 57 |
| 10 | Essence Break | Eye Beam > 4s away | cooldowns | SimC 59 |
| 11 | Death Sweep | | aura 162264, cooldown, Fury | SimC 60 |
| 12 | Blade Dance | | cooldown, Fury | SimC 61 |
| 13-14 | Consuming Fire / Immolation Aura | **2+ targets** | active_enemies, cooldown | SimC 62 |
| 15 | Annihilation | | aura 162264, Fury | SimC 63 |
| 16 | Felblade | Fury <= 100 | Fury, cooldown | SimC 64 |
| 17 | Chaos Strike | | Fury | SimC 65 |
| 18 | Felblade | | cooldown | SimC 66 |
| 19-20 | Consuming Fire / Immolation Aura | | cooldown | SimC 67 |
| 21 | Throw Glaive | nothing else | cooldown, Fury | Method |

Fel-Scarred's per-button Demonsurge empowerments have no aura in the log. They are markers:
Metamorphosis' cast empowers Annihilation, Death Sweep, Abyssal Gaze and Consuming Fire; an Eye
Beam or Abyssal Gaze empowers Annihilation and Death Sweep; casting the button spends it; all of
them end when the Metamorphosis aura 162264 is removed (as SimC's `demonsurge_*` buffs do).

**AoE.** SimC's Havoc APL is one list for every target count; its only target-count rules in this
build are Immolation Aura above Annihilation at 2+ targets (line 62). The Burning Wound and
Aldrachi Reaver multi-target lines (39-40, 49, 53-54) need talents this build doesn't take. Neither
Method nor the other public guides give a separate Fel-Scarred AoE priority with thresholds for
12.1.5, so the list adds none of its own.

## What the demo and tests show

`--demo` is a closed loop: a stand-in game presses whatever the overlay says and reports it back
only as log events (casts, auras with remaining time, energizes, swings, damage on each target).
At 17% haste the 1-target opener is Eye Beam (Demonic) → The Hunt → Essence Break → empowered
Death Sweep, and Death Sweep again (Eternal Hunt reset it) → Vengeful Retreat + Felblade →
Annihilation → hardcast Metamorphosis once the window is spent → the new empowered Death Sweep and
Annihilation → Abyssal Gaze → its empowered spenders, then Vengeful Retreat just before the next
Eye Beam at 31s; with 4 targets Consuming Fire moves above Annihilation. The stand-in's Fury income is
only 15 a swing plus Felblade and Immolation Aura (no Tactical Retreat, Student of Suffering,
soul fragments or refunds), so it is starved compared with real play.

`tests/overlay_havoc.rs` builds state from events only and checks: the opener; The Hunt then
Essence Break then Death Sweep/Annihilation in the Demonic window, read from the target's
320338 debuff; the hardcast Metamorphosis after the window and the Chaotic Transformation reset;
Felblade when starved; Immolation Aura above Chaos Strike at 3 targets (and back once they leave
the 3s window); and a 120s loop with no unaffordable press and no idle gap over 4s.

## Known gaps

- **Without combatfeed's `cooldowns` lines, cooldowns are timers from the cast.** Cycle of
  Hatred's per-stack cuts (and the tier 4pc's Essence Break cuts) are then not followed: Eye Beam
  uses the spec's 20s steady state, so the first Eye Beam after the pull is predicted ready ~10s
  early. A cast always restarts the timer, so the error never carries over.
- **Fury between `power` lines** is integrated from energizes minus costs.
- **The look-ahead only moves when a line arrives.** A cooldown coming up between lines shows on the
  next line (combatfeed sends aura and cooldown changes as they happen, so this is short in combat).
- **Haste from gear** is a setting (`--haste`), not read from the log.
- **Aura durations** come from the event's remaining time (wow-forever has it); events without it
  fall back to the table's `duration`.
- **Talent choices** are fixed to the profile's build; another build needs its own overlay file.
