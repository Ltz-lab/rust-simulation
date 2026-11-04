
use azsim::{gear, sim, spec};
use rayon::prelude::*;
use serde_json::json;
use std::time::Instant;

struct Args {
    spec: String,
    iterations: u64,
    seed: u64,
    json: Option<String>,
    log: bool,
    set: Vec<(String, f64)>,
    calib: Option<String>,
    gear: Option<String>,
    gear_stats: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        spec: String::new(), iterations: 5000, seed: 1, json: None, log: false,
        set: vec![], calib: None, gear: None, gear_stats: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "-n" | "--iterations" => a.iterations = it.next().unwrap().parse().unwrap(),
            "--seed" => a.seed = it.next().unwrap().parse().unwrap(),
            "--json" => a.json = it.next(),
            "--log" => a.log = true,
            "--calib" => a.calib = it.next(),
            "--gear" => a.gear = it.next(),
            "--gear-stats" => a.gear_stats = true,
            "--set" => {
                // --set player.haste=0.2  (override a numeric field for stat weights etc.)
                let kv = it.next().unwrap();
                let (k, v) = kv.split_once('=').expect("--set key=value");
                a.set.push((k.to_string(), v.parse().unwrap()));
            }
            _ => a.spec = x,
        }
    }
    if a.spec.is_empty() {
        eprintln!("usage: azsim <spec.toml> [-n iterations] [--seed n] [--json out.json] [--log] [--set path=value] [--calib calibration.toml] [--gear profile.simc] [--gear-stats]");
        std::process::exit(2);
    }
    a
}

/// Apply `--set a.b.c=value` overrides to the raw TOML before deserializing.
fn apply_overrides(mut doc: toml::Value, sets: &[(String, f64)]) -> toml::Value {
    for (path, v) in sets {
        let mut cur = &mut doc;
        let parts: Vec<&str> = path.split('.').collect();
        for p in &parts[..parts.len() - 1] {
            cur = cur.get_mut(*p).unwrap_or_else(|| panic!("--set: no field '{p}' in {path}"));
        }
        cur.as_table_mut().unwrap().insert(parts[parts.len() - 1].to_string(), toml::Value::Float(*v));
    }
    doc
}

