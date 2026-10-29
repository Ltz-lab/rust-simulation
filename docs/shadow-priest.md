# Shadow Priest (Voidweaver)

The Midnight Season 2 sample profile (`profiles/MID2_Priest_Shadow.simc`) uses
the Voidweaver hero tree. In this version Shadow Word: Madness (335467) replaces
Devouring Plague, and Mind Spike, Dark Ascension, Mind Sear and Mindbender are
not in the talent set. Mechanics come from DB2 (build 12.1.5.70575), SimC's
`midnight` priest module and default APL, and the committed core observations.
The per-hit residuals are fitted by `tools/calibrate.py`. Cast and hit counts are
not fitted; they come out of the engine's Insanity, cooldown, rift and apparition
mechanics. The full mechanics brief is in
[shadow-priest-brief.md](shadow-priest-brief.md).

## Results

300-second Patchwerk, core kit without consumables, Bloodlust, trinkets or set
bonuses. The reference is SimC's MID2 sample run; azsim runs 10,000 iterations,
seed 1, with `calibration/shadow_voidweaver.toml`.

| | DPS |
|---|---:|
| SimC | 143,444 |
| azsim | 142,586 ± 60 |
| difference | -0.60% |

Command: `./target/release/azsim specs/shadow_voidweaver.toml -n 10000 --seed 1 --calib calibration/shadow_voidweaver.toml`

## Hit counts

Hits per fight, direct hits plus ticks, from `tools/compare.py` against the
reference. Top 12 sources by reference DPS:

| source | SimC hits/fight | azsim hits/fight | DPS diff |
|---|---:|---:|---:|
| shadow_word_madness | 272.8 | 274.5 | +0.5% |
| shadow_word_pain | 362.5 | 361.8 | -0.3% |
| mind_flay | 176.0 | 177.7 | +0.9% |
| void_blast | 28.7 | 27.3 | -5.1% |
| mind_blast | 32.2 | 33.4 | +3.5% |
| shadeburst | 188.2 | 187.0 | -0.7% |
| shadowy_apparition | 137.8 | 137.1 | -0.6% |
| vampiric_touch | 203.4 | 202.5 | -0.5% |
| void_torrent | 57.4 | 57.0 | -0.8% |
| entropic_rift | 191.3 | 195.1 | +1.9% |
| void_apparition | 59.2 | 58.8 | -0.7% |
| thing_from_beyond_void_spike | 174.3 | 174.9 | +0.2% |

Mind Blast and Void Blast share their charges. Together they give 60.7 hits
against SimC's 60.9, but the split differs by 1.4 casts (azsim has more Mind
Blast and fewer Void Blast). Shadow Word: Madness is the Insanity test: the
round 7 log has 46.2 hard casts against SimC's 46.5.

## Held-out stat probes

The same per-hit calibration is used for all probes. Gains are relative to each
simulator's own baseline. `tools/validate_stats.py` on the 10,000-iteration run.

| probe | SimC gain | azsim gain | difference |
|---|---:|---:|---:|
| +500 crit | +10.39% | +10.29% | -0.10 points |
| +500 haste | +9.72% | +9.43% | -0.29 points |
| +500 mastery | +8.26% | +8.53% | +0.27 points |
| +500 versatility | +8.81% | +8.76% | -0.05 points |
| +200 intellect | +6.44% | +6.51% | +0.07 points |

## Mechanics

- **Insanity** (max 100). Mind Blast and Void Blast give 6 and 8, Shadow Word:
  Death 4, Tentacle Slam 6, Void Volley 10. Mind Flay gives 3 per tick, +30% from
  Surge of Insanity (391399), so 7.8 per two-tick cast. Void Torrent gives 6 per
  unhasted tick while channelling. Shadow Word: Madness costs 45. The budget
  matches the spender casts only with Surge of Insanity; without it the sim is
  about 7% short on SW:M casts.
- **Shadow Word: Madness rolling refresh.** SimC refreshes SW:M as a rolling
  periodic: the duration resets to the pandemic-capped value, and the per-tick
  damage becomes the damage left spread over the new tick count. The spec uses
  `refresh = "rolling"` with `rolling_pandemic = true`. Tentacle Slam applies
  SW:M at 25% strength (Maddening Tentacles, 1279353) through `apply_dot` with
  `strength = 0.25`, which extends the running DoT when it is up.
- **Mind Blast and Void Blast routing.** During Entropic Rift or while Void Torrent
  channels, Mind Blast becomes Void Blast (450983). It uses Mind Blast's charges and
  cooldown, and each cast extends the rift by 1 s (Darkening Horizon, at most 3
  times per rift). Shadowy Insight makes the next Mind Blast instant and is spent
  instead of a charge.
