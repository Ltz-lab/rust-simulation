# Destruction Warlock (Hellcaller)


`specs/destruction_hellcaller.toml`, built the same way as Ret and Fire:
coefficients, cast times, charges, durations and talent values from DB2 (plus
the `Spell` table's tooltip text for what each talent does), SimC's class
module where a mechanic isn't in the data, and the rest observed in SimC's log.
Destruction brings a fractional resource spent in chunks, spenders whose damage
grows with crit chance, a DoT whose stacks are a resource of their own, and
three summoned demons. Every mechanic below is generic engine data, not
warlock code.

## Results

Reference: SimC's Midnight S2 sample Destruction profile (Hellcaller), core kit
only (no trinkets, tier set, consumables or Bloodlust), 300s Patchwerk, 5,000
iterations. Pet damage (imp, Infernal, Overfiend) is included.

| | DPS |
|---|---:|
| reference | 157,019 |
| azsim | 155,825 (±76) |
| difference | **-0.76%** |

## Structure: hit counts per fight

| source | reference DPS | azsim DPS | diff | ref hits | azsim hits |
|---|---:|---:|---:|---:|---:|
| chaos_bolt | 43,529 | 43,319 | -0.5% | 48.8 | 48.6 |
| incinerate | 22,134 | 21,762 | -1.7% | 112.3 | 110.5 |
| shadowburn | 18,958 | 18,616 | -1.8% | 37.9 | 37.2 |
| conflagrate | 14,506 | 14,353 | -1.1% | 42.1 | 41.6 |
| blackened_soul | 12,489 | 12,265 | -1.8% | 190.0 | 186.7 |
| soul_fire | 10,968 | 10,964 | -0.0% | 6.8 | 6.8 |
| wither | 10,603 | 10,610 | +0.1% | 147.2 | 147.4 |
| overfiend_chaos_bolt | 5,685 | 5,600 | -1.5% | 22.5 | 22.2 |
| imp_firebolt | 4,116 | 4,118 | +0.0% | 78.4 | 78.4 |
| echo_of_sargeras_cb | 3,387 | 3,442 | +1.6% | 47.8 | 48.6 |
| infernal_immolation | 2,899 | 2,877 | -0.8% | 83.7 | 83.2 |
| echo_of_sargeras_sb | 2,601 | 2,661 | +2.3% | 36.3 | 37.2 |
| malevolence_proc | 1,577 | 1,572 | -0.4% | 5.4 | 5.4 |
| rune_of_unleashed_fire | 1,027 | 1,028 | +0.1% | 50.0 | 50.1 |
| infernal_melee | 960 | 961 | +0.2% | 79.8 | 80.0 |
| echo_of_sargeras | 777 | 788 | +1.4% | 10.9 | 11.0 |
| wither_direct | 411 | 411 | +0.0% | 9.4 | 9.4 |
| rune_of_unleashed_fire_lingering | 315 | 368 | +16.8% | 245.6 | 287.0 |
| infernal_awakening | 51 | 51 | +0.2% | 3.7 | 3.7 |

The Soul Shard economy matches: Incinerate generates 55.7 shards per fight
(56.1 in the reference) and Conflagrate 20.8 (20.9). Chaotic Inferno makes 12.1
Incinerates a fight instant (11.4), Fiendish Cruelty procs 14.6 times (14.0),
and Blackened Soul burns 187 Wither stacks (190).

## Held-out validation: stat sensitivity

| stat probe | reference DPS gain | azsim DPS gain | difference |
|---|---:|---:|---:|
| +500 crit | +8.72% | +8.63% | -0.09 pts |
| +500 haste | +8.23% | +8.33% | +0.10 pts |
| +500 mastery | +8.99% | +9.47% | +0.48 pts |
| +500 vers | +8.78% | +8.76% | -0.02 pts |
| +200 intellect | +6.50% | +6.53% | +0.04 pts |

Crit, haste, vers and intellect are within 0.1 points, and the stat priority
(mastery > vers > crit > haste > intellect) is the reference's exactly. Crit
matters most here: Chaos Bolt, Shadowburn, Conflagrate and the Overfiend's
Chaos Bolt always crit and gain damage equal to crit chance, so for about half
the damage crit works through that multiplier rather than through crit rolls. See [Known gaps](#known-gaps) for mastery.

## Speed

Same machine (10 cores), 10,000 iterations: azsim **0.22s**, SimC **1.6s**.

## What the raw data alone gets you

With DB2 coefficients and the modelled mechanics but no calibration, the spec
reaches **62% of the reference** (97.8k). The derived layer then explains most
of the gap: the Destruction spec aura's +88% to Wither, Through the Felvine's
+35% to Chaos Bolt (doubled inside Malevolence), Improved Chaos Bolt, Roaring
Blaze and the school talents. What's left per source is ×1.05-1.20 on
everything the player casts; only the pets' spell power (imp ×7.1, Infernal
Immolation ×2.8) sits beyond ±25%.

Mechanics read from SimC's source rather than the tables: Chaotic Energies
rolling 50-100% of mastery per hit (70-100% for Chaos Bolt and Shadowburn with
Chaos Incarnate), Wither's damage *not* scaling with its stacks, Blackened
Soul's collapse counting down from the stack count it started at and resyncing
whenever Wither is at 8+, the 23% collapse chance and 15% Bleakheart Tactics
chance, Malevolence adding its duration on refresh, Chaotic Inferno landing just
after the Chaos Bolt that procs it, and the Chaos Bolt/Shadowburn echoes
hitting at full damage despite the tooltip's 50%.

## Engine features added for Destruction

All generic, data-driven, and Ret's and Fire's results are bit-for-bit
unchanged:

- **`crit_scaling`**: damage further increased by the player's crit chance
  (Chaos Bolt, Shadowburn, Conflagrate);
- **`mastery_scale`**: the share of mastery a hit gets, for a mastery that rolls
  a random fraction per hit (0.75 on average, 0.85 with Chaos Incarnate);
- **DoT stacks**: `max_stacks` and `per_stack` on DoTs, a `dot_stack` effect to
  add, remove (down to a floor) or `set` stacks from a condition expression,
  `cancel_dot`, and `dot.X.stack` in conditions (Wither's stacks, the Blackened
  Soul countdown);
- DoT **`on_crit`** effects (Wither crits' extra shard fragment) and
  **`whole_ticks`** for pet attacks and casts that stop at expiry instead of
  dealing a shortened last tick;
- **buffs**: per-ability `cast_time_pct` (Backdraft), `flat` buffs whose
  bonuses don't multiply with stacks (Backdraft's charges, Crashing Chaos's 8
  Chaos Bolts), `refresh_extend` (Malevolence), and a `duration` override on
  the `buff` effect (Devil Fruit's 8s Malevolence).

## Known gaps

- **Mastery +0.48 pts.** Every mastery source responds 0.5 points more than in
  the reference, uniformly; but the reference's imp, Infernal and Malevolence
  hits, which don't use mastery at all, also lose 0.5% in that probe. So the
  gap looks like something probe-wide (rating diminishing returns on the extra
  mastery, which the time-averaged gear procs here don't see) rather than a
  per-spell error.
- **Hit counts 1-2% low on casts.** About two fewer hardcast Incinerates per
  fight and a little less Shadowburn and Conflagrate; cast times and Chaotic
  Inferno instants match the reference, so it's a couple of seconds of casting
  time a fight. Gear haste procs are folded in as a time average here, while in
  the reference they come and go.
- **Fitted rather than derived [obs]:** Wither's 3s base tick (DB2 lists 2s),
  the gear stat procs as time-averaged ratings, Vision of Nihilam's 8% haste
  and crit, Reverse Entropy and Devil Fruit as Poisson rates (RPPM with bad-luck
  protection in game), Flashpoint from Seeds of Their Demise (a threshold roll,
  approximated as 4.8% per Blackened Soul hit while Flashpoint is down), the
  crit damage of Wither and Blackened Soul (Mark of Peroth'arn), and the pets'
  timings (imp energy pool then one Firebolt per 4s; Infernal swing, Immolation
  and Ember timers).
- **Simplified:** proc chances that are PRD in game (Fiendish Cruelty,
  Chaotic Inferno, Embers of Nihilam, Bleakheart Tactics) are flat; Mark of
  Xavius's +2% Blackened Soul damage per Wither stack is left to the residual;
  no travel time; Shadowburn's refund on a dying target isn't modelled; Rune of
  Unleashed Fire's lingering DoT overlaps too long, as for Fire.
