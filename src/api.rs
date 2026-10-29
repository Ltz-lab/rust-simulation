//! Batch simulation: one base character plus N gear variants in one call.
//!
//! Spec and calibration are parsed once. Each variant is the base gear with some
//! slot lines replaced; all variants (and the base) run the same iteration seeds,
//! so the per-iteration luck is shared (see the RNG streams in `sim`) and the
//! paired difference to the base is much tighter than either DPS alone.

use crate::{gear, sim, spec};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;

const SPECS: &[(&str, &str, &str)] = &[
    (
        "arcane_spellslinger",
        include_str!("../specs/arcane_spellslinger.toml"),
        include_str!("../calibration/arcane_spellslinger.toml"),
    ),
    (
        "elemental_farseer",
        include_str!("../specs/elemental_farseer.toml"),
        include_str!("../calibration/elemental_farseer.toml"),
    ),
    (
        "fury_slayer",
        include_str!("../specs/fury_slayer.toml"),
        include_str!("../calibration/fury_slayer.toml"),
    ),
    (
        "affliction_hellcaller",
        include_str!("../specs/affliction_hellcaller.toml"),
        include_str!("../calibration/affliction_hellcaller.toml"),
    ),
    (
        "shadow_voidweaver",
        include_str!("../specs/shadow_voidweaver.toml"),
        include_str!("../calibration/shadow_voidweaver.toml"),
    ),
    (
        "balance_elunes_chosen",
        include_str!("../specs/balance_elunes_chosen.toml"),
        include_str!("../calibration/balance_elunes_chosen.toml"),
    ),
    (
        "retribution_herald",
        include_str!("../specs/retribution_herald.toml"),
        include_str!("../calibration/retribution_herald.toml"),
    ),
    (
        "fire_mage",
        include_str!("../specs/fire_mage.toml"),
        include_str!("../calibration/fire_mage.toml"),
    ),
    (
        "frost_mage_spellslinger",
        include_str!("../specs/frost_mage_spellslinger.toml"),
        include_str!("../calibration/frost_mage_spellslinger.toml"),
    ),
    (
        "destruction_hellcaller",
        include_str!("../specs/destruction_hellcaller.toml"),
        include_str!("../calibration/destruction_hellcaller.toml"),
    ),
    (
        "subtlety_deathstalker",
        include_str!("../specs/subtlety_deathstalker.toml"),
        include_str!("../calibration/subtlety_deathstalker.toml"),
    ),
    (
        "demonology_diabolist",
        include_str!("../specs/demonology_diabolist.toml"),
        include_str!("../calibration/demonology_diabolist.toml"),
    ),
    (
        "arms_slayer",
        include_str!("../specs/arms_slayer.toml"),
        include_str!("../calibration/arms_slayer.toml"),
    ),
    (
        "enhancement_stormbringer",
        include_str!("../specs/enhancement_stormbringer.toml"),
        include_str!("../calibration/enhancement_stormbringer.toml"),
        "feral_wildstalker",
        include_str!("../specs/feral_wildstalker.toml"),
        include_str!("../calibration/feral_wildstalker.toml"),
    ),
    (
        "devourer_voidscarred",
        include_str!("../specs/devourer_voidscarred.toml"),
        include_str!("../calibration/devourer_voidscarred.toml"),
    ),
    (
        "havoc_fel_scarred",
        include_str!("../specs/havoc_fel_scarred.toml"),
        include_str!("../calibration/havoc_fel_scarred.toml"),
    ),
    (
        "survival_sentinel",
        include_str!("../specs/survival_sentinel.toml"),
        include_str!("../calibration/survival_sentinel.toml"),
    ),
    (
        "assassination_deathstalker",
        include_str!("../specs/assassination_deathstalker.toml"),
        include_str!("../calibration/assassination_deathstalker.toml"),
    ),
    (
        "frost_deathbringer",
        include_str!("../specs/frost_deathbringer.toml"),
        include_str!("../calibration/frost_deathbringer.toml"),
    ),
    (
        "marksmanship_sentinel",
        include_str!("../specs/marksmanship_sentinel.toml"),
        include_str!("../calibration/marksmanship_sentinel.toml"),
    ),
    (
        "unholy_rider",
        include_str!("../specs/unholy_rider.toml"),
        include_str!("../calibration/unholy_rider.toml"),
    ),
    (
        "windwalker_shadopan",
        include_str!("../specs/windwalker_shadopan.toml"),
        include_str!("../calibration/windwalker_shadopan.toml"),
    ),
];

/// Names of the specs built into the library.
pub fn spec_names() -> Vec<&'static str> {
    SPECS.iter().map(|s| s.0).collect()
}

