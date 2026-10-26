# Paired comparisons


Gear and stat decisions care about *differences* between two setups, not
absolute DPS. azsim draws every random number from its own deterministic
stream, keyed by (iteration seed, purpose): fight length, the crit roll of each
damage source, the damage range of each source, every chance roll site (each
proc or trigger effect in the spec) and every RPPM proc. So the 40th Final
Verdict crit roll is the same number in both runs of a pair, even when one run
got an extra proc earlier and the two fights have drifted apart. Run two setups
with the same `--seed` and they share all the luck they can.

`tools/paired_noise.py` measures it: 50 seeds × 500 iterations per setup, the
same stat changes as the held-out probes, and the SD of the DPS delta when both
setups use the same seed (paired) vs different seeds (independent). The
variance-reduction factor is how many times fewer iterations a paired
comparison needs for the same precision (each factor is good to about ±20%).

| probe | mean delta | before: indep / paired SD | before: reduction | after: indep / paired SD | after: reduction |
|---|---:|---:|---:|---:|---:|
| +500 crit | +10,973 | 223 / 129 | 3.0x | 180 / 47 | **14.8x** |
| +500 haste | +12,915 | 220 / 136 | 2.6x | 182 / 74 | **6.1x** |
| +500 mastery | +12,306 | 226 / 132 | 2.9x | 179 / 29 | **37.6x** |
| +500 vers | +11,723 | 239 / 12 | 411x | 175 / 12 | **226x** |

"Before" is the previous engine, where every roll came in order from one shared
RNG per iteration. Once two runs diverged, every later roll landed on a
different event.

In iterations: a paired +500 crit comparison at 500 iterations is as precise as
an unpaired one at about 7,400.

Versatility changes no decisions, so pairing removes nearly all of its noise
either way. What's left is its gain being proportional to DPS (8.8% of a run's
SD of 132 is 12). Crit and mastery only move roll thresholds, so per-purpose
streams keep almost every outcome aligned. Haste is the hard case: it shifts
every GCD and tick, so the rotation eventually takes a different branch and the
two runs diverge in what they cast. Even +0.2% haste leaves a paired SD of
about 36. The streams still keep each source's n-th roll shared, but cast-count
divergence is the floor. With all randomness turned off except fight length, a
haste change still has a paired SD of about 20, because its gain genuinely
depends on how many cooldowns fit into the fight.

Streams are keyed by *name* (`crit/final_verdict`, `roll/final_verdict/on_cast/1`,
`rppm/rune_of_unleashed_fire`, ...), not by position in the spec. So a gear set
that adds a trinket ability or proc leaves every other stream untouched:
inserting a firing, roll-consuming but damageless ability at the top of the
spec gives bit-identical DPS for the same seed. Each stream is a splitmix64
sequence, with no measurable cost in speed.
