# Beast Mastery Hunter (Pack Leader)


`specs/beast_mastery_pack_leader.toml`, built the same way as the others:
coefficients, costs, cooldowns, durations and talent values from DB2 and
SimC's hunter module (`sc_hunter.cpp`), the rotation paraphrased from the
default single-target APL, and the rest observed in SimC's report. Every
mechanic is generic engine data — the spec is mostly pets: the main pet
("duck", SimC's default name), the Animal Companion, Nature's Ally's copy,
Dire Beasts, Fenryr/Hati and the Pack Leader Bear.

## Results

Reference: SimC's Midnight S2 sample Beast Mastery profile (Pack Leader), core
kit only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk,
5,000 iterations.

| | DPS |
|---|---:|
| reference | 181,956 |
| azsim | 179,109 (±58) |
| difference | **-1.56%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| barbed_shot | 39,591 | 39,129 | -1.2% | 233.3 | 230.7 |
| cobra_shot | 14,335 | 15,004 | +4.7% | 88.7 | 92.9 |
| duck_kill_command | 12,270 | 12,155 | -0.9% | 95.7 | 94.8 |
| animal_companion_kill_command | 12,263 | 12,091 | -1.4% | 95.7 | 94.8 |
| auto_shot | 11,870 | 11,889 | +0.2% | 129.7 | 130.1 |
| natures_ally_pet_kill_command | 7,978 | 6,954 | -12.8% | 57.4 | 50.2 |
| bloodshed (three pets) | 21,135 | 21,096 | -0.2% | 364.5 | 363.9 |
| dire_beast_kill_command | 6,830 | 6,442 | -5.7% | 127.9 | 120.9 |
| stampede_tick | 4,796 | 4,817 | +0.4% | 91.8 | 92.3 |
| boar_charge + cleave | 7,592 | 7,297 | -3.9% | 17.0 | 16.4 |
| dire_beast_melee | 3,840 | 3,900 | +1.5% | 296.9 | 301.7 |
| duck/AC claw and melee | 8,847 | 8,868 | +0.2% | 667.9 | 671.0 |
| fenryr/hati kill_command | 2,391 | 1,584 | -33.8% | 22.4 | 14.8 |
| fenryr/hati melee + leap | 3,942 | 3,423 | -13.2% | 74.2 | 63.6 |
| bear melee + rend + KC | 6,258 | 6,008 | -4.0% | 247.0 | 234.2 |
| stomp (two pets) | 4,780 | 4,939 | +3.3% | 117.8 | 121.6 |
| rune/void/acid procs | 2,528 | 2,524 | -0.2% | — | — |

The focus economy matches: 88.7 Cobra Shots (92.9), 95.7 Kill Commands (94.8),
58.9 Barbed Shots (60.8) and 10.3 Bestial Wraths a fight. Auto Shot lands
130.1 times (129.7) and pet melee 272.7 (271.4).

## Validation

`tools/validate_stats.py`, the same calibration applied to the five probes:

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| +500 crit | +8.93% | +7.24% | -1.69 |
| +500 haste | +8.59% | +8.01% | -0.57 |
| +500 mastery | +8.23% | +8.29% | +0.05 |
| +500 versatility | +8.62% | +8.82% | +0.20 |
| +200 agility | +5.86% | +6.02% | +0.16 |

## Known gaps

- **Crit scaling (-1.69 pts).** Go for the Throat makes Kill Command crits
  deal extra damage proportional to crit chance (a quadratic response to
  crit rating). The engine only has a fixed crit-damage bonus, so Kill
  Command's crit response is linear, and the spec's per-source crit bonuses
  dilute the total crit gain.
- **Nature's Ally pet Kill Commands (50 vs 57).** In SimC the Kill Commands
  cluster into Bestial Wrath windows (the APL holds them for the howl beast),
  more than the pet's 52% uptime alone would give. The spec prioritises Kill
  Command while the pet is out but still casts it uniformly.
- **Fenryr/Hati (15 vs 22 Kill Commands).** The Huntmaster's Call counter and
  summon counts match (5.5 vs 6.0 a fight); their 10s windows see slightly
  fewer Kill Commands than SimC's, and the fight end clips some summons.
- **Gear.** No `[gear]`/`[gear.kit]` yet, so `--gear` is not supported and
  the sample gear's stat procs are folded into `[player]` (haste is the
  fight average: 130 Auto Shots a fight).
- **Residuals.** Barbed Shot's rolling periodic accumulates damage differently
  from SimC's rolling multiplier (the engine keeps a bounded average), and
  Bloody Frenzy's faster ticks are baked into the tick cadence; calibration
  absorbs both in the per-hit multiplier.

## Engine notes

Two generic pet-scheduling fixes, both no-ops for specs that already
complete (verified: every existing spec gives identical DPS):

- a pet action that is within a microsecond of affordable is retried with
  at least a 1-microsecond wait instead of at the same timestamp;
- a per-instance cooldown that lands inside the current microsecond no
  longer re-schedules `PetAct` at that same timestamp.

Without them a pet on a focus budget could spin forever at one timestamp.
