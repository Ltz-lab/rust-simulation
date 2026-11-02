# Enhancement Shaman (Stormbringer)

MID2 Stormbringer single-target core kit, using
`profiles/MID2_Shaman_Enhancement.simc` and the committed `reference/enh_*.json`
SimC observations. Run with `specs/enhancement_stormbringer.toml` and
`calibration/enhancement_stormbringer.toml`. The reference excludes trinkets,
consumables, set bonuses and Bloodlust.

At 2,000 iterations, seed 1, azsim gives **142,652 DPS** against SimC
**148,565** (-4.0%). Calibration fits damage per hit, not counts; the counts
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
  number of stacks; Storm Unleashed makes it ignore its cooldown and leave a
  6-tick storm; Stormwell turns a single-target Crash Lightning into a Windfury
  Weapon proc.
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

- **Hit counts.** Per fight, azsim has Lightning Bolt 44.2 vs 36.5, Crash
  Lightning 29.2 vs 40.3, Windstrike 28.3 vs 37.7, Lava Lash 8.8 vs 14.1,
  Voltaic Blaze 11.5 vs 15.2, Tempest (from Thorim's Invocation) 7.0 vs 11.6,
  and more auto attacks, flametongue hits and manual Flame Shocks. The
  rotation is GCD-bound and those surpluses crowd out the Lava Lash / Voltaic
  Blaze / Crash Lightning filler, so the DPS sits 4.0% low even though every
  modelled source matches per hit.
- The Fire Nova talent (Voltaic Blaze rolls a Fire Nova Explosion, about 6 a
  fight) is not modelled. The MID2 2-piece's Fire Nova Explosion is, with gear
  lines (see Tier set below).
- Windfury/Stormstrike proc rolls use SimC's per-hit chance; SimC's own
  deck-RNG guarantees (Tempest, Storm Unleashed, Doom Winds from stacks) are
  approximated with flat per-stack chances.
- No `[gear]` table yet: the spec runs on the reference's buffed stats.

## Tier set (Midnight season 2, set 2065)

`[[gear.effects]]` entries for 2pc 1296627 and 4pc 1296628 in `specs/enhancement_stormbringer.toml`, applied with gear lines when the pieces are worn (`--set gear.set_bonuses=0` turns them off). Mechanics from `sc_shaman.cpp` (`burning_core`, `short_circuit`, `fire_nova_explosion_t`, `crash_lightning_t`, `crash_lightning_attack_t`), numbers from the set spells and Burning Core / Short Circuit (1299975, 1299991) in the game data.

- **2pc**: Voltaic Blaze leaves Burning Core for 6s; it ticks every 2s and each tick is a Fire Nova Explosion (333977, 0.99682 AP), which Burning Core makes deal +200%. Three explosions per Voltaic Blaze, at 2, 4 and 6s. The explosion is a new source with no calibration entry: its multiplier is the spell-data `derived` value (1.4025: the spec's +25%, Elemental Tempo, Natural Gift) times the residual the calibration fits for Ascendance's hit (same derived set), 1.062781, so it follows the other sources rather than being fitted to its own hits. Average hit, crits included: 59,759 in azsim against 58,767 in SimC (SimC's average includes about 6 unamplified Fire Nova talent explosions a fight); crit rate 35.2% against 40.2%, the same offset azsim has on its other fire hits (Flametongue Attack 36.3% against 41.6% without the set).
- **4pc**: each explosion takes 2s off Crash Lightning's cooldown and adds a Short Circuit stack (+8%, 5 stacks, 15s). A Crash Lightning cast spends the stacks: the Crash Lightning hit that Stormstrike and Lava Lash add is multiplied by 1 + the average of the live enhancement stacks' values, and the storm it leaves multiplies each tick by its own stacks.
- **Rotation**: with the 2pc the Stormbringer APL casts Voltaic Blaze ahead of Crash Lightning while Maelstrom Weapon is under 9 stacks (`voltaic_blaze,if=set_bonus.midnight_season_2_2pc&buff.maelstrom_weapon.stack<9`); the fragment adds that line. Without the set the rotation is unchanged.

Profile `MID2_Shaman_Enhancement`, gear lines, seed 1, Patchwerk 300s, one target; SimC 0.05% error, azsim 12,000 iterations (0.07% at 95%). Off is as in [elemental-shaman.md](elemental-shaman.md#tier-set-midnight-season-2-set-2065).

| | SimC | azsim |
|---|---:|---:|
| Full profile DPS | 248,245 | 230,414 (-7.2%) |
| Set bonuses off | 218,796 | 203,688 (-6.9%) |
| Tier gain | +13.46% | +13.12% |
| 2pc alone | +6.7% | +8.68% |
| 4pc on top of the 2pc | +6.1% | +4.08% |
| Voltaic Blaze casts a fight (full / off) | 28.5 / 15.4 | 28.1 / 11.7 |
| Fire Nova Explosion hits a fight (full) | 92.5 | 83.1 |
| Crash Lightning casts a fight (full / off) | 55.9 / 43.5 | 43.8 / 31.2 |

The total is close but the two halves are off by about two points each, in opposite directions (the 2pc-alone rows for SimC come from a 0.14% error run, about +-0.2 points). The 2pc reads high, most likely because azsim's off baseline casts Voltaic Blaze less often (11.7 a fight against 15.4); the set's priority line takes both to about 28, so azsim gains 16 casts of three explosions each where SimC gains 13. The 4pc reads low because azsim's Storm Unleashed storm hangs on its own buff, not on the Crash Lightning cast, so the 12 extra casts the 4pc brings add no storm hits (SimC: 100.2 storm hits a fight against 77.9 off; azsim 65.3 against 66.4). Not modelled: the Fire Nova talent's explosions (they would also feed the 4pc), and Feral Spirit's Crackling Surge and Molten Weapon stacks, which the explosion shares with the calibration's residual.
