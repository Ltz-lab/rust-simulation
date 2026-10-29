# Shadow Priest (Voidweaver): mechanics brief

Target: `specs/shadow_voidweaver.toml`, reference `reference/shadow_core_observations.json`
(SimC MID2 sample profile, core kit, 300 s Patchwerk, **143,444 DPS**). Tags: `[db2]` =
build 12.1.5.70575 via `tools/spelldb.py target/db2 <id>`; `[simc]` = simc `midnight`
branch (`engine/class_modules/priest/sc_priest*.cpp`, `ActionPriorityLists/default/priest_shadow.simc`;
`midnight` and `master` were byte-identical on 2026-10-29); `[obs]` = reference JSON.

This is the brief the spec was written from, kept as a record. Two things changed
when it landed: the "rollover" refresh of §4.3 is the engine's `refresh = "rolling"`
with `rolling_pandemic = true` (see `docs/spec-format.md`), and the prep files it
cites (`prep/`) are not committed.

## 0. Corrections to the prep pass (`prep/shadow.json`)

- **Devouring Plague doesn't exist in Midnight: it's now Shadow Word: Madness** (335467,
  talent "Shadow Word: Madness"). It is the top source (17.1%). Ignore 369128/138490.
- **Mind Spike, Dark Ascension, Mind Sear and Mindbender** are absent: SimC's Midnight
  module has no code for them and the talent string doesn't take them. Correct to omit them.
- **Mind Flay: Insanity (391403)** is an Archon talent. Not taken, not in the reference. Omit it.
- **Insanity gains.** The `x100` scaling is right: DB2 stores Insanity ×100 (SW:P `300` = 3).
  Mind Blast and SW:D energize bases are 0 in their own rows. The gain comes from the
  spec aura **137033**: #5 is `+600` on EFFECT_2 of Mind Blast/Void Blast (mask 8192), so
  they give **6** each. #6 is `+400` (op 33) on SW:D, so SW:D gives **4**. Void Infusion
  450612 #0 adds `+200` to Void Blast, so Void Blast gives **8** [db2].
- **Pets.** The Shadowfiend summon is **1280172** (5 s). Its melee is SimC's `fiend_melee_t`:
  1.5 s base swing, hasted, power 0.408 × owner SP, 1 Insanity per hit from Power Leech
  262485 [simc, db2]. **Voidwraith** is summon **451235** (5 s) and casts **Void Flay
  451435** (sp 1.0, 1.5 s, pet power 0.5 × owner SP). Thing from Beyond 373277 (20 s)
  casts Void Spike 373279. Details are in §2.9.
- **Void Bolt / Void Apparition** come from the apex talent Void Apparitions (1264096 /
  1264104 / 1264107): a Shadowy Apparition can become a Void Apparition, which also fires a
  Void Bolt. They aren't separate casts.
- **Mastery: Shadow Weaving** (343690, sp field 0.5) is 0.5% per mastery point **per active
  DoT** (SW:P, VT, SW:M). The reference `mastery_value` 0.2043 is that per-DoT value [obs].
- **The reference misses ~0.8% of DPS.** Source shares sum to 99.22%. The missing part is the
  Void Tendril (Idol of C'Thun, spawned by Void Apparitions' random idol, 7/17). SimC
  reports its `mind_flay` as a child of `idol_of_cthun`, and `tools/observe.py` keys sources
  by bare name, so the pet's row collides with the player's `mind_flay` and is lost. The
  `mind_flay` row (id 15407, 88 casts, 2 ticks each) is the player's.

## 1. Damage sources (reference)

Shares are of total DPS. "Casts" is SimC `executes` (for triggered spells, the trigger count).

