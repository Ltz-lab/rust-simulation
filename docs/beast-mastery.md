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
| azsim | 178,658 (±61) |
| difference | **-1.81%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| barbed_shot | 39,591 | 39,049 | -1.4% | 233.3 | 230.1 |
| cobra_shot | 14,335 | 14,880 | +3.8% | 88.7 | 92.2 |
| duck_kill_command | 12,270 | 12,204 | -0.5% | 95.7 | 95.2 |
| animal_companion_kill_command | 12,263 | 12,144 | -1.0% | 95.7 | 95.2 |
| auto_shot | 11,870 | 11,891 | +0.2% | 129.7 | 130.1 |
| natures_ally_pet_kill_command | 7,978 | 6,799 | -14.8% | 57.4 | 49.0 |
| bloodshed (three pets) | 21,135 | 21,024 | -0.5% | 364.5 | 362.7 |
| dire_beast_kill_command | 6,830 | 6,442 | -5.7% | 127.9 | 120.9 |
| stampede_tick | 4,796 | 4,799 | +0.1% | 91.8 | 91.9 |
| boar_charge + cleave | 7,592 | 7,281 | -4.1% | 17.0 | 16.4 |
| dire_beast_melee | 3,840 | 3,900 | +1.6% | 296.9 | 301.7 |
| duck/AC claw and melee | 8,847 | 8,870 | +0.2% | 667.9 | 671.1 |
| fenryr/hati kill_command | 2,391 | 1,598 | -33.2% | 22.4 | 15.0 |
| fenryr/hati melee + leap | 3,942 | 3,528 | -10.5% | 74.2 | 68.4 |
| bear melee + rend + KC | 6,258 | 5,961 | -4.7% | 247.0 | 233.6 |
| stomp (two pets) | 4,780 | 4,848 | +1.4% | 117.8 | 119.4 |
| rune/void/acid procs | 2,528 | 2,526 | -0.1% | — | — |

The focus economy matches: 88.7 Cobra Shots (92.2), 95.7 Kill Commands (95.2),
58.9 Barbed Shots (59.7) and 10.3 Bestial Wraths a fight. Auto Shot lands
130.1 times (129.7) and pet melee 272.7 (271.4). Barbed Shot ticks 230 times
(233) with Bloody Frenzy's tick-rate buff modelled through `tick_rate_pct`.

## Validation

`tools/validate_stats.py`, the same calibration applied to the five probes:

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| +500 crit | +8.93% | +7.40% | -1.52 |
| +500 haste | +8.59% | +8.48% | -0.11 |
| +500 mastery | +8.23% | +8.29% | +0.05 |
| +500 versatility | +8.62% | +8.82% | +0.20 |
| +200 agility | +5.86% | +6.02% | +0.16 |

## Known gaps

- **Total DPS is 1.8% low** (178,658 against 181,956). This is a known issue,
  landed as is. The items below account for it.
- **Crit scaling (-1.52 pts).** Go for the Throat makes Kill Command crits
  deal extra damage proportional to crit chance (a quadratic response to
  crit rating). The engine only has a fixed crit-damage bonus, so Kill
  Command's crit response is linear, and the spec's per-source crit bonuses
  dilute the total crit gain.
- **Nature's Ally pet Kill Commands (49 vs 57).** In SimC the Kill Commands
  cluster into windows where the Nature's Ally buff is up more than the
  pet's ~52% uptime would suggest. The spec prioritises Kill Command while
  the pet is out but still casts it uniformly.
- **Fenryr/Hati (15 vs 22 Kill Commands).** The Huntmaster's Call counter and
  summon counts match (5.5 vs 6.0 a fight); their 10s windows see slightly
  fewer Kill Commands than SimC's, and the fight end clips some summons.
  Dire Beasts are 6% short for the same reason.
- **Gear.** No `[gear]`/`[gear.kit]` yet, so `--gear` is not supported and
  the sample gear's stat procs are folded into `[player]` (haste is the
  fight average: 130 Auto Shots a fight).
- **Residuals.** Barbed Shot's rolling periodic accumulates damage differently
  from SimC's rolling multiplier (the engine keeps a bounded average), so
  calibration absorbs that in the per-hit multiplier (2.2x on the tick after
  the derived 2.13x).

## Engine notes

Two generic pet-scheduling fixes, both no-ops for specs that already
complete (verified: all eleven existing specs give identical DPS before and
after):

- a pet action that is a microsecond short of affordable is retried with
  at least a 1-microsecond wait instead of at the same timestamp;
- a per-instance cooldown that lands inside the current microsecond no
  longer re-schedules `PetAct` at that same timestamp.

Without them a pet on a focus budget could spin forever at one timestamp.