/// `azsim batch <request.json|-> [--out result.json]`: JSON in, JSON out.
fn batch_main(mut it: impl Iterator<Item = String>) {
    let (mut input, mut out) = (None, None);
    while let Some(x) = it.next() {
        if x == "--out" { out = it.next(); } else { input = Some(x); }
    }
    let Some(input) = input else {
        eprintln!("usage: azsim batch <request.json | -> [--out result.json]");
        std::process::exit(2);
    };
    let text = if input == "-" {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).expect("read stdin");
        s
    } else {
        std::fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("{input}: {e}"); std::process::exit(1) })
    };
    match azsim::api::run_json(&text) {
        Ok(res) => match out {
            Some(p) => std::fs::write(p, res).expect("write result"),
            None => println!("{res}"),
        },
        Err(e) => { eprintln!("batch error: {e}"); std::process::exit(1); }
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("batch") {
        return batch_main(std::env::args().skip(2));
    }
    let args = parse_args();
    let text = std::fs::read_to_string(&args.spec).expect("read spec");
    let mut doc: toml::Value = toml::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", args.spec));
    if let Some(path) = &args.gear {
        // overrides first too, so they can steer the gear (`gear.kit.enabled=0`)
        let (new_doc, report) = gear::apply(apply_overrides(doc, &args.set), path).unwrap_or_else(|e| {
            eprintln!("gear error: {e}");
            std::process::exit(1);
        });
        if args.gear_stats {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            return;
        }
        doc = new_doc;
    }
    let doc = apply_overrides(doc, &args.set);
    let mut spec: spec::Spec = doc.try_into().unwrap_or_else(|e| panic!("{}: {e}", args.spec));
    if let Some(path) = &args.calib {
        let text = std::fs::read_to_string(path).expect("read calibration");
        let cal: spec::Calibration = toml::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        spec.apply_calibration(&cal).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
    }
    let compiled = sim::Compiled::new(spec).unwrap_or_else(|e| {
        eprintln!("spec error: {e}");
        std::process::exit(1);
    });

    if args.log {
        let r = sim::run_iteration(&compiled, args.seed, true);
        println!("fight length {:.1}s, dps {:.0}", r.length, r.damage.iter().sum::<f64>() / r.length);
        return;
    }

    let names = compiled.source_names().to_vec();
    let n = names.len();
    let t0 = Instant::now();
    // Per iteration: dps, and per-source damage/casts/hits/crits plus length for averaging.
    let results: Vec<(f64, sim::IterResult)> = (0..args.iterations)
        .into_par_iter()
        .map(|i| {
            let r = sim::run_iteration(&compiled, args.seed.wrapping_mul(1_000_003).wrapping_add(i), false);
            (r.damage.iter().sum::<f64>() / r.length, r)
        })
        .collect();
    let elapsed = t0.elapsed().as_secs_f64();

    let iters = results.len() as f64;
    let mean = results.iter().map(|r| r.0).sum::<f64>() / iters;
    let var = results.iter().map(|r| (r.0 - mean).powi(2)).sum::<f64>() / (iters - 1.0).max(1.0);
    let stderr = (var / iters).sqrt();
    let total_len: f64 = results.iter().map(|r| r.1.length).sum();

    let mut rows = Vec::new();
    for s in 0..n {
        let dmg: f64 = results.iter().map(|r| r.1.damage[s]).sum();
        let casts: u64 = results.iter().map(|r| r.1.casts[s]).sum();
        let hits: u64 = results.iter().map(|r| r.1.hits[s]).sum();
        let crits: u64 = results.iter().map(|r| r.1.crits[s]).sum();
        let noncrit: f64 = results.iter().map(|r| r.1.noncrit_damage[s]).sum();
        let rgain: f64 = results.iter().map(|r| r.1.res_gain[s]).sum();
        let rwaste: f64 = results.iter().map(|r| r.1.res_waste[s]).sum();
        if casts == 0 && hits == 0 && rgain == 0.0 { continue; }
        rows.push(json!({
            "name": names[s],
            "dps": dmg / total_len,
            "casts_per_fight": casts as f64 / iters,
            "hits_per_fight": hits as f64 / iters,
            "avg_hit": if hits > 0 { dmg / hits as f64 } else { 0.0 },
            "crit_rate": if hits > 0 { crits as f64 / hits as f64 } else { 0.0 },
            "avg_noncrit": if hits > crits { noncrit / (hits - crits) as f64 } else { 0.0 },
            "resource_gain_per_fight": rgain / iters,
            "resource_waste_per_fight": rwaste / iters,
        }));
    }
    rows.sort_by(|a, b| b["dps"].as_f64().partial_cmp(&a["dps"].as_f64()).unwrap());

    if let Some(path) = &args.json {
        let out = json!({
            "spec": compiled.spec.name,
            "iterations": args.iterations,
            "dps": mean,
            "dps_error_95": 1.96 * stderr,
            "seconds": elapsed,
            "sources": rows,
        });
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap()).expect("write json");
    }

    // stdout may be a closed pipe (e.g. `| head`): ignore write errors
    let mut o = std::io::stdout().lock();
    use std::io::Write;
    let _ = writeln!(o, "{}  |  {} iterations in {:.2}s", compiled.spec.name, args.iterations, elapsed);
    let _ = writeln!(o, "DPS: {:.0}  (+/- {:.0} at 95%)\n", mean, 1.96 * stderr);
    let _ = writeln!(o, "{:<34} {:>9} {:>7} {:>7} {:>10} {:>6}", "source", "dps", "share", "casts", "avg hit", "crit");
    for r in &rows {
        let _ = writeln!(
            o,
            "{:<34} {:>9.0} {:>6.1}% {:>7.1} {:>10.0} {:>5.1}%",
            r["name"].as_str().unwrap(),
            r["dps"].as_f64().unwrap(),
            100.0 * r["dps"].as_f64().unwrap() / mean,
            r["casts_per_fight"].as_f64().unwrap(),
            r["avg_hit"].as_f64().unwrap(),
            100.0 * r["crit_rate"].as_f64().unwrap()
        );
    }

}
