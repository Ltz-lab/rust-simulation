//! The Havoc overlay picks sensible next presses from combat-log events alone.
use azsim::overlay::{Demo, LogEvent, Overlay, Tracker};

const ME: &str = "Player-1";
const BOSS: &str = "Creature-1";
const FURY: i64 = 17;

fn overlay() -> Overlay {
    Overlay::load(concat!(env!("CARGO_MANIFEST_DIR"), "/overlay/havoc_fel_scarred.toml")).unwrap()
}

/// A tracker that has seen the pull: the player swung at the boss with `fury` Fury.
fn pulled(o: &Overlay, fury: f64) -> Tracker<'_> {
    let mut tr = Tracker::new(o);
    tr.player = Some(ME.into());
    tr.ingest(&LogEvent::new(0.0, "SWING_DAMAGE", ME, BOSS, 0, 1000.0, 1));
    tr.ingest(&LogEvent::new(0.0, "POWER_UPDATE", ME, ME, 0, fury, FURY));
    tr
}

fn cast(tr: &mut Tracker, t: f64, id: i64) {
    tr.ingest(&LogEvent::new(t, "SPELL_CAST_SUCCESS", ME, BOSS, id, 0.0, 1));
}
fn aura(tr: &mut Tracker, t: f64, id: i64, secs: f64) {
    tr.ingest(&LogEvent::new(t, "SPELL_AURA_APPLIED", ME, ME, id, secs * 1000.0, 1));
}
fn energize(tr: &mut Tracker, t: f64, amount: f64) {
    tr.ingest(&LogEvent::new(t, "SPELL_ENERGIZE", ME, ME, 203796, amount, FURY));
}
/// The next GCD press (off-GCD picks such as Vengeful Retreat are woven in between).
fn next(tr: &Tracker) -> String {
    tr.recommend(3).into_iter().find(|p| tr.o.spec.spells[tr.o.spell_index(&p.action).unwrap()].gcd != Some(0.0)).unwrap().action
}

const EYE_BEAM: i64 = 198013;
const DEATH_SWEEP: i64 = 210152;
const BLADE_DANCE: i64 = 188499;
const META_CAST: i64 = 191427;
const META_BUFF: i64 = 162264;
const ESSENCE_BREAK: i64 = 258860;
const ANNIHILATION: i64 = 201427;
const FELBLADE: i64 = 232893;
const IMMOLATION: i64 = 258920;

#[test]
fn every_rule_parses_and_names_real_ids() {
    let o = overlay();
    assert!(o.spec.spells.iter().all(|s| !s.ids.is_empty()));
    assert!(o.spec.auras.iter().all(|a| !a.ids.is_empty()));
    assert_eq!(o.spell_by_id(452497).map(|i| o.spell_name(i)), Some("abyssal_gaze"));
    assert_eq!(o.spell_by_id(452487).map(|i| o.spell_name(i)), Some("consuming_fire"));
}

#[test]
fn opener_is_eye_beam_into_the_hunt() {
    let o = overlay();
    let tr = pulled(&o, 50.0);
    let q: Vec<_> = tr.recommend(3).into_iter().map(|p| p.action).collect();
    // Eye Beam first (Demonic opens Metamorphosis), The Hunt once Eye Beam is down and
    // Metamorphosis is still up [simc line 44], then Essence Break for the Demonsurge window.
    assert_eq!(q, ["eye_beam", "the_hunt", "essence_break"]);
}

#[test]
fn demonic_window_spends_the_empowerments() {
    let o = overlay();
    let mut tr = pulled(&o, 80.0);
    cast(&mut tr, 0.0, EYE_BEAM);
    aura(&mut tr, 0.0, META_BUFF, 6.9); // Demonic
    tr.advance(1.7);
    assert_eq!(next(&tr), "the_hunt", "[simc line 44] Eye Beam down, Metamorphosis ready");
    cast(&mut tr, 1.7, 370965);
    tr.advance(1.8);
    assert_eq!(next(&tr), "essence_break");
    cast(&mut tr, 1.8, ESSENCE_BREAK);
    tr.ingest(&LogEvent::new(1.9, "SPELL_AURA_APPLIED", ME, BOSS, 320338, 4000.0, 1));
    tr.advance(2.9);
    assert!(tr.aura_remains("essence_break") > 2.0, "debuff read from the target");
    assert_eq!(next(&tr), "death_sweep");
    cast(&mut tr, 2.9, DEATH_SWEEP);
    energize(&mut tr, 3.0, 40.0);
    tr.advance(3.9);
    assert_eq!(next(&tr), "annihilation");
    cast(&mut tr, 3.9, ANNIHILATION);
    tr.advance(4.9);
    // Both empowerments are spent, Death Sweep holds the shared Blade Dance cooldown and Eye Beam is
    // 15s off: hardcast Metamorphosis now, extending the Demonic one [simc line 42].
    assert_eq!(next(&tr), "metamorphosis");
}

