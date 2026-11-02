# Enhancement Shaman (Stormbringer)

MID2 Stormbringer single-target core kit, using
`profiles/MID2_Shaman_Enhancement.simc` and the committed `reference/enh_*.json`
SimC observations. Run with `specs/enhancement_stormbringer.toml` and
`calibration/enhancement_stormbringer.toml`. The reference excludes trinkets,
consumables, set bonuses and Bloodlust.

At 10,000 iterations, seed 1, azsim gives **141,229 DPS** against SimC
**148,565** (-4.94%). Calibration fits damage per hit, not counts; the counts
below come out of the rotation and procs.

## What is modelled

- **Maelstrom Weapon** is a buff, not a resource. Every melee-class hit has a
  20% chance (187880) to add a stack (max 10, Overflowing Maelstrom), and
  Lightning Bolt / Chain Lightning / Tempest spend every stack: +20% damage
  and -20% cast time each (344179), so 5 stacks make them instant. Flametongue
  Attack is Magic-school and deliberately does not roll for stacks, matching
  SimC (`sc_shaman.cpp`, `action_init_finished`).
- Enhancement's spell power is 101% of attack power (137041 effect #6), so
  every spell coefficient is expressed as an attack-power coefficient
  (`sp_coeff x 1.01`) and scales with the same AP the engine computes.
- **Stormstrike** (two charges, Stormblast) with Stormsurge (5% + 30% of
  mastery per hit) resetting it and giving the next strike an extra 25% hit;
  Stormflurry chains 40% strikes; Ride the Lightning adds a full Chain
  Lightning to Stormstrike and Lava Lash; Stormblast's extra hits never crit.
- **Ascendance** replaces Stormstrike with Windstrike (armour-ignoring), starts
  Doom Winds, upgrades the next Lightning Bolt to Tempest (Descending Skies)
  and recharges Windstrike 60% faster. Elemental Tempo takes 0.3s off
  Stormstrike and Lava Lash per Maelstrom Weapon spent.
- **Thorim's Invocation** re-casts the last Lightning Bolt (or Tempest, or
  Chain Lightning) for free from Stormstrike while Doom Winds is up, and from
  Windstrike for all of Ascendance.
- **Crash Lightning** leaves a 12s weapon enhancement (overlapping stacks) that
  adds a Crash Lightning hit to every Stormstrike/Lava Lash, scaled by the
  number of stacks; each cast schedules two storm ticks, each with a 90% hit
  chance. Storm Unleashed grants a cooldown-bypassing cast (approximated as a
  reset here); Stormwell turns a single-target cast into a Windfury Weapon proc.
- Windfury Weapon (three attacks, Unruly Winds), Flametongue Weapon (a fire hit
  on every melee hit), auto attacks with Flurry, Flame Shock (refreshed by Lava
  Lash and Voltaic Blaze), Tempest, Arc Discharge doubling the next Chain
  Lightning, Lightning Strikes, Unlimited Power, Storm Swell, Lightning Rod
  (10% of Lightning damage dealt again, never critting), Doom Winds ticks,
  Static Accumulation stacks, Feral Spirit lightning wolves and the gear's
  fixed damage procs (Rune of Unleashed Fire, Acidbrood Scourge, Void Eruption,
  Venomfang).
- Overcharge (1251026) needs crit damage that grows with crit chance, so the
  engine gained a generic `crit_damage_per_crit` damage field (documented in
  [spec-format.md](spec-format.md)).

## Stat probes

20,000 iterations, paired seed 42; +500 rating, +200 agility:

| probe | SimC gain | azsim gain | difference |
|---|---:|---:|---:|
| crit | +9.54% | +8.25% | -1.29 pts |
| haste | +10.81% | +12.07% | +1.25 pts |
| mastery | +9.26% | +9.05% | -0.21 pts |
| versatility | +8.83% | +8.79% | -0.03 pts |
| agility | +6.52% | +6.52% | 0.00 pts |