- **Apparitions.** Shadowy Apparitions come from every SW:M impact, Tormented
  Spirits rolls on SW:P ticks and idol events. Each one fires Shadeburst and
  extends SW:P and Vampiric Touch by 1 s (Haunting Shadows). Each apparition
  travels 3.33 s. A fraction of them become Void Apparitions, which fire a Void
  Bolt. The Void Apparition share is fitted to observed hits (see gaps).
- **Entropic Rift.** Void Torrent opens an 8 s rift (buff plus hasted timer DoT,
  sp 0.6 per tick from tick 2, 5% miss). Channel end resets the rift and
  Voidheart. Collapsing Void hits when the rift ends for 4.4 × (1 + 0.2 × stacks),
  stacks from hard-cast SW:M during the rift, up to five.
- **Voidform and Void Volley.** Voidform (228260) gives 20% more Shadow damage for
  20 s, removes Shadowform, fires a free Void Volley, and grants three volley
  charges. Ancient Madness adds haste stacks that decay after Voidform.
- **Idols.** Idol of N'Zoth is a stack counter on the target: Horrific Vision at
  50 stacks, Vision of N'Zoth at 100. Idol of Yogg-Saron stacks per apparition
  and spawns a Thing from Beyond at 20. Tentacle Slam draws a random idol from the
  Void Apparitions deck.
- **Pets.** Thing from Beyond (20 s) casts Void Spike. Voidwraith (5 s) is summoned
  at each rift end and casts Void Flay. Shadowfiend (5 s) comes from Shadow Word:
  Death below 20% or the Y'Shaarj idol, and melees for 0.408 × owner SP.
  Void Tendril (10 s) is the Idol of C'Thun result and casts Mind Flay.
- **Screams of the Void.** Shadow Word: Madness applies a buff that speeds
  Shadow Word: Pain and Vampiric Touch ticks by 80% on both DoTs (two ranks of
  40%), with a spec aura that weakens Quickened Pulse.
- **Shadow Weaving (mastery).** Each active DoT (SW:P, VT, SW:M) adds 0.5% per
  mastery point. Voidform counts as three DoTs. The sources use `mastery_scale = 3`.

## Gear

`[gear]` and `[gear.kit]` are in the spec, so `--gear profiles/MID2_Priest_Shadow.simc`
works. Gearless runs (calibration, reference, the table above) never read them, so
the core-kit DPS doesn't depend on them. SimC's gear report
(`tools/check_gear.py`) has not been run; instead the profile's gear without
trinkets and with the kit off was compared with the reference's buffed snapshot
(which is core kit without trinkets). The crit, haste, mastery and versatility
ratings (364 / 1,123 / 1,424 / 123) match exactly, and so do the final crit
(12.91%: 7.91% from rating plus the 5% base spell crit), haste (29.29%: rating
haste x1.03) and mastery and versatility. Spell power is 3,361 against 3,374
(-0.4%), so `--gear` on that gear gives 142,099 against 142,649 gearless
(`-n 2000 --seed 1`). The full profile adds both trinkets' stats and, with the
kit, food, flask, rune, oil and potion, so its stats are higher than the
snapshot's by design. Neither trinket's effect nor the 2/4-piece set bonus is
in the effect library yet (`--gear-stats` lists them as `unmodelled`).

## Known gaps

Approximations marked `APPROX` in `specs/shadow_voidweaver.toml`:

- **Void Apparition share.** azsim uses 30% of apparitions, fitted to the 59.2
  observed hits. DB2 and SimC give 15%, which does not explain the count.
- **Void Torrent tail.** The reference averages 5.63 ticks per cast. DB2 gives
  the channel a 1 s tick with no tick-time modifier, so the 0.64 s tick is not
  from data; the 63% tail-tick chance is fitted to the hit count.
- **Shadowy Insight.** The proc is a fixed 8.73% roll while the buff is down, not
  SimC's threshold RNG. It gives 28.4 starts per fight, matching the reference.
- **Ancient Madness extension.** SimC's 10% per extension stack over 5 is modelled
  as 10 stacks of 0.9%, with 4.45 average extensions (13.0 refreshes against 2.9
  starts observed).
- **GCD lag.** SimC's 5 ms queue lag is written as a fixed `gcd_pct = -0.45`, so
  it is a percentage of the GCD rather than a time.
- **Pets.** Thing from Beyond's Void Spike is modelled at 2.0 s hasted, which gives
  150 spikes, against the reference's 174.3. The reference implies about 1.4 s at
  this haste. Subservient Shadows' +20% pet duration is not applied, as in SimC.
  Shadowfiend melee uses `sp_coeff = 0.408` where SimC uses an attack-power path,
  and Shadow Weaving on pet hits is applied without crit, as SimC does.
