# Outlaw Rogue (Trickster), Midnight Season 2

`specs/outlaw_trickster.toml`, calibrated against
`reference/outlaw_core_observations.json` (the core kit built from
`profiles/MID2/MID2_Rogue_Outlaw.simc` by `tools/make_reference.sh`).

## Result

| | reference | azsim |
|---|---:|---:|
| DPS | 162,375 | 155,630 (−4.2%) |
| fight length | 300 s | 300 s ± 20% |

`./target/release/azsim specs/outlaw_trickster.toml -n 10000 --seed 1 --calib calibration/outlaw_trickster.toml`

Sources within ±5%: Between the Eyes (−0.6%), Dispatch (+7.2%), Pistol Shot
(+4.3% / +4.5%), Main Gauche (−4.4%), Unseen Blade (−0.4%), Coup de Grace
(+1.0%), Sinister Strike (+4.4%), the gear procs (±1%). The remaining gap is
Killing Spree (−21.7%: 7.9 casts a fight instead of 9.95 because Adrenaline
Rush comes back 10-15% slower), auto attacks (−7.5%, SimC's swing timer also
resets on attack-speed changes), and Instant Poison (−12.7%), which follows
the auto-attack count.

## Validation (stat probes)

Held out from calibration (`tools/validate_stats.py`, same calibration):

| probe | reference | azsim | diff |
|---|---:|---:|---:|
| +500 crit rating | +9.71% | +9.53% | −0.18 |
| +500 haste rating | +9.36% | +10.26% | +0.90 |
| +500 mastery rating | +5.82% | +5.46% | −0.36 |
| +500 versatility rating | +7.41% | +7.78% | +0.37 |
| +200 agility | +5.87% | +5.97% | +0.10 |

## What is modelled

- Energy (10/s + Vigor 5/s, hasted, 13.1/s base so 17.6/s with the reference's
  haste), combo points (7 = 5 + Deeper/Devious Stratagem), Adrenaline Rush's
  +75% regeneration and +20% attack speed, Blade Rush's regeneration not used,
  Thistle Tea's automatic 100 energy below 30.
- The APL: Adrenaline Rush, Preparation, Keep It Rolling, Roll the Bones
  rerolls, Between the Eyes / Killing Spree / Coup de Grace / Dispatch
  finishers, Pistol Shot with Fan the Hammer and Quick Draw, Sinister Strike.
- Roll the Bones' four states (builder damage and combo points, Restless
  Blades' 1.4x, Jackpot's crit), Loaded Dice and Keep It Rolling extensions,
  Dragon-Bone Dice's rank-ups.
- Opportunity (30% + 25% while Roll the Bones is up), Fan the Hammer's two
  extra shots, Quick Draw's combo points, Gravedigger's free Between the Eyes
  and Scoundrel Strike, Ace Up Your Sleeve, Ruthlessness, Restless Blades and
  Cut to the Chase.
- Killing Spree's channel (tick count from effective combo points, hasted and
  17% faster during Adrenaline Rush, main- and off-hand strikes, one combo
  point a tick).
- Outlaw mastery as Main Gauche (30% of main-hand attacks; its damage is
  `0.001 + 13.1 x mastery`), Instant Poison (30% per strike), Fatal Flourish.
- Trickster: Unseen Blade (modelled at the observed 13.4/min, which folds in
  Thousand Cuts' lockout resets and Disorienting Strikes), Fazed (+8% with
  Hoodwink), Escalating Blade, Flawless Form, Coup de Grace.
- The gear passives in the core kit: Searing Lightwood, Venomfang (+ burst),
  Rune of Unleashed Fire (+ lingering). The Venomcursed crit/haste and
  Arcanoweave/Might of the Void stat procs are folded into `[player]`
  (`crit = 0.336`, `haste = 0.352`, `attack_power = 3577` — the agility probe
  says damage scales on ~3,577 AP in the reference fights, not the 2,593
  snapshot).
- Crit chance and damage: 2.13x base crit multiplier; +50% crit damage for
  combo-point generators (Surprising Strikes, Fazed is up throughout) and
  Between the Eyes (Improved Between the Eyes); +5% crit for the generators
  (Deadly Precision), +20% for Pistol Shot (Clever Combatant), Zero In's +2%
  crit/damage a stack.

## Known gaps

- Killing Spree and Adrenaline Rush cadence (see above): the reference's
  Restless Blades chain is ~10% stronger. SimC rolls Killing Spree with
  `total_cp` including Supercharger; the spec does the same.
- Auto attacks are ~7.5% short of the reference. The engine does not shorten a
  swing in progress when haste changes, which is where SimC's extra swings
  come from.
- Unseen Blade is a flat 13.4 procs/min rather than the real lockout:
  20 s after each proc, cleared early by Thousand Cuts and Disorienting
  Strikes. Its damage and Fazed refresh are right; the timing is smoothed.
- Killing Spree's first tick grants a combo point in the spec (SimC skips it),
  and Coup de Grace's three strikes land at once instead of at 0.3 s / 1 s.
- Blade Rush is in SimC's APL but never usable in the reference sim, so it is
  not used here either; Vanish/Ambush need Hidden Opportunity, which this
  build does not take.
- No `[gear]`/`[gear.kit]` yet: `[player]` stats are the reference's in-fight
  averages.