/// True if `spec` (built-in name or path) can turn gear lines into stats.
pub fn supports_gear(spec: &str) -> bool {
    load_spec_text(spec).map(|t| t.contains("\n[gear]")).unwrap_or(false)
}

fn builtin(name: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    let n = name.trim_end_matches(".toml").rsplit('/').next().unwrap_or(name);
    SPECS.iter().find(|s| s.0 == n && !name.contains('/'))
}

fn load_spec_text(spec: &str) -> Result<String, String> {
    match builtin(spec) {
        Some(b) => Ok(b.1.to_string()),
        None => std::fs::read_to_string(spec).map_err(|e| format!("spec '{spec}': {e} (built-ins: {})", spec_names().join(", "))),
    }
}

fn default_iterations() -> u64 { 10_000 }
fn default_seed() -> u64 { 1 }
fn default_max_iterations() -> u64 { 200_000 }

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Variant {
    pub name: String,
    /// SimC gear lines; each replaces the base line for the same slot (or adds it).
    #[serde(default)]
    pub gear: Vec<String>,
    /// Numeric spec overrides on top of the request's, e.g. {"player.haste": 0.2}.
    #[serde(default)]
    pub set: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    /// Built-in spec name ("retribution_herald") or a path to a spec TOML.
    pub spec: String,
    /// Built-in calibration name, a path, "none", or absent for the spec's own built-in calibration.
    #[serde(default)]
    pub calib: Option<String>,
    /// The base character's /simc gear lines. Empty: use the spec's stats as written.
    #[serde(default)]
    pub base_gear: Vec<String>,
    #[serde(default)]
    pub variants: Vec<Variant>,
    #[serde(default)]
    pub set: BTreeMap<String, f64>,
    #[serde(default = "default_iterations")]
    pub iterations: u64,
    /// If set, run until every variant's paired delta (or, with no variants, the DPS)
    /// has a 95% error at or below this many DPS, up to `max_iterations`.
    #[serde(default)]
    pub target_error: Option<f64>,
    /// Same, as a percentage of the base DPS (`target_error` wins if both are set).
    #[serde(default)]
    pub target_error_pct: Option<f64>,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u64,
    #[serde(default = "default_seed")]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stat {
    pub dps: f64,
    pub dps_error_95: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantResult {
    pub name: String,
    pub dps: f64,
    pub dps_error_95: f64,
    /// DPS minus the base's, and the 95% error of that paired difference.
    pub delta: f64,
    pub delta_error_95: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub spec: String,
    pub iterations: u64,
    pub seconds: f64,
    pub base: Stat,
    pub variants: Vec<VariantResult>,
}

fn slot_of(line: &str) -> &str {
    line.split_once('=').map(|(k, _)| k.trim()).unwrap_or("")
}

/// `base` with every slot named in `over` replaced (or appended).
fn merge_gear(base: &[String], over: &[String]) -> Vec<String> {
    let mut out: Vec<String> = base
        .iter()
        .filter(|b| !over.iter().any(|o| slot_of(o) == slot_of(b)))
        .cloned()
        .collect();
    out.extend(over.iter().cloned());
    out
}

fn apply_sets(mut doc: toml::Value, sets: &[&BTreeMap<String, f64>]) -> Result<toml::Value, String> {
    for set in sets {
        for (path, v) in set.iter() {
            let parts: Vec<&str> = path.split('.').collect();
            let mut cur = &mut doc;
            for p in &parts[..parts.len() - 1] {
                cur = cur.get_mut(*p).ok_or_else(|| format!("set: no field '{p}' in {path}"))?;
            }
            cur.as_table_mut()
                .ok_or_else(|| format!("set: '{path}' is not under a table"))?
                .insert(parts[parts.len() - 1].to_string(), toml::Value::Float(*v));
        }
    }
    Ok(doc)
}

fn build(
    doc: &toml::Value,
    cal: Option<&spec::Calibration>,
    gear_lines: &[String],
    sets: &[&BTreeMap<String, f64>],
) -> Result<sim::Compiled, String> {
    // Sets go in before the gear too, so they can steer it (`gear.kit.enabled`),
    // and again after, so `player.*` overrides win over the gear's stats.
    let doc = if gear_lines.is_empty() {
        doc.clone()
    } else {
        gear::apply_text(apply_sets(doc.clone(), sets)?, &gear_lines.join("\n"))?.0
    };
    let doc = apply_sets(doc, sets)?;
    let mut sp: spec::Spec = doc.try_into().map_err(|e| format!("spec: {e}"))?;
    if let Some(c) = cal {
        sp.apply_calibration(c)?;
    }
    sim::Compiled::new(sp)
}

fn iter_dps(c: &sim::Compiled, seed: u64, i: u64) -> f64 {
    let r = sim::run_iteration(c, seed.wrapping_mul(1_000_003).wrapping_add(i), false);
    r.damage.iter().sum::<f64>() / r.length
}

/// (mean, 95% error of the mean)
fn mean_err(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    (mean, 1.96 * (var / n).sqrt())
}

pub fn run(req: &Request) -> Result<Response, String> {
    let t0 = Instant::now();
    let doc: toml::Value = {
        let text = load_spec_text(&req.spec)?;
        toml::from_str(&text).map_err(|e| format!("{}: {e}", req.spec))?
    };
    let cal: Option<spec::Calibration> = match req.calib.as_deref() {
        Some("none") => None,
        Some(c) => {
            let text = match builtin(c) {
                Some(b) => b.2.to_string(),
                None => std::fs::read_to_string(c).map_err(|e| format!("calibration '{c}': {e}"))?,
            };
            Some(toml::from_str(&text).map_err(|e| format!("calibration '{c}': {e}"))?)
        }
        None => builtin(&req.spec)
            .map(|b| toml::from_str(b.2).map_err(|e| format!("calibration: {e}")))
            .transpose()?,
    };

    let mut names = vec!["base".to_string()];
    let mut models = vec![build(&doc, cal.as_ref(), &req.base_gear, &[&req.set])?];
    for v in &req.variants {
        let gear = merge_gear(&req.base_gear, &v.gear);
        models.push(build(&doc, cal.as_ref(), &gear, &[&req.set, &v.set]).map_err(|e| format!("variant '{}': {e}", v.name))?);
        names.push(v.name.clone());
    }

    let mut dps: Vec<Vec<f64>> = vec![Vec::new(); models.len()];
    let (total, step) = match req.target_error.or(req.target_error_pct) {
        Some(_) => (req.max_iterations.max(1), 2_000u64),
        None => (req.iterations.max(2), req.iterations.max(2)),
    };
    let mut done = 0u64;
    loop {
        let n = step.min(total - done);
        for (m, out) in models.iter().zip(dps.iter_mut()) {
            let seed = req.seed;
            let chunk: Vec<f64> = (done..done + n).into_par_iter().map(|i| iter_dps(m, seed, i)).collect();
            out.extend(chunk);
        }
        done += n;
        if done >= total { break; }
        let target = req.target_error.or_else(|| req.target_error_pct.map(|p| p / 100.0 * mean_err(&dps[0]).0));
        if let Some(t) = target {
            let worst = (1..models.len())
                .map(|k| delta_stat(&dps[k], &dps[0]).1)
                .fold(if models.len() == 1 { mean_err(&dps[0]).1 } else { 0.0 }, f64::max);
            if worst <= t { break; }
        }
    }

    let (bm, be) = mean_err(&dps[0]);
    let variants = (1..models.len())
        .map(|k| {
            let (m, e) = mean_err(&dps[k]);
            let (d, de) = delta_stat(&dps[k], &dps[0]);
            VariantResult { name: names[k].clone(), dps: m, dps_error_95: e, delta: d, delta_error_95: de }
        })
        .collect();
    Ok(Response {
        spec: models[0].spec.name.clone(),
        iterations: done,
        seconds: t0.elapsed().as_secs_f64(),
        base: Stat { dps: bm, dps_error_95: be },
        variants,
    })
}

fn delta_stat(a: &[f64], b: &[f64]) -> (f64, f64) {
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| x - y).collect();
    mean_err(&d)
}