## Known gaps

- **Hit counts.** In the core profile, azsim has Lightning Bolt 40.9 vs 36.5,
  Crash Lightning 28.6 vs 40.3, Lava Lash 9.1 vs 14.1 and Tempest from
  Thorim's Invocation 8.2 vs 11.6 per fight. The rotation and proc approximations
  leave DPS 4.94% low even though calibrated sources match per hit.
- **Storm Unleashed cooldown.** SimC uses a separate cooldown for proc casts,
  leaving the normal Crash Lightning cooldown running; azsim resets the normal
  cooldown instead, losing casts. The storm ticks also use the current 4pc buff
  rather than the multiplier snapshotted when each cast started.
- The Fire Nova talent (Voltaic Blaze rolls a Fire Nova Explosion, about 6 a
  fight) is not modelled. The MID2 2-piece's Fire Nova Explosion is, with gear
  lines (see Tier set below).
- Windfury/Stormstrike proc rolls use SimC's per-hit chance; SimC's own
  deck-RNG guarantees (Tempest, Storm Unleashed, Doom Winds from stacks) are
  approximated with flat per-stack chances.
- No `[gear]` table yet: the spec runs on the reference's buffed stats.

## Tier set (Midnight season 2, set 2065)

`[[gear.effects]]` entries for 2pc 1296627 and 4pc 1296628 in `specs/enhancement_stormbringer.toml`, applied with gear lines when the pieces are worn (`--set gear.set_bonuses=0` turns them off). Mechanics from `sc_shaman.cpp` (`burning_core`, `short_circuit`, `fire_nova_explosion_t`, `crash_lightning_t`, `crash_lightning_attack_t`), numbers from the set spells and Burning Core / Short Circuit (1299975, 1299991) in the game data.

- **2pc**: Voltaic Blaze leaves Burning Core for 6s; it ticks every 2s and each tick is a Fire Nova Explosion (333977, 0.99682 AP), which Burning Core makes deal +200%. Three explosions per Voltaic Blaze, at 2, 4 and 6s. The explosion has no calibration entry; its 1.122 multiplier comes from Elemental Tempo and Natural Gift, with the spec's global multiplier applied separately. The Fire Nova talent's additional explosions and other dynamic modifiers remain unmodelled.
- **4pc**: each explosion takes 2s off Crash Lightning's cooldown and adds a Short Circuit stack (+8%, 5 stacks, 15s). A Crash Lightning cast spends the stacks: the Crash Lightning hit that Stormstrike and Lava Lash add is multiplied by 1 + the average of the live enhancement stacks' values, and the storm it leaves multiplies each tick by its own stacks.
- **Rotation**: with the 2pc the Stormbringer APL casts Voltaic Blaze ahead of Crash Lightning while Maelstrom Weapon is under 9 stacks (`voltaic_blaze,if=set_bonus.midnight_season_2_2pc&buff.maelstrom_weapon.stack<9`); the fragment adds that line. Without the set the rotation is unchanged.

Profile `MID2_Shaman_Enhancement`, gear lines, seed 1, Patchwerk 300s, one target; SimC target error 0.1%, azsim 10,000 iterations (±154 DPS at 95%). SimC gives **248,110 DPS** and azsim **227,176** (-8.44%).

| Source | SimC per fight | azsim per fight |
|---|---:|---:|
| Crash Lightning casts | 55.7 | 43.3 |
| Crash Lightning storm hits | 99.8 | 77.5 |
| Fire Nova Explosion hits | 92.2 | 83.1 |
| Lightning Bolt casts | 35.8 | 41.9 |
| Lava Lash casts | 13.3 | 8.3 |

The set's explosion currently averages 44,494 damage in azsim versus about 58,800 in SimC. The Fire Nova talent's extra explosions, Feral Spirit's Crackling Surge and Molten Weapon stacks, and correct 4pc storm snapshotting remain unmodelled; these results are not yet within the 3% target.
