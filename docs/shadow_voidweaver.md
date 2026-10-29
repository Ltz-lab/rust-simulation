# Shadow Priest (Voidweaver) — draft

`specs/shadow_voidweaver.toml`, reference `reference/shadow_*` (from
`profiles/MID2/MID2_Priest_Shadow.simc`, 300s Patchwerk, core kit), talents
`CIQAAAAAAAAAAAAAAAAAAAAAAMMjZGAAAAAAAAAAAAMLmxMbzMMz2MzYG2mZGzMzYDZGLmpBYmZGAIAz2stEMbMAwgxMzMmtxMYmBzgB`
(spec 258, hero tree Voidweaver).

**DRAFT.** The spec runs and covers the rotation's damage sources, but hit
counts are ~12% short of the reference and a few mechanics are approximated
(see gaps). It is not registered in `src/api.rs` and has no `[gear]` lines.

## Results

| run | DPS | reference |
|---|---:|---:|
| core kit, `-n 500`, no calibration | 50,486 | — |
| core kit, `-n 500`, `--calib calibration/shadow_voidweaver.toml` | 126,554 | 143,444 (−12%) |

Per-hit damage is calibrated (`tools/calibrate.py --derived
derived/shadow_voidweaver.toml`): every modelled source matches its observed
per-hit damage within 0.04%. The −12% is hit counts (Shadow Word: Madness
46.0 casts vs 67.9, and the apparition/shadeburst chain that follows it).

Stat probes (`tools/validate_stats.py`, same calibration):

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| +500 crit | +10.39% | +9.24% | −1.15 |
| +500 haste | +9.72% | +9.21% | −0.51 |
| +500 mastery | +8.26% | +7.66% | −0.60 |
| +500 versatility | +8.81% | +8.80% | −0.01 |
| +200 intellect | +6.44% | +6.58% | +0.14 |

Stat priority order matches.

## Modelled

- **Resource**: Insanity (0/100), per-source gains taken from a logged SimC
  fight (Mind Blast 6, Void Blast 8, Mind Flay 3.9/tick, Void Torrent 6/tick,
  Void Volley 10, Tentacle Slam 6, Shadow Word: Pain 3, Shadow Word: Death 4,
  Vampiric Touch 4, Auspicious Spirits 1 per apparition). Shadow Word:
  Madness costs 45 (50 − 5 Mind's Eye).
- **DoTs**: Shadow Word: Pain (direct + DoT, taps immediately, refresh
  extends), Vampiric Touch, Shadow Word: Madness (rolling refresh), Mind Flay
  (2-tick channel, as the APL interrupts it), Void Torrent (channel with tick
  zero), Entropic Rift.
- **Cooldowns**: Voidform (120s, +20% damage, 3 Void Volley charges, free
  volley on entry, extended 1.5s per Shadow Word: Madness, Crushing Void
  charge on expiry), Power Infusion (120s, +20% haste), Void Torrent (30s),
  Tentacle Slam (15s recharge, 2 charges), Shadow Word: Death (10s), Mind
  Blast / Void Blast (9s hasted, separate cooldowns), Void Volley (charges).
- **Procs**: Shadowy Apparitions (from Shadow Word: Madness casts and from
  Shadow Word: Pain ticks via Tormented Spirits), Void Apparitions (40% of
  spawns, also fire a Void Bolt), Shadeburst on every apparition, Haunting
  Shadows (apparitions extend both DoTs by 1s), Screams of the Void buff,
  Collapsing Void on rift expiry, rune of the unleashed fire (10.04/min).
- **Pets**: Thing from Beyond (Idol of Yogg-Saron, ~1.81 spawns/min, Void
  Spike), Shadowfiend (40% on an execute-range Shadow Word: Death),
  Voidwraith (on Entropic Rift expiry).
- **Mastery**: every Shadow source gets `(1 + mastery)^2`
  (`mastery_power = 2`) — the shadow-weaving multiplier and the spec mastery
  aura. The +500 mastery probe moves every source by the same 8.2%.
- Derived modifiers from `tools/effects.py` (spec aura 137033, talents), see
  `derived/shadow_voidweaver.toml`.

## Known gaps

- **Insanity generation is ~2,080 per fight vs SimC's ~3,000**, mostly the
  Idol of C'Thun void tendrils (their Mind Flays generate ~59) and the
  interactions around apparitions. This is why Shadow Word: Madness is cast
  46.0× instead of 67.9×, and everything downstream (apparitions 104 vs 138,
  void apparitions 41 vs 59, shadeburst 145 vs 188) is low. The rotation is
  also GCD-bound: it does not fit the reference's cast counts for Mind Blast
  (22.5 vs 32.2) and Void Blast (21.5 vs 28.7) even though both cooldowns are
  idle ~half the fight.
- **Idol of N'Zoth** is fired as two fixed-rate procs (horrific_vision
  4.8/min, vision_of_nzoth 3.96/min) rather than by the 50/100-stack debuff.
- **Effective DoT cadence**: Shadow Word: Pain and Vampiric Touch use
  `tick_time` 1.1s / 1.9s (vs 2s / 3s in the game data) so their tick counts
  match the reference. SimC's Screams of the Void (+40% tick rate while up)
  and "periodic refresh extends duration" produce a faster cadence than the
  base data, and the engine has no DoT tick-speed hook.
- **Not modelled**: Maddening Tentacles' weaker Shadow Word: Madness
  application, Psychic Link (multi-target only), Insidious Ire (+20% while
  both DoTs are up), Voidheart, Shadowform (+10%), ancient madness (haste on
  Voidform expiry), Mind Devourer / Shadowy Insight (not taken or not
  modelled), Shadow Weaving (the mastery bonus damage from Shadowfiend and
  Voidwraith hits, 80.8 hits per fight), trinkets, tier sets and gear.
- No `[gear]` section, no `src/api.rs` registration, no README row.