- **Void Tendril.** The reference has no row for it, so azsim's 641 DPS (about
  0.45% of the reference total) has nothing to compare against. The mechanics
  brief estimated the gap at about 0.8% from the source share sum; the measured
  row gives the figure above, and the report's 0.36% figure does not match it.
- **Reference gap cause.** `tools/observe.py` keys sources by bare name, so the
  Void Tendril's `mind_flay` row collides with the player's and is dropped.
  Fixing the observer and regenerating the reference removes this gap.
- **Gear constants** are not checked against SimC's gear report. Gear spell
  power is 13 short of the snapshot (3,361 vs 3,374, see Gear above): the
  Void Elf base intellect (611, carried from Arcane) or an item's intellect is
  off, which needs `tools/check_gear.py` with a SimC build to settle.

Mechanics not taken: Mind Devourer, Thought Harvester, Shattered Psyche, Dark
Thoughts, Void Empowerment, Mind Flay: Insanity (Archon), and the tier set's 4pc
chance per Tentacle Slam.

## Workflow trial

The spec was built with a tiered multi-model workflow; the timing and round log
are below.


## Workflow trial

This spec was built as a trial of a tiered multi-model workflow. An Opus
coordinator delegated each stage to a subagent: Opus for the mechanics
brief, the engine work and escalations; Sonnet for writing and
calibrating the spec; Haiku for prep, triage and wrap-up. Every subagent
ran at medium effort.

| stage | model | wall time | tokens (subagent context at hand-back) |
|---|---|---:|---:|
| 1. Prep: build, DB2, spell dump, baselines | Haiku | 5.1 min | 150k |
| 2. Mechanics brief | Opus | 14.4 min | 315k |
| 3. Engine: tick-rate buffs, rolling DoTs with pandemic/strength, `apply_dot`, `full_recharge_time` | Opus | 5.0 min | 144k |
| 4. Write spec (hit counts within 10%) | Sonnet | 16.3 min | 318k |
| 5. Calibrate, 6 rounds | Sonnet | 23.6 min | 78k–226k per round, ~1.0M total |
| 5a. Triage, 5 runs | Haiku | 1.7 min | ~61k each, ~307k total |
| 6. Escalation: missing Insanity | Opus | 24.2 min | 373k |
| 7. Wrap-up | Haiku | WRAPUP_TIME | WRAPUP_TOKENS |

Token counts are what the harness reported for each subagent when it
handed back. They show context size, not billed tokens, so cached
re-reads are not separated out. Per model, excluding wrap-up:
- Haiku: about 7 minutes and 0.46M tokens.
- Sonnet: about 40 minutes and 1.3M tokens.
- Opus: about 44 minutes and 0.83M tokens, plus the coordinator's own small context.

The calibration loop ran round by round, starting at -2.23% DPS:

| round | DPS vs SimC | change |
|---|---:|---|
| 1 | -2.23% | first calibration |
| 2 | -1.50% | DB2 tick times for the DoTs |
| 3 | -2.32% | crit multiplier 2.067; crit probe fixed (-1.58 → -0.09) |
| 4 | -2.06% | rift reset at the true end of the Void Torrent channel |
| 5 | -1.64% | Shadowy Insight no longer refreshes while up |
| 6 | -1.40% | Void Torrent's 4th Insanity tick restored |
| 7 | -0.60% | Opus escalation |

There was one escalation, at round 6 rather than the planned 8. The
remaining gap was a single Insanity shortfall: SW:M was cast 42.3 times
against 46.5. Both Sonnet and the brief had missed it, and no single
source was over the 3%/3% threshold. Opus found it: Surge of Insanity
raises Mind Flay's Insanity per tick by 30%. It also replaced two of
Sonnet's three fitted fudges with real mechanics:
- The Screams tick rate was fitted at 63%/85%. Opus replaced it with
  80% on both DoTs, plus a spec aura that weakens Quickened Pulse.
- The 3% GCD lag was replaced by SimC's channel lag and queue lag.

What we'd change:
- **Brief.** The brief was the highest-leverage stage. Its one missing
  Insanity talent cost five Sonnet rounds. Have the brief check its
  resource budget, so that generation per fight matches the spender
  casts in the reference, before handing over.
- **Sonnet's fitting.** Sonnet tended to fit hit counts with free
  parameters: tick rates, proc chances, a tail-tick chance. The fits
  looked right but hid real mechanics. A rule worth enforcing in
  review: any non-DB2 timing value must name its SimC source or go to
  Opus.
- **Haiku triage.** Haiku triage was cheap and accurate but added
  little. Sonnet's own round summaries were already short, so triage
  could be dropped or merged into the calibrator's compare step.
- **Escalation trigger.** Escalate on a stalled root cause (the same
  mechanic unexplained for two rounds), not on a per-source threshold
  after 8 rounds.
