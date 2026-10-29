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
- Fire Nova (Voltaic Blaze's Fire Nova talent) and the MID2 2-piece's Fire Nova
  Explosion are not modelled.
- Windfury/Stormstrike proc rolls use SimC's per-hit chance; SimC's own
  deck-RNG guarantees (Tempest, Storm Unleashed, Doom Winds from stacks) are
  approximated with flat per-stack chances.
- No `[gear]` table yet: the spec runs on the reference's buffed stats.