#[test]
fn metamorphosis_hardcast_after_the_demonic_window() {
    let o = overlay();
    let mut tr = pulled(&o, 60.0);
    cast(&mut tr, 0.0, EYE_BEAM);
    aura(&mut tr, 0.0, META_BUFF, 6.9);
    cast(&mut tr, 1.7, 370965); // The Hunt
    cast(&mut tr, 2.0, DEATH_SWEEP);
    cast(&mut tr, 3.0, ANNIHILATION);
    tr.ingest(&LogEvent::new(6.9, "SPELL_AURA_REMOVED", ME, ME, META_BUFF, 0.0, 1));
    tr.advance(7.0);
    // Eye Beam is ~13s off, Blade Dance shares Death Sweep's cooldown: Metamorphosis [simc line 42].
    assert_eq!(next(&tr), "metamorphosis");
    cast(&mut tr, 7.0, META_CAST);
    aura(&mut tr, 7.0, META_BUFF, 20.0);
    energize(&mut tr, 7.5, 90.0);
    tr.advance(8.0);
    // Chaotic Transformation reset Eye Beam and Blade Dance; the four Demonsurge empowerments
    // are up, and Essence Break opens them [simc line 50].
    assert_eq!(tr.cooldown_remains("eye_beam"), 0.0);
    assert_eq!(next(&tr), "essence_break");
    let q: Vec<_> = tr.recommend(5).into_iter().map(|p| p.action).filter(|a| a != "vengeful_retreat").collect();
    assert_eq!(&q[1..3], ["death_sweep", "annihilation"]);
}

#[test]
fn low_fury_builds_with_felblade() {
    let o = overlay();
    let mut tr = pulled(&o, 20.0);
    // Meta first: its cast resets Eye Beam, Blade Dance and Immolation Aura. Then everything else down.
    cast(&mut tr, 0.0, META_CAST);
    cast(&mut tr, 0.0, EYE_BEAM);
    cast(&mut tr, 0.0, 370965);
    cast(&mut tr, 0.0, 198793);
    cast(&mut tr, 0.0, ESSENCE_BREAK);
    cast(&mut tr, 0.0, BLADE_DANCE);
    cast(&mut tr, 0.0, IMMOLATION);
    tr.ingest(&LogEvent::new(0.0, "POWER_UPDATE", ME, ME, 0, 20.0, FURY));
    tr.advance(3.0);
    assert_eq!(next(&tr), "felblade");
    cast(&mut tr, 3.0, FELBLADE);
    energize(&mut tr, 3.0, 15.0);
    energize(&mut tr, 3.3, 15.0);
    tr.advance(3.6);
    assert_eq!(next(&tr), "chaos_strike");
}

#[test]
fn aoe_puts_immolation_aura_above_chaos_strike() {
    let o = overlay();
    let mut tr = pulled(&o, 60.0);
    for id in [META_CAST, EYE_BEAM, 370965, ESSENCE_BREAK, BLADE_DANCE, FELBLADE, 198793] { cast(&mut tr, 0.0, id); }
    tr.ingest(&LogEvent::new(0.0, "POWER_UPDATE", ME, ME, 0, 60.0, FURY));
    tr.advance(2.0);
    assert_eq!(tr.active_enemies(), 1);
    assert_eq!(next(&tr), "chaos_strike");
    for u in ["Creature-2", "Creature-3"] {
        tr.ingest(&LogEvent::new(2.0, "SPELL_DAMAGE", ME, u, BLADE_DANCE, 5000.0, 1));
    }
    assert_eq!(tr.active_enemies(), 3);
    assert_eq!(next(&tr), "immolation_aura");
    // the extra targets drop out of the window
    tr.advance(6.0);
    assert_eq!(tr.active_enemies(), 1);
}

#[test]
fn closed_loop_fight_is_sensible() {
    let o = overlay();
    let extra = [("eye_beam", "eternal_hunt", 2.0, 8.0), ("eye_beam", "demonsurge", 0.7, 15.0), ("metamorphosis", "demonsurge", 0.7, 15.0)];
    let d = Demo::run(&o, 120.0, 1, 40.0, 1.3, 15.0, &extra);
    let n = |a| d.count(a);
    assert!(n("eye_beam") + n("abyssal_gaze") >= 5, "Eye Beam family on cooldown");
    assert!(n("metamorphosis") >= 1 && n("the_hunt") >= 2);
    assert!(n("essence_break") >= 3);
    // never a press the game would refuse for Fury
    for (t, a, fury, _) in &d.presses {
        let s = &o.spec.spells[o.spell_index(a).unwrap()];
        assert!(*fury >= s.cost, "{a} at {t:.2}s with {fury} fury");
    }
    // no long idle stretches
    let gaps = d.presses.windows(2).map(|w| w[1].0 - w[0].0).fold(0.0, f64::max);
    assert!(gaps < 4.0, "longest gap {gaps:.1}s");
}