| source | spell id | share | casts | hits (dir / tick) | model in azsim |
|---|---:|---:|---:|---|---|
| shadow_word_madness | 335467 | 17.14% | 67.9 (46.5 cast + 21.4 Tentacle Slam) | 67.9 / 204.9 | spender: direct sp 2.61625 + DoT 2.61625/2 s hasted, 6 s; refresh, see §2.4 |
| shadow_word_pain | 589 | 10.03% | 6.9 | 6.9 / 355.6 | DoT 0.1246/2 s hasted, 16 s; kept up by Haunting Shadows; ticks sped up by Screams of the Void (§2.6) |
| mind_flay | 15407 | 8.52% | 88.2 | 0 / 176.0 | channel clipped at **2 ticks**, 1.38138/0.75 s hasted |
| void_blast | 450983 | 8.34% | 28.7 | 28.7 / 0 | Mind Blast during Entropic Rift, sp 2.1, shares MB charges (§2.3) |
| mind_blast | 8092 | 7.24% | 32.2 | 32.2 / 0 | sp 0.78336, 9 s hasted recharge, 1 charge |
| shadeburst | 1231479 | 5.38% | 188.2 | 188.2 | sp 1.33 on every apparition impact (§2.5) |
| shadowy_apparition | 413231 | 4.76% | 137.8 | 137.8 | sp 0.8505, travel 3.33 s (§2.5) |
| vampiric_touch | 34914 | 4.39% | 21.4 (all from Tentacle Slam) | 0 / 203.4 | DoT 0.6667/3 s hasted, 21 s; SoV speedup (§2.6) |
| void_torrent | 263165 | 4.38% | 10.2 | 0 / 57.4 | channel, 2.37685/tick, ~5.6 ticks per cast (§2.7) |
| entropic_rift | 447445 (dmg 447448) | 4.15% | 10.2 | 191.3 | timer DoT, sp 0.6 per hasted 1 s tick from tick 2 (§2.7) |
| void_apparition | 1264176 | 3.98% | 59.2 | 59.2 | 30% of apparitions [obs], sp 0.8505 × 1.15 |
| thing_from_beyond_void_spike | 373279 | 3.86% | 174.8 | 174.3 | pet, sp 0.938961, 2 s cast (§2.9) |
| void_volley_damage | 1242189 | 3.43% | 11.3 casts | 112.5 | 10 bolts × sp 0.9544 per cast (§2.2) |
| vision_of_nzoth | 1243106 | 2.93% | 19.8 | 19.8 | Idol of N'Zoth at 100 stacks, sp 6.89 (§2.8) |
| void_bolt | 1264177 | 1.91% | 59.8 | 59.5 | one per Void Apparition, sp 1.5, 25 yd/s |
| collapsing_void | 448405 | 1.89% | 10.2 | 9.8 | rift collapse, sp 4.4 × (1 + 0.2 × stacks) |
| tentacle_slam_damage | 1227621 | 1.81% | 21.4 | 21.4 | sp 0.687926, 0.5 s travel |
| shadow_word_death | 32379 | 1.24% | 5.1 | 5.1 | execute only (<20%), sp 0.85 (§2.10) |
| horrific_vision | 1243105 | 0.93% | 24.0 | 24.0 | Idol of N'Zoth at 50 stacks, sp 1.82 |
| void_volley_voidform_damage | 1242189 | 0.88% | 2.95 casts | 29.4 | free volley when Voidform starts |
| rune_of_unleashed_fire | 1286970 | 0.72% | 50.2 | 50.2 | gear; copy from destruction spec |
| voidwraith_void_flay | 451435 | 0.50% | 50.0 | 50.0 | pet after each rift (§2.9) |
| shadow_weaving | 346111 | 0.43% | 80.8 | 80.8 | mastery copy of fiend/wraith hits; may be approximated |
| rune_of_unleashed_fire_lingering | 1287663 | 0.22% | 50.2 | 0 / 240 | gear; copy from destruction spec |
| shadowfiend_melee | — | 0.16% | 30.9 | 30.9 | pet (§2.9) |
| *(void_tendril mind_flay)* | 193473 | ~0.8% | ? | ? | missing from the reference (§0) |

Buff uptimes worth matching [obs]: voidform 26.0% (2.95 × 26.4 s), shadowform 74.0%,
power_infusion 14.5%, entropic_rift 44.6% (10.2 × 13.1 s), voidheart 70.3%,
screams_of_the_void 67.5%, insidious_ire 98.2%, shadowy_insight 28.4 procs,
thing_from_beyond 64.7%, twist_of_fate 34.9%, entropic_embrace 26.0%.
Reference stats: SP 3374, crit 12.91%, haste 29.29%, mastery_value 20.43% (per DoT), vers 5.28%.

## 2. Mechanics

### 2.1 Insanity

