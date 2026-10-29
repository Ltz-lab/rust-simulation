# Unholy Death Knight (Rider of the Apocalypse)

Midnight Season 2, core kit: `specs/unholy_rider.toml`, calibrated by
`calibration/unholy.toml`, derived modifiers in `derived/unholy.toml`. The
reference is `reference/unholy_core_observations.json`, built by
`tools/make_reference.sh` from
`profiles/MID2/MID2_Death_Knight_Unholy.simc` (talents, default APL, hero
tree Rider of the Apocalypse).

| | SimC | azsim | diff |
|---|---:|---:|---:|
| DPS (10,000 iterations, seed 1) | 148,582 | 147,786 | -0.54% |

Stat probes (`tools/validate_stats.py`, same calibration):

| probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit rating | +11.60% | +11.09% | -0.52 pts |
| +500 haste rating | +9.06% | +7.15% | -1.91 pts |
| +500 mastery rating | +9.01% | +7.15% | -1.86 pts |
| +500 versatility rating | +8.79% | +8.86% | +0.07 pts |
| +200 strength | +6.11% | +8.50% | +2.40 pts |

## What is modelled

- **Runes and runic power.** Six runes regenerating at `0.20/s` base, hasted
  and doubled by Runic Corruption (60% chance per 30 RP, 3s); Rune Mastery's
  15%-per-rune 8s +6% regen. Runic power from auto attacks (Runic
  Attenuation), Festering Strike (20), Scourge Strike / Soul Reaper /
  Putrefy / Outbreak (10) and Army of the Dead (50); Death Coil spends 30.
- **Midnight unholy.** Festering Strike costs two runes and leaves the free
  Festering Scythe follow-up (which also applies the 35%-faster-plague
  debuff); both grant Lesser Ghoul stacks. Scourge Strike spends a stack to
  summon a ghoul and erupts both plagues at 35% of a tick with mastery
  double-dipped. Soul Reaper spends up to three stacks. Putrefy summons a
  ghoul that strikes and explodes, Putrid Echoes repeats it at two charges,
  and Harbinger of Doom takes 2.5s off its recharge per Scourge Strike.
- **Diseases and procs.** Dread and Virulent Plague tick every 1.4625s
  (Ebon Fever -25% and Festering Scythe -35% baked in, no haste — SimC's
  diseases are not hasted), permanently maintained by Mograine's Death and
  Decay extensions. Dread Plague ticks roll Sudden Doom (35%: free +35%
  Death Coil) and the Forbidden Knowledge reanimation strike (20%).
  Infected Claws, the Pestilence runeforge, Venomfang, Rune of Unleashed Fire
  and the gear/embellishment procs are clocks or on-hit rolls.
- **Cooldowns.** Army of the Dead (90s: eight ghouls, a Magus, an
  Abomination and all four riders, plus Forbidden Knowledge) and Dark
  Transformation (45s, off GCD: 200% ghoul damage, Commander of the Dead,
  a Magus, Whitemane, Soul Reaper reset, 0.5s per RP spent). The APL casts
  Dark Transformation once Army ghouls are out and Soul Reaper when Dark
  Transformation has under 38s left.
- **Pets.** Raise Dead ghoul, lesser ghouls, Army ghouls (1.4x AP, Death
  Orders on RP spends and summons during Army), Army/Dark Transformation
  magi, the near-permanent Lord of the Dead, Raise Abomination and the four
  horsemen. Pets inherit player crit (their crits are ~3.3x, from Grave
  Mastery/mastery, so pet abilities carry `crit_damage_bonus = 1.28`).
- **Rider of the Apocalypse** abilities: Whitemane's Death Coil and Let
  Terror Reign, Mograine's Heart Strike and Death and Decay, Nazgrim's
  Scourge Strike (physical + shadow), Trollbane's Obliterate and Icy Fury,
  and the Undeath DoT.

## Engine change

`src/sim.rs`: a pet whose next energy payment is less than a microsecond away
used to schedule its action at the current microsecond again and spin forever.
The wait is now floored at one microsecond. Every existing spec's DPS is
unchanged (checked with `-n 2000 --seed 1 --calib ...` before and after).

## Known gaps

- **Pet damage is hand-fitted to the reference total.** SimC's per-actor
  report attributes pet damage both to the pets and to the owner's summon
  actions, so its pet sections over-count, and pet sources are reported with
  a `<pet>_` prefix the calibration tool cannot match. Pet ability coefficients
  start from SimC's per-actor damage per execute and are then scaled by 0.9
  to land on the reference total. Death Order, Ruptured Viscera, Undeath and
  Trollbane's Icy Fury are player-side abilities and *are* calibrated.
- **Reference row collisions.** SimC's report merges rows by action name:
  the player's own Death Coil (47632) and auto attack are overwritten by
  Whitemane's Death Coil and a horseman's auto attack, so they have no
  reference row to calibrate against. They are named `death_coil_player` and
  `auto_attack_player` and hand-fitted to SimC's per-actor averages.
- **Cast counts** are within ~10–20% of SimC per ability but not exact: the
  APL paraphrase uses a continuous rune pool and a coarser Festering
  Strike/Lesser Ghoul bookkeeping than SimC, and SimC waits on runes for
  ~25% of GCDs while the engine casts every GCD it can. Necrotic Coil casts
  (Forbidden Knowledge uptime) and Scourge Strike counts are the largest
  gaps.
- **Hasted pet energy** is not an engine feature; the baked `energy_regen`
  and pet swing times stand in for SimC's hasted 10/s and Unholy Devotion's
  25% attack speed.
- **DoT `on_expire` only fires for overlapping DoTs** in the engine. Venomfang
  is modelled as a one-stack overlapping DoT to get its burst; a shared-dot
  fix would change existing specs and was left alone.
- No `[gear]` table, trinkets, tier set (the reference is the core kit) and
  no multi-target APL. Strength is the most sensitive probe (+2.4 points):
  every azsim source scales with attack power, while SimC's melee abilities
  and autos have a weapon-damage share that does not.
