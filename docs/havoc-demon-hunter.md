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
| azsim | 158,729 (±74) |
| difference | **-0.59%** |

Stat probes (+500 rating / +200 agility, same calibration):

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| crit | +10.40% | +8.18% | -2.21 |
| haste | +7.72% | +4.66% | -3.06 |
| mastery | +9.70% | +10.42% | +0.72 |
| versatility | +8.42% | +8.50% | +0.08 |
| agility | +6.40% | +6.33% | -0.08 |

## Gear

`[gear]` computes the character's stats from `/simc` slot lines, and
`[gear.kit]` adds a raid-night approximation of SimC's default consumables and
Bloodlust (flask, food, augment rune, both weapon oils, and a Potion of
Recklessness at the pull). `tools/check_gear.py` matches every slot's item
level and every rating against SimC, with and without trinkets: attack power
2,784 (3,018 with trinkets), crit 36.96%, haste 16.86%, mastery 87.70% and
versatility 8.98%. `--gear` gives 158,809 DPS against 158,811 from the
hand-entered final stats (same seed, 5,000 iterations).

Full-gear DPS is not validated: the Fel-Scarred tier set's bonuses (about 10%
in SimC) are not in the effect library and the weapon oil's damage proc is not
modelled, so SimC's full MID2 profile gives 261,044 against azsim's 224,740.
The potion is modelled as +1,725 crit rating and -232 versatility rating for
30s, the highest and lowest secondaries on this gear set; other gear with
different highest/lowest stats would need its own buff.

## What is modelled

- **Fury** (170): Demon Blades on every auto-attack hit (15), Immolation Aura
  (20 on cast, 3 a tick), Felblade (15), Vengeful Retreat's Tactical Retreat
  (80, taken up front instead of 8/s for 10s), Arcane Torrent (15), Student of
  Suffering (15 a channel), Demonic Appetite's soul fragments (a real PPM, 20
  each) and the off-hand refund (50% chance, 20 — the reference's 45% base
  plus Chaos Theory/Critical Chaos). Chaos Strike/Annihilation 40, Blade
  Dance/Death Sweep 35, Eye Beam/Abyssal Gaze 30.
- **Auto attacks**: a dual-wielded white swing misses 16.3% of the time and
  Skyfury repeats 20% of them; Demon Blades rolls only on landed hits. The
  swing cadence is the observed one: Vengeful Retreat leaps out of melee range
  and cancels a swing or two a cast, which is what the 2.94s/3.02s effective
  weapon speeds stand for.
- **Metamorphosis**: 20s, +20% haste; hardcast on a 120s cooldown, or opened by
  Eye Beam's Demonic (5s + the hasted channel). Shattered Destiny extends it
  0.1s per 12 fury spent. Chaotic Transformation resets the Eye Beam lockout and
  Blade Dance, Violent Transformation resets Immolation Aura.
- **Annihilation / Death Sweep** replace Chaos Strike / Blade Dance inside
  Metamorphosis; both are two-hit (main hand, then off hand 300ms later) or
  four-hit (First Blood) abilities, the four Blade Dance/Death Sweep hits as
  four sources (0/150/450/700ms, 0/150/450/950ms).
- **Demonsurge** (Fel-Scarred): Eye Beam/Abyssal Gaze and Metamorphosis hand out
  per-ability empowerments; the empowered Annihilation/Death Sweep/Abyssal
  Gaze/Immolation Aura fires a 2.8 AP Demonsurge hit (0.45s/0.7s delay) and
  stacks the Demonsurge buff (+5% damage a stack, 4 stacks, 15s).
- **Burning Blades**: 35% of every direct hit from auto attacks, Chaos
  Strike/Annihilation and Blade Dance/Death Sweep rolls into a 6s, 2s-tick
  banked DoT.
- **Eye Beam**: 11-tick channel, 2 fury a tick (Blind Fury); Furious Gaze
  (+8% haste) on completion, and the last tick triggers Eternal Hunt (8s), which
  a Blade Dance/Death Sweep then consumes to reset its cooldown. Abyssal Gaze
  inside Metamorphosis (+30%, Demonic Intensity); Eternal Hunt's empowered
  channel doubles the ticks. Eye Beam and Abyssal Gaze share a 20s lockout
  (Cycle of Hatred takes the 30s cooldown to ~20s in steady state).
- **Essence Break** (40s, 4s debuff), whose hits echo for 2.574 AP once a
  second; **The Hunt** (60s, impact + 6s DoT) and its empowered Eye Beam;
  **Immolation Aura** (30s, 10 ticks of 1s, always critting via Isolated Prey);
  **Felblade**, **Vengeful Retreat** (Initiative: +10% crit for 5s),
  **Relentless Onslaught** (10% of off-hand hits), **Chaos Theory** (a +40%
  crit buff on average after Blade Dance/Death Sweep).
- The APL's single-target priority, including the demonsurge-empowerment
  ordering and the Metamorphosis/The Hunt cooldown conditions. Blade Dance and
  Death Sweep are modelled with their own cooldowns (SimC shares one), so the
  APL's `!cooldown.blade_dance.up` gate on Metamorphosis is dropped: read as
  Blade Dance's own 15s cooldown it blocks the hardcast for 10s after every
  Demonic window and Metamorphosis falls from 2.8 casts a fight to 2.1.
- The default gear's embellishments: Venomfang and Rune of Unleashed Fire.

## Known gaps

- **Eye Beam / Abyssal Gaze split**: the family's total is right (16.5 channels
  vs 17.2) but too many of them land outside Metamorphosis, so the plain Eye
  Beam gets 45 ticks where the reference has 71 and Abyssal Gaze 83 where it
  has 58; the empowered channel almost always goes to Eye Beam (54 vs 39)
  instead of Abyssal Gaze (1 vs 22). The per-tick damage is calibrated, so the
  mix costs ~1% of the fight.
- **Blade Dance**: 8.3 casts vs 10.8 (Death Sweep 46.3 vs 43.8). The reference
  shares one cooldown between the two and this draft does not (see above), so
  the out-of-Metamorphosis side is short.
- **Immolation Aura** fires 8.5 times a fight against 9.8 and its ticks 84
  against 105: as the APL's last resort it loses the GCD almost every time.
  Immolation Aura ticks always crit (Isolated Prey), so the reference reports no
  non-crit tick to fit — the calibrator leaves that source at its derived
  multiplier, which leaves it ~35% short (~0.2% of the fight).
- **Venomfang** lands 148 ticks against 164: every re-proc restarts the DoT's
  tick cadence, which costs about one tick per application.
- **Essence Break's echo** procs 21.0 times against 22.9 (its 1s internal
  cooldown is right; the hits inside the 4s window land a little further apart).
- **Critical Chaos's crit-scaled fury refund** is not modelled (the refund is a
  flat 50%, crit only moves damage), so crit rating is worth ~2 points less than
  in the reference and haste ~3 (fewer refunds also means fewer casts per point
  of haste).
- Immolation Aura's Undying Embers re-proc (25% on expiry) is not modelled.
- Relentless Onslaught uses the Annihilation or Chaos Strike values of whichever
  form the player was in when the off-hand hit landed, but not its own 300ms
  delay against a Metamorphosis that ends in between.