/// Per-item and total stats computed from `gear_lines` (the same report as `--gear-stats`).
pub fn gear_report(spec: &str, gear_lines: &[String]) -> Result<serde_json::Value, String> {
    let doc: toml::Value = toml::from_str(&load_spec_text(spec)?).map_err(|e| format!("{spec}: {e}"))?;
    Ok(gear::apply_text(doc, &gear_lines.join("\n"))?.1)
}

/// JSON in, JSON out (the CLI's `batch` mode and the Python binding both use this).
pub fn run_json(req: &str) -> Result<String, String> {
    let req: Request = serde_json::from_str(req).map_err(|e| format!("request: {e}"))?;
    let resp = run(&req)?;
    serde_json::to_string_pretty(&resp).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = include_str!("../profiles/MID2_Paladin_Retribution.simc");

    fn gear_lines() -> Vec<String> {
        PROFILE.lines().filter(|l| l.contains(",id=")).map(String::from).collect()
    }

    #[test]
    fn merge_replaces_by_slot() {
        let m = merge_gear(&["head=a".into(), "neck=b".into()], &["neck=c".into(), "back=d".into()]);
        assert_eq!(m, vec!["head=a", "neck=c", "back=d"]);
    }

    #[test]
    fn batch_is_paired_and_deterministic() {
        let base = gear_lines();
        // variant 1: a lower-ilvl main hand; variant 2: identical to base
        let mh = base.iter().find(|l| l.starts_with("main_hand=")).unwrap().clone();
        let worse = mh.replace("ilevel=344", "ilevel=320");
        assert_ne!(mh, worse);
        let req = Request {
            spec: "retribution_herald".into(),
            calib: None,
            base_gear: base,
            variants: vec![
                Variant { name: "no_enchant".into(), gear: vec![worse], ..Default::default() },
                Variant { name: "same".into(), ..Default::default() },
            ],
            set: Default::default(),
            iterations: 400,
            target_error: None,
            target_error_pct: None,
            max_iterations: 1000,
            seed: 7,
        };
        let a = run(&req).unwrap();
        let b = run(&req).unwrap();
        assert_eq!(a.base.dps, b.base.dps);
        assert_eq!(a.iterations, 400);
        // an identical variant shares every random number with the base
        assert_eq!(a.variants[1].delta, 0.0);
        assert_eq!(a.variants[1].delta_error_95, 0.0);
        // the paired error is far tighter than the independent errors
        assert!(a.variants[0].delta < 0.0, "{:?}", a.variants[0]);
        assert!(a.variants[0].delta_error_95 < a.base.dps_error_95);
        // a batch of one equals a standalone run of the same gear
        let mut solo = req.clone();
        solo.variants.clear();
        assert_eq!(run(&solo).unwrap().base.dps, a.base.dps);
    }

    #[test]
    fn target_error_stops_early() {
        let req = Request {
            spec: "retribution_herald".into(), calib: None, base_gear: gear_lines(),
            variants: vec![Variant { name: "same".into(), ..Default::default() }],
            set: Default::default(), iterations: 0, target_error: Some(50.0), target_error_pct: None,
            max_iterations: 20_000, seed: 1,
        };
        let r = run(&req).unwrap();
        assert_eq!(r.iterations, 2_000); // identical variant: delta error 0 after the first chunk
    }

    #[test]
    fn gear_kit_is_on_by_default_and_switchable() {
        let req = |kit: Option<f64>| Request {
            spec: "retribution_herald".into(), calib: None, base_gear: gear_lines(), variants: vec![],
            set: kit.map(|v| BTreeMap::from([("gear.kit.enabled".to_string(), v)])).unwrap_or_default(),
            iterations: 400, target_error: None, target_error_pct: None, max_iterations: 0, seed: 3,
        };
        let full = run(&req(None)).unwrap().base.dps;
        let core = run(&req(Some(0.0))).unwrap().base.dps;
        assert_eq!(run(&req(Some(1.0))).unwrap().base.dps, full);
        // consumables + Bloodlust: SimC puts this at +15.3% for this gear
        let gain = full / core - 1.0;
        assert!((0.12..0.18).contains(&gain), "kit gain {gain}");
        // the kit never touches gearless (calibration) runs
        let mut bare = req(None);
        bare.base_gear.clear();
        let mut bare_off = req(Some(0.0));
        bare_off.base_gear.clear();
        assert_eq!(run(&bare).unwrap().base.dps, run(&bare_off).unwrap().base.dps);
    }

    #[test]
    fn unknown_spec_and_gearless_spec() {
        assert!(run_json(r#"{"spec":"nope"}"#).is_err());
        for name in ["retribution_herald", "fire_mage", "destruction_hellcaller", "subtlety_deathstalker", "arms_slayer"] {
            assert!(supports_gear(name), "{name}");
        }
        // A spec without a [gear] table can't take gear lines
        let gearless = std::env::temp_dir().join("azsim_gearless_spec.toml");
        let fire = builtin("fire_mage").unwrap().1;
        std::fs::write(&gearless, fire.split("\n[gear]").next().unwrap().to_string() + "\n[fight]" + fire.split("\n[fight]").nth(1).unwrap()).unwrap();
        assert!(!supports_gear(gearless.to_str().unwrap()));
        // Fire without gear lines still sims
        let out = run_json(r#"{"spec":"fire_mage","iterations":100}"#).unwrap();
        assert!(out.contains("\"dps\""));
    }

    #[test]
    fn destruction_wither_stacks_collapse() {
        let b = builtin("destruction_hellcaller").unwrap();
        let mut s: spec::Spec = toml::from_str(b.1).unwrap();
        s.apply_calibration(&toml::from_str(b.2).unwrap()).unwrap();
        let c = sim::Compiled::new(s).unwrap();
        let r = sim::run_iteration(&c, 7, false);
        let hits = |n: &str| r.hits[c.source_names().iter().position(|s| s == n).unwrap()];
        // Chaos Bolt and Shadowburn stack Wither, which collapses into Blackened Soul hits
        assert!(hits("chaos_bolt") > 30 && hits("shadowburn") > 15);
        assert!(hits("blackened_soul") > 100, "blackened_soul hits: {}", hits("blackened_soul"));
    }

    #[test]
    fn subtlety_energy_combo_points_and_clones() {
        let b = builtin("subtlety_deathstalker").unwrap();
        let mut s: spec::Spec = toml::from_str(b.1).unwrap();
        s.apply_calibration(&toml::from_str(b.2).unwrap()).unwrap();
        let c = sim::Compiled::new(s).unwrap();
        let r = sim::run_iteration(&c, 7, false);
        let hits = |n: &str| r.hits[c.source_names().iter().position(|s| s == n).unwrap()];
        // energy regeneration and Shadow Techniques' combo points carry ~130 Eviscerates a fight
        assert!((90..170).contains(&hits("eviscerate")), "eviscerate: {}", hits("eviscerate"));
        // both hands swing, and the Mark's stacks turn auto attacks into Hunt Them Down
        assert!(hits("auto_attack_mh") > 300 && hits("auto_attack_oh") > 300);
        assert!(hits("hunt_them_down") > 400);
        // clones and copies: Ancient Arts, Shadow Blades, Goremaw's Bite
        assert!(hits("eviscerate_shadow_clone") > 20 && hits("shadow_blades_attack") > 300 && hits("goremaws_bite_finisher") > 60);
    }

    #[test]
    fn windwalker_chi_kicks_and_flurry_strikes() {
        let b = builtin("windwalker_shadopan").unwrap();
        let mut s: spec::Spec = toml::from_str(b.1).unwrap();
        s.apply_calibration(&toml::from_str(b.2).unwrap()).unwrap();
        let c = sim::Compiled::new(s).unwrap();
        let r = sim::run_iteration(&c, 7, false);
        let hits = |n: &str| r.hits[c.source_names().iter().position(|s| s == n).unwrap()];
        // chi from Tiger Palm pays for ~35 kicks, ~28 Fists of Fury and ~17 Spinning Crane Kicks
        assert!((25..50).contains(&hits("rising_sun_kick_damage")), "rising_sun_kick: {}", hits("rising_sun_kick_damage"));
        assert!((100..180).contains(&hits("fists_of_fury")), "fists_of_fury ticks: {}", hits("fists_of_fury"));
        assert!(hits("spinning_crane_kick_tick") > 40);
        // Jade Fire Stomp rides the last Fists of Fury tick
        assert!((15..40).contains(&hits("jadefire_stomp")), "jadefire_stomp: {}", hits("jadefire_stomp"));
        // Shado-Pan: auto attacks stow charges that the kicks cash as flurry strikes
        assert!(hits("flurry_strike") > 400 && hits("flurry_strike_shado_over_the_battlefield") > 400);
        assert!(hits("melee_main_hand") > 120 && hits("dual_threat") > 30);
        // Zenith's stomp and the two-charge cooldown
        assert!((3..7).contains(&hits("zenith_stomp_trigger")), "zenith_stomp: {}", hits("zenith_stomp_trigger"));
    }

    #[test]
    fn arms_rage_execute_and_bladestorm() {
        let b = builtin("arms_slayer").unwrap();
        let mut s: spec::Spec = toml::from_str(b.1).unwrap();
        s.apply_calibration(&toml::from_str(b.2).unwrap()).unwrap();
        let c = sim::Compiled::new(s).unwrap();
        let r = sim::run_iteration(&c, 7, false);
        let hits = |n: &str| r.hits[c.source_names().iter().position(|s| s == n).unwrap()];
        // rage from autos pays for ~50 Mortal Strikes; Execute comes from Sudden Death and the execute phase
        assert!((35..70).contains(&hits("mortal_strike")), "mortal_strike: {}", hits("mortal_strike"));
        assert!(hits("execute_damage") > 40 && hits("slayers_strike") > 40);
        // Bladestorm ticks 7-10 times a cast; Unhinged Mortal Strikes on the odd ticks
        assert!(hits("bladestorm_mh") > 60 && hits("mortal_strike_bladestorm_unhinged") > 30);
        // Deep Wounds rolls nearly the whole fight
        assert!(hits("deep_wounds") > 200);
    }
}
