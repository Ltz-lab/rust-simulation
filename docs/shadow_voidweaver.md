# Shadow Priest (Voidweaver) — draft

`specs/shadow_voidweaver.toml`, reference `reference/shadow_*` (from
`profiles/MID2/MID2_Priest_Shadow.simc`, 300s Patchwerk, core kit), talents
`CIQAAAAAAAAAAAAAAAAAAAAAAMMjZGAAAAAAAAAAAAMLmxMbzMMz2MzYG2mZGzMzYDZGLmpBYmZGAIAz2stEMbMAwgxMzMmtxMYmBzgB`
(spec 258, hero tree Voidweaver).

**DRAFT.** The spec runs and covers every damage source of the rotation; hit
counts are within a few percent of the reference except Mind Blast and Void
Blast. It is not registered in `src/api.rs` and has no `[gear]` lines.

## Results

| run | DPS | reference |
|---|---:|---:|
| core kit, `-n 500`, no calibration | 55,463 | — |
| core kit, `-n 500`, `--calib calibration/shadow_voidweaver.toml` | 133,464 | 143,444 (−7.0%) |

Per-hit damage is calibrated (`tools/calibrate.py --derived
derived/shadow_voidweaver.toml`); every modelled source matches its observed
per-hit damage within 0.01%. Per-source DPS is within a few percent apart
from Mind Blast (7.2k vs 10.4k) and Void Blast (9.0k vs 12.0k): both
cooldowns come up more often than the rotation can fit them.

Stat probes (`tools/validate_stats.py`, same calibration):

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| +500 crit | +10.39% | +10.22% | −0.17 |
| +500 haste | +9.72% | +9.45% | −0.27 |
| +500 mastery | +8.26% | +7.68% | −0.58 |
| +500 versatility | +8.81% | +8.80% | −0.01 |
| +200 intellect | +6.44% | +6.58% | +0.14 |

Stat priority order matches: crit > haste > vers > mastery > int.

## Modelled

- **Resource**: Insanity (0/100). Gains from a logged SimC fight: Mind Blast
  6, Void Blast 8, Mind Flay 3.9 per tick, Void Torrent 6 per tick, Void
  Volley 10, Tentacle Slam 6, Shadow Word: Pain 3, Shadow Word: Death 4,
  Vampiric Touch 4, Auspicious Spirits 1 per apparition, ~3 from the Idol of
  C'Thun tendrils the Void Apparitions talent rolls off Tentacle Slam.
  Shadow Word: Madness costs 45 (50 − 5 Mind's Eye).
- **DoTs**: Shadow Word: Pain (direct + DoT, taps immediately, refresh
  extends), Vampiric Touch, Shadow Word: Madness (rolling refresh), Mind Flay
  (2-tick channel, as the APL interrupts it), Void Torrent (channel with tick
  zero), Entropic Rift.
- **Cooldowns**: Voidform (120s, +20% damage, 3 Void Volley charges, free
  volley on entry, extended 1.5s per Shadow Word: Madness, Crushing Void
  charge on expiry), Power Infusion (120s, +20% haste), Void Torrent (30s,
  off the GCD, opens the rift), Tentacle Slam (15s recharge, 2 charges),
  Shadow Word: Death (10s), Mind Blast / Void Blast (9s hasted, separate
  cooldowns), Void Volley (charges).
- **Procs**: Shadowy Apparitions (Shadow Word: Madness casts and Shadow Word:
  Pain ticks via Tormented Spirits), Void Apparitions (40% of spawns, also
  fire a Void Bolt), Shadeburst on every apparition, Haunting Shadows
  (apparitions extend both DoTs by 1s), Collapsing Void on rift expiry,
  rune of the unleashed fire (10.04/min).
- **Pets**: Thing from Beyond (Idol of Yogg-Saron, ~1.81 spawns/min, Void
  Spike), Shadowfiend (40% on an execute-range Shadow Word: Death),
  Voidwraith (on Entropic Rift expiry).
- **Mastery**: every Shadow source gets `(1 + mastery)^2`
  (`mastery_power = 2`) — the shadow-weaving multiplier plus the spec mastery
  aura; the +500 mastery probe moves every source by the same 8.2%.
- Derived modifiers from `tools/effects.py` (spec aura 137033, talents), see
  `derived/shadow_voidweaver.toml`.

## Known gaps

- **Mind Blast / Void Blast casts are low** (22.5 + 21.6 vs 32.2 + 28.7, −3.8%
  DPS). The GCD budget with the modelled priority cannot fit both 9s
  cooldowns as often as the reference does.
- **Idol of N'Zoth** is fired as two fixed-rate procs (horrific_vision
  4.8/min, vision_of_nzoth 3.96/min) rather than by the 50/100-stack debuff.
  The insanity those buffs tick out is folded into the two abilities.
- **Maddening Tentacles**: Tentacle Slam's free, 25%-damage Shadow Word:
  Madness application is modelled as its direct hit plus the apparition it
  spawns (`tentacle_slam_swm`), not as a second application of the DoT — the
  engine keys one DoT per name. Tick counts match; the weak ticks are
  missing.
- **Effective DoT cadence**: Shadow Word: Pain and Vampiric Touch use
  `tick_time` 1.1s / 1.9s (vs 2s / 3s in the game data) so their tick counts
  match the reference. SimC's Screams of the Void (+40% tick rate while up)
  and "periodic refresh extends duration" produce a faster cadence than the
  base data, and the engine has no DoT tick-speed hook.
- **Crit multiplier 2.20** is fitted so the +500 crit probe matches; the
  logged median crit/noncrit ratio is 2.06. With 2.06 the probe comes out
  1.1 points low and the crit/haste priority order flips.
- **Not modelled**: Psychic Link (multi-target only), Insidious Ire (+20%
  while both DoTs are up), Voidheart, Shadowform (+10%), ancient madness
  (haste on Voidform expiry), Mind's Eye / Shadowy Insight charge mechanics,
  Shadow Weaving (the mastery bonus damage from Shadowfiend and Voidwraith
  hits, 80.8 hits and 623 DPS per fight), Mind Devourer, trinkets, tier sets
  and gear.
- Entropic Rift is a DoT with `overlap = true, max_stacks = 1` so the engine
  runs its `on_expire` (Collapsing Void, Voidwraith); with a shared-timer DoT
  the engine skips `on_expire`.
- No `[gear]` section, no `src/api.rs` registration, no README row.