- Resource `insanity`: max **100** [simc default], start 0 [simc]. No passive regen or drain.
  Voidform has no drain in Midnight. Its 1 s periodic (194249 #3) is a dummy that SimC skips.
- Spender: **Shadow Word: Madness costs 45**: 50 from DB2 (5000 /100), −5 from Mind's Eye
  407470 [db2].
- Gains [db2 unless noted]:

| source | Insanity |
|---|---|
| Mind Blast | 6 (spec aura 137033 #5) |
| Void Blast | 8 (6 + Void Infusion 450612) |
| Mind Flay tick | **3.9** per tick: 3 (15407 #2) +30% from Surge of Insanity (391399 #1, P_EFFECT_3 on Mind Flay; SimC applies it via `parse_all_passive_talents` although the talent is marked NYI), so 7.8 per 2-tick cast |
| SW:P (hard cast only) | 3 |
| Vampiric Touch hard cast | 4; **the VT from Tentacle Slam gives 0** [simc `insanity=false`] |
| Tentacle Slam | 6 (1227280 #1) |
| Void Torrent | 6 per 0.975 s unhasted tick, one at t=0, while the channel lasts (289577) [simc] |
| Void Volley (cast and the free Voidform one) | 10 (1242173 #1) |
| Shadow Word: Death | 4 (137033 #6) |
| Shadowy Apparition hit | 1 (Auspicious Spirits 155271 #1 flat +100); SimC threshold RNG averages **0.8 per apparition** at one target [simc] |
| Shadowfiend melee / Void Flay | 1 per hit (Power Leech 262485) [simc] |
| Horrific Vision | 0.5 per 0.75 s for 3.05 s per proc, 4 ticks = 2 [db2 1243113] |
| Vision of N'Zoth | 0.5 per 0.25 s for 3.05 s, 12 ticks = 6 [db2 1243114] |
| Void Tendril mind flay tick | 1 (377358) |

With Surge of Insanity the budget is ~2.11k generated against 46.5 × 45 ≈ 2.09k spent
(without it, ~1.96k: the 7% shortfall of rounds 1–6).

### 2.2 Voidform (talent 228260, buff 194249) and Void Volley

- **Voidform** spell 228260: 1.5 s cast (hasted), 120 s cooldown [db2]. Usable only when
  Voidform, Crushing Void and Void Volley charges are all down [simc `voidform_t::ready`].
  On cast [simc]:
  1. buff `voidform`, 20 s;
  2. remove `shadowform` (reapply it when Voidform ends);
  3. fire a free **void_volley_voidform** (10 bolts);
  4. give **3 Void Volley charges** (buff 1242171, 30 s; 228260 #2 base 3).
- Voidform buff, 194249 [db2]: +20% direct and +20% periodic damage on Shadow spells (mask
  0/0/256, the same set Shadowform's +10% covers). Its own haste field is 0. **Shadow Weaving
  counts as 3 DoTs** while it's up [simc]. Shadowform (232698) is +10% / +10% while Voidform
  is down. Put both buffs on the same ability list and let `tools/effects.py` derive which
  spells they touch.
- **Ancient Madness** (1231346) [simc, db2 194249 #11-#13]:
  - Each *hard-cast* SW:M during Voidform extends Voidform by **1.5 s**, at most 5 times.
  - Each of those casts also adds a stack of `ancient_madness_extension`: +2% haste per
    stack, max 5, lost when Voidform ends.
  - When Voidform ends, buff `ancient_madness` (1304485) gives 10% × stacks/5 haste,
    decaying linearly to 0 over 10 s (1 s steps).
  - Model the decay with 10 stacks of 1% haste. A 1 s unhasted timer DoT (10 s,
    `whole_ticks`) removes one stack per tick (`buff set = "buff.ancient_madness.stack-1"`).
  - Fires from the Voidform buff's `on_expire`. Voidform always runs out on its own.
- **Void Volley** (missile 1242173, damage 1242189):
  - Usable only with a `void_volley` or `crushing_void` stack. Spends one stack.
  - Instant, on the GCD, 10 Insanity.
  - **10 bolts** of sp 0.9544, 50 ms apart, travel 40 yd ÷ 50 yd/s = 0.8 s
    [simc distance 40, see §2.5]. Model as `trigger` with `count = "10"`, `interval = 0.05`,
    `delay = 0.8`.
  - Free Voidform volley: same 10 bolts, reported as `void_volley_voidform_damage`, and it
    also gives 10 Insanity.
- **Crushing Void** (1279354): when Voidform ends, reset Void Volley and add 1 stack of
  `crushing_void` (1279437, 30 s). That stack is a castable volley that also blocks Voidform
  [simc]. It is cancelled at pull. Observed 2.72 starts, 1.9 s average, so it's spent at once.
  Casts: 2.95 × 3 + 2.72 ≈ 11.3 [obs ✓].
- Cast-count anchors: voidform 2.95, void_volley 11.3, ancient_madness_extension 13 refreshes [obs].

### 2.3 Mind Blast, Void Blast, Shadowy Insight

- Mind Blast 8092: 1.5 s cast hasted, **1 charge**, 9 s hasted recharge, sp 0.78336, 6 Insanity.
  Overwhelming Shadows (1266883 #1) gives +100% direct to MB/VB [db2]; leave it to `effects.py`.
- **Void Blast** 450983 replaces Mind Blast while `entropic_rift` is up or Void Torrent is
  channeling. It uses **Mind Blast's cooldown and charges** [simc `mind_blast_base_t` sets
  `cooldown = p.cooldowns.mind_blast`]. sp 2.1, 1.5 s cast, 8 Insanity.
  - Each cast during the rift (**Darkening Horizon** 449912) extends the rift, Voidheart and
    the Darkening Horizon counter by **1 s**, at most 3 times per rift [db2 #1, #2].
  - No engine gap: model **one** player ability `mind_blast` with the cast time and charges
    and no damage. Its `on_start` is `if buff.entropic_rift.up then trigger void_blast_hit
    (+8 Insanity, Darkening Horizon) else trigger mind_blast_hit (+6)`. The two `*_hit`
    abilities carry the damage and the source names.
- **Shadowy Insight** (375888, buff 375981, 10 s):
  - Trigger: each SW:P tick with damage. SimC's threshold RNG averages **8%** [simc
    `init_rng_shadow`]. That gives 28.4 procs from 355.6 ticks [obs ✓].
  - While up, Mind Blast/Void Blast is instant and the buff is spent **instead of a charge**
    [simc `update_ready`].
  - Model: a second ability `mind_blast_insight` with `requires_buff = "shadowy_insight"`,
    no cooldown, `on_start` consume, the same routing, placed just above `mind_blast` in the
    rotation.
- Not taken: Mind Devourer, Thought Harvester, Shattered Psyche, Dark Thoughts, Void Empowerment.

### 2.4 Shadow Word: Madness (the old Devouring Plague)

- 335467: 45 Insanity, instant, direct sp 2.61625 plus a DoT of sp 2.61625 per 2 s hasted
  tick for 6 s. Both can crit [db2, simc].
- On cast [simc `shadow_word_madness_t`]:
  - a Shadowy Apparition on impact (§2.5);
  - `screams_of_the_void` buff (§2.6);
  - if Voidform is up, the Ancient Madness extension (§2.2);
  - if the rift is up, +1 `collapsing_void` stack (§2.7);
  - 12 Idol of N'Zoth stacks (§2.8).
  - Tormenting Whispers is +15% to it [db2]. Leave that to `effects.py`.
- **Refresh in SimC is "rollover"** [simc `trigger_dot`]. The duration refreshes with the
  normal pandemic cap, but the per-tick multiplier becomes `(old_ticks_left × old_mult +
  new_base_ticks × new_mult) / new_ticks_left`. All remaining damage is kept and squeezed into
  the refreshed ticks.
- **Maddening Tentacles** (1279353): each Tentacle Slam applies SW:M at **25% strength**.
  This means a 25% direct hit plus a 25%-strength DoT merged into the running one. It costs
  nothing and has no Ancient Madness, Collapsing Void or cost effects. It still spawns an
  apparition and gives Screams of the Void and N'Zoth stacks. It is the 21.4 extra executes;
  direct average 17,535 = 0.76 × full [obs ✓].
- SW:M is up ~100% of the fight. 204.9 ticks ≈ 300 s / (2 s / ~1.36 average haste) [obs].
- **Approximation, no engine change**:
  - `refresh = "extend"` keeps every applied tick's damage, like SimC. Because the DoT never
    drops, tick cadence and count are unchanged. Per-tick damage differs by ~3%, which
    calibration absorbs.
  - Tentacle SW:M: a separate triggered ability `shadow_word_madness_tentacle` (`source =
    "shadow_word_madness"`, `damage.mult = 0.25`). Its `on_cast` is `if
    dot.shadow_word_madness.ticking then extend_dot 1.5 s (25% of 6 s) else trigger the full
    apply`. The else branch overcounts a little and is rare after the opener.
  - If SW:M tick counts or the rotation (the `remains<=gcd` re-cast condition) drift, use the
    rollover gap (§4.3).

### 2.5 Shadowy Apparitions, Void Apparitions, Shadeburst

- **Triggers** [simc `trigger_shadowy_apparitions`]: one apparition per target with VT, so 1.
  - Every SW:M impact, cast or tentacle: 67.9.
  - **Tormented Spirits** (391284) on SW:P ticks and impacts: threshold RNG mean **13%**,
    doubled on a crit. About 362 × 0.13 × 1.347 ≈ 63.
  - **Void Apparitions** (1264096) on each idol event: Horrific Vision 24, Vision of N'Zoth
    19.8, Thing from Beyond spawn ~9–11, Shadowfiend from the Y'Shaarj idol ~3.8, Void
    Tendril ~8.8.
  - Total ≈ 197 = 137.8 SA + 59.2 VA [obs ✓].
- Each trigger event adds an **Idol of Yogg-Saron** stack (373276, max 20). At 20 the stacks
  clear and a **Thing from Beyond** spawns, which itself triggers an apparition (Void
  Apparitions). 9.85 idol cycles [obs].
- Each apparition is a **Void Apparition** with probability **0.30** [obs: 59.2/197].
  DB2 1264104 #1 says 15% and SimC rolls `effectN(2)`, so this is unexplained. Use 0.30 and
  note it. A Void Apparition deals sp 0.8505 × **1.15** (1264104 #1) and fires a **Void Bolt**
  (sp 1.5, 40/25 = 1.6 s travel) at its start [simc].
- **Travel**: 12 yd/s over 40 yd = **3.33 s** [db2 speed]. The 40 yd is from simc
  `init_base_stats`: it sets `max(distance, 40)`.
- **On impact** [simc]:
  - damage sp 0.8505. Auspicious Spirits +15%, Haunting Shadows +10%, Subservient Shadows
    +15% (`effects.py`). Spectral Horrors 407469 is +30% while SW:M ticks (≈ always).
  - **Haunting Shadows** (1279355 #1): **+1 s** to SW:P and VT (`extend_dot`). This is why
    SW:P needs only 6.9 casts in 300 s.
  - **Shadeburst** (73510, 1231479): sp 1.33 on the parent target, every apparition.
  - Auspicious Spirits: 1 Insanity, 0.8 average (§2.1).
- Model: SW:M and the SW:P tick roll trigger a helper `apparition_spawn`. Its `on_cast` adds
  the Yogg stack check, then `trigger void_apparition_start` with chance 0.30, else
  `shadowy_apparition_start`. Each `*_start` triggers its damage ability with delay 3.33
  (Void: plus `void_bolt` with delay 1.6). Damage abilities `on_cast`: Shadeburst,
  `extend_dot` ×2, insanity `gain` with chance 0.8.
- Threshold RNG (SimC's lower-variance accumulator) can be plain `chance`. Or copy arcane's
  `clearcasting_attempts` accumulator pattern if variance matters.

### 2.6 Shadow Word: Pain, Vampiric Touch, Tentacle Slam, Screams of the Void, Insidious Ire

- **SW:P** 589:
  - direct sp 0.168, DoT 0.1246 / 2 s hasted, 16 s, pandemic [db2].
  - Invoked Nightmare +150%, Inner Quietus +20% (`effects.py`).
  - Hard cast: 3 Insanity and 3 N'Zoth stacks.
  - Each tick rolls Shadowy Insight (8%) and Tormented Spirits (13%, ×2 on crit). Its ticks
    give no N'Zoth stacks [simc].
- **VT** 34914:
  - DoT 0.6667 / 3 s hasted, 21 s. The direct part (3.04) is disabled [simc].
  - Applied only by Tentacle Slam in practice: 21.4 of 21.4. Gives 4 N'Zoth stacks
    (`casted=true`) but no Insanity.
- **Tentacle Slam** 1227280: 2 charges, **15 s unhasted** recharge (21.4 casts ≈ 300/15 + 2)
  [db2, obs]. Instant, 6 Insanity, **0.5 s travel** [simc]. On impact, in order:
  1. apply VT;
  2. damage 1227621, sp 0.687926;
  3. SW:M at 25% (§2.4);
  4. **random idol** (Void Apparitions 1264107 #0, 100%): Y'Shaarj 3/17, Horrific Vision
     4/17, Vision of N'Zoth 1/17, Thing from Beyond 2/17, C'Thun 7/17. SimC deals these from
     a shuffled deck without replacement; independent chances are a fine approximation.
     The idols are in §2.8–2.9.
  5. 6 N'Zoth stacks.
- **Screams of the Void** (375767, buff 393919):
  - Each SW:M application (cast or tentacle) gives a 3 s buff that **extends** on refresh.
  - While it's up, **SW:P and VT tick 40% faster per rank**: 393919 #0 is a tick-time modifier
    (base 0) set to −28.6 (= 1/1.4) by the talent (375767 #1, a flat label modifier, aura 219).
    The node is taken at **2/2**, and +80% matches both SW:P and VT tick counts [obs].
  - SW:P's base tick is **1.74 s**: Quickened Pulse #0 is −20% but the spec aura 137033 #24
    adds +7 to it (−13%). The same −13% applies to the Entropic Rift driver (0.87 s hasted).
  - Uptime 67.5% [obs]. Without it SW:P comes out ~20% short on ticks. Engine gap §4.1.
- **Insidious Ire** (373212):
  - +20% to Mind Blast, Void Blast, Mind Flay, Void Torrent and Void Volley while SW:P, VT
    and SW:M all tick.
  - The buff lasts the shortest remaining of the three [simc]. Uptime **98.2%** [obs].
  - Approximate: always on, folded into derived/calibration multipliers.

### 2.7 Voidweaver: Void Torrent, Entropic Rift, Collapsing Void, Voidheart, Voidwraith

- **Void Torrent** 263165:
  - 30 s cooldown, off the GCD (`use_off_gcd`), channelled 3 s hasted, `tick_zero`.
  - sp 2.37685 per tick, base tick 1 s, Quickened Pulse ×0.8 ×0.8 (two −20% tick-time
    effects) = **0.64 s**.
  - Observed **5.63 ticks per cast** [obs 57.4 / 10.2]. Try `channel.ticks = 5` with
    `tick_on_apply` (6 per cast) against 4 + 1 (5) and keep whichever lands near 5.6. The
    fight-end cut explains part of the gap. **Uncertain.**
  - Gives 2 N'Zoth stacks per tick. Insanity as in §2.1, via a timer DoT cancelled in
    `channel.on_finish`.
  - Order [simc]: spawn the Entropic Rift **before** the channel starts.
  - At the channel's end: reset the rift's remaining time to its full 8 s, and Voidheart's
    to its full duration.
- **Entropic Rift** (driver 459314: **8 s**, 1 s periodic, hasted; damage 447448: sp **0.6**):
  - Damage on every tick **from tick 2 on**: the first tick never hits [simc].
  - Each damage tick **misses 5%** [simc option `entropic_rift_miss_percent` 0.05].
  - Multiplier ×(1 + 0.20 × `collapsing_void` stacks).
  - Average life 13.1 s (8 s + ~2.3 s channel + up to 3 s Darkening Horizon). 191.3 hits in
    10.2 rifts [obs ✓].
- Model the rift as a **buff** `entropic_rift`, for `on_expire` and `buff.entropic_rift.up`,
  plus a **timer DoT** `entropic_rift_ticks`, for hasted ticks, kept in sync:
  - same duration;
  - `extend_buff` / `extend_dot` together;
  - re-apply both at the end of Void Torrent: buff refresh, DoT `refresh = "reset"` keeps the
    cadence.
  - DoT `on_tick`: `if buff.rift_armed.up then trigger entropic_rift_hit (chance 0.95) else
    buff rift_armed`.
- **Collapsing Void** (448403):
  - Each **hard-cast** SW:M while the rift is up adds a `collapsing_void` stack. Max **5** for
    Shadow, **+20%** per stack (talent #2) [simc].
  - When the rift ends: damage 448405, sp **4.4** × (1 + 0.2 × stacks); clear the stacks.
    Then Voidheart expires, Touch of the Void starts, Voidwraith spawns (below).
  - Observed ~2.3 stacks per rift (10.06 starts + 12.78 refreshes).
- **Voidheart** (449880, buff 449887):
  - Starts with the rift (8.1 s, extended with it). Gives **+5% to all of the player's
    Shadow damage**, pets excluded [simc `composite_player_multiplier`].
  - **Touch of the Void** (1266856): Voidheart again for **8 s** after the rift ends.
    Uptime 70.3% [obs].
- **Voidwraith** (451234): spawned when each rift ends (§2.9).
- Dark Energy and Void Leech: movement and healing only. Devour Matter needs an absorb shield,
  so it's inactive on Patchwerk. Inner Quietus, Overwhelming Shadows and Void Infusion are
  plain modifiers.

### 2.8 Idol of N'Zoth (373280) and Y'Shaarj (via Void Apparitions)

- **N'Zoth stacks** on the target (1243069) by source [simc]:

| source | stacks |
|---|---|
| Mind Blast / Void Blast cast | 6 |
| SW:M hard cast | 12 |
| SW:P hard cast | 3 |
| VT application | 4 |
| SW:D | 4 |
| Void Volley cast | 10 |
| Voidform cast | 10 |
| Tentacle Slam impact | 6 |
| Mind Flay tick | 1 |
| Void Torrent tick | 2 |

- **Thresholds**:
  - Crossing **50** (but not on the very first application) → **Horrific Vision**: sp 1.82,
    the Insanity buff (§2.1), and an apparition.
  - Crossing **100** → **Vision of N'Zoth**: sp 6.89, the Insanity buff, an apparition.
    Reset to `new − 100`.
  - Observed 24.0 HV and 19.8 VoN (both include deck draws).
- Model with a resource `nzoth` (max 200) and a resource or buff `nzoth_hv_done`. A helper
  `nzoth_check` runs after every gain. The insanity buffs become `trigger` with
  `count`/`interval` gains.
- **Y'Shaarj idol** from the deck (talent not taken):
  - spawns a Shadowfiend;
  - while a fiend is out, buff `idol_of_yshaarj` +10% haste (373316, 9.0% uptime) [simc];
  - also triggers an apparition.

### 2.9 Pets

| pet | spawned by | duration | action | damage | Insanity |
|---|---|---|---|---|---|
| thing_from_beyond | Yogg idol at 20 stacks, deck 2/17 | 20 s [db2 373277] | `void_spike` 2 s cast, hasted | sp 0.938961, owner SP ×1.0, Shadow Weaving applies [simc] | — |
| voidwraith | each rift end | 5 s [db2 451235] | `void_flay` 1.5 s cast/GCD | sp 1.0, owner SP ×0.5, ×(1 + 0.5 × target.health.pct/100) [simc]; use `damage.scale` | 1 per hit |
| shadowfiend | SW:D below 20%, 40% chance [db2 34433 #2]; Y'Shaarj deck draw | 5 s [db2 1280172] | melee 1.5 s hasted | owner SP ×0.408 as AP, weapon = spell_scaling × 2 [simc] | 1 per hit |
| void_tendril | deck 7/17 | 10 s [db2 377355] | mind_flay 193473, 2 s unhasted ticks | sp 0.6436/tick | 1 per tick |

- Several Things from Beyond can be out at once: 9.07 starts and 2.36 overlaps. Check the
  174.8 Void Spikes [obs]. If the count is low, check how the pet loop handles the GCD:
  try `gcd = 0` with the 2 s cast.
- **shadow_weaving** (346111, 0.43%): fiend melee and Void Flay don't get mastery. Instead
  each hit makes an extra hit of `damage × (shadow-weaving mult − 1)`, which can't crit.
  Approximate: an owner ability triggered from those pet hits, with the pet hit's
  coefficient, `scale = "3*mastery"` and `crit_bonus = -1`, and let calibration fit it.
  Low priority.

### 2.10 Shadow Word: Death, Power Infusion, Twist of Fate, Psychic Link, misc

- **SW:D** 32379:
  - 1 charge, 10 s recharge, instant, sp **0.85** (SimC uses #1; #0 2.55 is Devour Matter).
  - **×2.5 below 20% health** (#3, 150%) [simc]. 4 Insanity, 4 N'Zoth stacks.
  - The APL casts it **only in execute** (`target.health.pct<20`). On a hit below 20%,
    40% chance to summon a Shadowfiend.
  - Self-damage is irrelevant. Death's Torment and Death and Madness are not taken.
- **Power Infusion** 10060: +20% haste for 15 s, 120 s cooldown. **Self-cast**: simc option
  `self_power_infusion` defaults to true, and Twins of the Sun Priestess is taken. The APL
  syncs it with Voidform: 2.95 casts, 14.5% uptime [obs].
- **Twist of Fate** (390972, 2 ranks, buff 390978): +5% damage per rank, so **+10%**
  [db2, check with effects.py], while the target is below **35%**. Use `below_health = 35`.
  Observed uptime 34.9%.
- **Psychic Link**: only hits *other* targets with VT, so 0 on single target. Omit.
- **Entropic Embrace** (Void Elf racial 255669): RPPM 1, 12 s, +5% damage
  [db2 256374 #0]. Use an `on_hit` proc. 26% uptime [obs].
- **Gear stat procs** (rune of critical power, soul_fang_alacrity, venomcursed_*,
  arcanoweave_insight, genius_insight): time-average into `[player]` and `[gear.procs]` as
  the destruction spec does.
- **Tier set**: the core kit disables `midnight_season_2_4pc`. SimC's MID2 4pc is a chance
  per Tentacle Slam to add a Void Volley/Crushing Void stack at ×1.25 effectiveness. Not
  needed for the reference.
- **Shadow Weaving (mastery)**: multiplier `1 + n × mastery_value`. n counts SW:P, VT and
  SW:M ticking on the target; the DoT being cast counts as up; in Voidform n = 3 [simc
  `shadow_weaving_active_dots`]. All three DoTs are up ~98% of the fight (Insidious Ire
  uptime), so **use `mastery = true, mastery_scale = 3`** on every affected source, with
  `player.mastery = 0.2043` [obs].
  - Affected: every player damage source above except rune procs; plus pet Void Spike and
    Void Tendril.
  - Not affected: fiend melee and Void Flay (they use `shadow_weaving` instead).
  - Gear: `mastery_coeff = 0.5` (% per point, per DoT) [db2 343690].
  - The mastery probe (+500 rating → 0.2523) checks the scaling.

## 3. Rotation (single target, from `ActionPriorityLists/default/priest_shadow.simc`)

Precombat [simc]:
1. `shadowform`;
2. `tentacle_slam` (VT and a 25% SW:M up at pull);
3. Mind Blast only with the 4pc, so not here.

`dots_up` = VT ticking & SW:P ticking.

1. **Cooldowns**, when `dots_up` (or fewer than 30 s left):
   - `power_infusion` if `buff.voidform.up & !buff.power_infusion.up`;
   - `voidform` (needs SW:P ticking; it's ready only when no volley/crushing stacks are held).
2. `tentacle_slam` if `!dot.vampiric_touch.ticking`.
3. `vampiric_touch` if `!dot.vampiric_touch.ticking` (rare; Slam covers it).
4. `shadow_word_madness` if any of:
   - `dot.shadow_word_madness.remains <= gcd`;
   - `insanity.deficit <= 35 & (cooldown.voidform.remains > 25 | insanity.deficit <= 5)`;
   - `buff.entropic_rift.up & cooldown.voidform.remains >= 75`;
   - `buff.voidform.up` (spam it in Voidform);
   - fewer than 10 s left.
5. `tentacle_slam` if `dot.vampiric_touch.refreshable` or
   `cooldown.tentacle_slam.charges_fractional >= ~1.8` (full recharge within 2 GCDs).
6. `void_torrent` if `dots_up`.
7. `shadow_word_pain` if `dot.shadow_word_pain.refreshable` (remains < 4.8) & VT ticking.
8. `void_volley` if `cooldown.voidform.up` (spends a leftover Crushing Void before the next
   Voidform).
9. `mind_blast_insight`, then `mind_blast`. Both route to Void Blast while the rift is up;
   the APL puts Void Blast above Mind Blast, and there's nothing between them.
10. `tentacle_slam` (Void Apparitions: dump charges whenever available on Patchwerk).
11. `vampiric_touch` if refreshable and Slam isn't back before it falls off (rare).
12. `shadow_word_death` if `target.health.pct < 20`.
13. `void_volley` (any stack: Voidform charges, Crushing Void).
14. `mind_flay`: 2 ticks (`interrupt_if=ticks>=2`). Model as `channel.ticks = 2`.

`insanity.deficit` = `insanity.max-insanity`; `gcd.max` → `gcd`; `refreshable` → `remains <
0.3*duration`. `full_recharge_time` isn't in the expression language, so use
`charges_fractional`.

## 4. Engine gaps

I checked `docs/spec-format.md` and, in `src/spec.rs` and `src/sim.rs`: `BuffDef`
(no tick-rate field), `DotDef` (no `scale`; refresh only `pandemic`/`extend`/`reset`),
`tick_len`, `apply_dot`, `dot_tick`, the expression resolver (no `full_recharge_time`,
`in_flight`, `refreshable`), and the pet loop. Everything not listed here can be expressed
with existing features, as described in §2:
- shared MB/VB charges: one routed ability;
- Shadow Weaving: `mastery_scale = 3`;
- rift lifetime: buff plus timer DoT;
- decaying Ancient Madness: stack buff plus timer DoT;
- N'Zoth thresholds: resources plus `if`;
- volley bolts: `trigger` `count`/`interval`;
- deck RNG: independent chances.

### 4.1 Per-DoT tick-rate buffs (needed)
- **Why**: Screams of the Void makes SW:P and VT tick 40% faster at 67.5% uptime, about +27%
  ticks. Hit counts must come from mechanics, and calibration can't fix counts.
- **Proposal**: `BuffDef.tick_rate_pct: HashMap<dot name, pct>`, in the same shape as
  `cooldown_rate_pct`.
  - `tick_len(di)` divides by `1 + Σ pct × stacks` of the active buffs that name that DoT.
  - SimC recomputes tick time at every tick, so applying it at the next scheduled tick is
    faithful.
  - The partial last tick uses the same `tick_len`.
- **Files**: `src/spec.rs` (field), `src/sim.rs` (compile a `tick_rate_buffs` list per DoT;
  `tick_len` takes `di`; callers in `start_ticking` and the `Ev::DotTick` handler at
  ~l.2278), `docs/spec-format.md`. Existing specs don't use it, so their results stay
  bit-identical.
- **Cheap fallback** if this is deferred: give SW:P and VT a static `tick_time` × 0.787
  (= 1/(1 + 0.4 × 0.675)) [obs]. Tick counts land right on the reference but don't respond to
  SW:M cadence in the probes.

### 4.2 `full_recharge_time` expression (nice-to-have, trivial)
- `cooldown.X.full_recharge_time`: time until all charges are back. Tentacle Slam's APL uses
  it. Approximating with `charges_fractional>=1.8` is fine, so this is optional.
- **Files**: `src/expr.rs` (Var), `src/sim.rs` (resolver, eval).

### 4.3 Rollover DoT refresh with application strength (only if the §2.4 approximation fails)
- **Why**: SW:M refresh and the 25% Maddening Tentacles application. The `extend` + `extend_dot
  1.5 s` approximation should hold because SW:M never drops. If SW:M ticks or casts miss the
  reference by more than ~3%, add:
  - `refresh = "rollover"`: pandemic-capped duration, and a per-DoT persistent multiplier
    `m = (old_ticks_left·m_old + new_ticks·strength)/new_ticks_left`, applied in `dot_tick`;
  - an effect `{ type = "apply_dot", dot = "...", strength = 0.25 }` so another ability can
    apply a DoT at reduced strength.
- **Files**: `src/spec.rs` (Effect variant, refresh validation), `src/sim.rs` (`dot_mult`
  state reset with the DoT, `apply_dot`, `dot_tick`, `compile_effects`),
  `docs/spec-format.md`.

Not gaps (checked):
- channel `tick_on_apply` (Void Torrent) is supported;
- pet `damage.scale` with `target.health.pct` (0–100, `health_pct()` l.1151) works for Void Flay;
- owner resource `gain` from pet effects, multi-instance pets and `below_health` buffs all exist;
- DoT `on_expire` is missing, but the buff+timer pattern covers the rift.

## 5. Open questions / risks

1. **Void Apparition rate**: 30% observed against 15% in DB2/SimC. Use [obs], and say so in
   `docs/shadow-priest.md`.
2. **Void Torrent ticks per cast** (5.63 observed): the channel tick rounding is unclear.
3. **Missing Void Tendril row** (~0.8%, §0). Expect azsim to sit ~0.8% above the summed
   sources unless the tendril is modelled with an estimated per-hit damage. Better: fix
   `tools/observe.py` to prefix a child's name with its parent's when the names collide,
   then regenerate the reference.
4. **SW:D execute multiplier** (×2.5) and the Twist of Fate rank value: confirm with
   `tools/effects.py`.
5. **Insanity budget** (§2.1): roughly 10% short in a hand count. Check gains in the engine
   output before tuning the rotation.
