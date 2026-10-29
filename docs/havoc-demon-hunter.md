# Havoc Demon Hunter (Fel-Scarred)

`specs/havoc_fel_scarred.toml` — first draft, calibrated. Coefficients, costs, cooldowns,
durations and talent values come from DB2 (`tools/spelldb.py`), SimC's demon
hunter module (`sc_demon_hunter.cpp`) and its default APL where a mechanic is
scripted, and the rest from SimC's log. Every mechanic is generic engine data
(fury, charges, channels, banked DoTs, per-hit procs); no engine code changed
for this spec.

## Results

Reference: SimC's Midnight S2 sample Havoc profile (Fel-Scarred), core kit only
(no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. azsim: 10,000 iterations.

| | DPS |
|---|---:|
| reference | 159,675 |
| azsim | 160,287 (±70) |
| difference | **+0.38%** |

Stat probes (+500 rating / +200 agility, same calibration):

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| crit | +10.40% | +8.18% | -2.22 |
| haste | +7.72% | +6.41% | -1.31 |
| mastery | +9.70% | +10.22% | +0.52 |
| versatility | +8.42% | +8.50% | +0.08 |
| agility | +6.40% | +6.26% | -0.14 |

## What is modelled

- **Fury** (170): Demon Blades on every auto-attack hit (15), Immolation Aura
  (20 on cast, 3 a tick), Felblade (15), Vengeful Retreat's Tactical Retreat
  (80, taken up front instead of 8/s for 10s), Arcane Torrent (15) and the
  off-hand refund (45% chance, 20). Chaos Strike/Annihilation 40, Blade
  Dance/Death Sweep 35, Eye Beam/Abyssal Gaze 30.
- **Metamorphosis**: 20s, +20% haste; hardcast on a 120s cooldown, or opened by
  Eye Beam's Demonic (5s + the hasted channel). Shattered Destiny extends it
  0.1s per 12 fury spent. Chaotic Transformation resets the Eye Beam lockout and
  Blade Dance, Violent Transformation resets Immolation Aura.
- **Annihilation / Death Sweep** replace Chaos Strike / Blade Dance inside
  Metamorphosis; both are two-hit (main hand, then off hand 300ms later) or
  four-hit (First Blood) abilities.
- **Demonsurge** (Fel-Scarred): Eye Beam/Abyssal Gaze and Metamorphosis hand out
  per-ability empowerments; the empowered Annihilation/Death Sweep/Abyssal
  Gaze/Immolation Aura fires a 2.8 AP Demonsurge hit (0.45s/0.7s delay) and
  stacks the Demonsurge buff (+5% damage a stack, 4 stacks, 15s).
- **Burning Blades**: 35% of every direct hit from auto attacks, Chaos
  Strike/Annihilation and Blade Dance/Death Sweep rolls into a 6s, 2s-tick
  banked DoT.
- **Eye Beam**: 11-tick channel, 2 fury a tick (Blind Fury); Furious Gaze
  (+8% haste) and Eternal Hunt on completion; Abyssal Gaze inside Metamorphosis
  (+30%, Demonic Intensity); Eternal Hunt's empowered channel doubles the ticks.
  Eye Beam and Abyssal Gaze share a 20s lockout (Cycle of Hatred takes the 30s
  cooldown to ~20s in steady state).
- **Essence Break** (40s, 4s debuff), whose hits echo for 2.574 AP once a
  second; **The Hunt** (60s, impact + 6s DoT) and its empowered Eye Beam;
  **Immolation Aura** (30s, 10 ticks of 1s, always critting via Isolated Prey);
  **Felblade**, **Vengeful Retreat** (Initiative: +10% crit for 5s),
  **Relentless Onslaught** (10% of off-hand hits), **Chaos Theory** (a +40%
  crit buff on average after Blade Dance/Death Sweep).
- The APL's single-target priority, including the demonsurge-empowerment
  ordering and the Metamorphosis/The Hunt cooldown conditions.
- The default gear's embellishments: Venomfang and Rune of Unleashed Fire.

## Known gaps

- **Empowered Eye Beam windows are missed**: The Hunt's 12s empowerment often
  expires before the next Eye Beam, so azsim gets 20 of the reference's 39
  empowered ticks (-2.1% of the fight).
- **Meta windows**: Annihilation and Death Sweep hit counts match (112/49 vs
  113/44), but Chaos Strike (30 vs 45) and Blade Dance (8 vs 11) are short, so
  more of the fight is spent in Metamorphosis than in the reference.
- **No miss chance**: auto attacks (161 vs 140 hits) and Demon Blades (322 vs
  276) land every swing; their per-hit damage is calibrated but their share of
  the total is ~15% high, which the short spender counts roughly offset.
- **Critical Chaos's crit-scaled fury refund** is not modelled, so crit rating
  is worth ~2 points less than in the reference; Immolation Aura ticks are
  unhasted (as in the reference's log), which costs ~1 point of haste.
- Immolation Aura's Undying Embers re-proc (25% on expiry) is not modelled.
- Relentless Onslaught always uses the Annihilation values (the Chaos Strike
  version differs by ~13%), and Blade Dance/Death Sweep report their three
  light hits as one source (`hits = 3`) instead of three rows.
