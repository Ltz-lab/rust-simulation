
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

/// `azsim overlay <overlay.toml> [--demo secs] [--targets n] [--haste h] [--start fury]
///  [--replay events.jsonl|-] [--next n] [--player guid] [--feed] [--audit]`: run a live-rotation overlay (src/overlay.rs) on a
/// stand-in fight, or on combat-log events (one JSON object a line), printing its picks.
fn overlay_main(mut it: impl Iterator<Item = String>) {
    use azsim::overlay::{Demo, LogEvent, Overlay, Tracker};
    let (mut path, mut demo, mut targets, mut haste, mut start, mut replay, mut next) =
        (None, 40.0, 1usize, None, 40.0, None, 3usize);
    let mut player: Option<String> = None;
    let mut feed = false;
    let mut audit = false;
    while let Some(x) = it.next() {
        let mut v = || it.next().expect("missing value");
        match x.as_str() {
            "--demo" => demo = v().parse().unwrap(),
            "--targets" => targets = v().parse().unwrap(),
            "--haste" => haste = Some(v().parse::<f64>().unwrap()),
            "--start" => start = v().parse().unwrap(),
            "--replay" => replay = Some(v()),
            "--next" => next = v().parse().unwrap(),
            "--player" => player = Some(v()),
            "--feed" => feed = true,
            "--audit" => audit = true,
            _ => path = Some(x),
        }
    }
    let Some(path) = path else {
        eprintln!("usage: azsim overlay <overlay.toml> [--demo secs] [--targets n] [--haste h] [--start fury] [--replay events.jsonl|-] [--next n] [--player guid] [--feed] [--audit]");
        std::process::exit(2);
    };
    let mut o = Overlay::load(&path).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(1) });
    if let Some(h) = haste { o.spec.haste = h; }
    if audit {
        // combatfeed lines in; for each of the player's presses of a listed spell, what the
        // overlay said just before it and whether it thought the spell was usable.
        use std::io::BufRead;
        let mut tr = Tracker::new(&o);
        let (mut n, mut first, mut inq, mut early) = (0, 0, 0, 0);
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
            let press = (v["type"] == "event" && v["sub"] == "SPELL_CAST_SUCCESS" && v["mine"] == true)
                .then(|| v["spell"].as_i64().and_then(|id| o.spell_by_id(id))).flatten();
            if let Some(i) = press {
                tr.advance(v["t"].as_f64().unwrap_or(0.0) / 1000.0);
                let name = o.spell_name(i).to_string();
                let q: Vec<String> = tr.recommend(next).into_iter().filter(|p| o.spec.spells[o.spell_index(&p.action).unwrap()].gcd != Some(0.0)).map(|p| p.action).collect();
                let cd = tr.cooldown_remains(&name);
                n += 1;
                if q.first() == Some(&name) { first += 1; } else if q.contains(&name) { inq += 1; }
                if cd > 0.05 { early += 1; }
                println!("{:>8.2}  {:<16} fury {:>4.0}  haste {:>4.0}%  own cd {:>5.2}  said: {}", tr.now, name, tr.resource,
                    tr.haste_at(tr.now) * 100.0, cd, q.join(" < "));
            }
            tr.feed_line(&v);
        }
        println!("{n} presses: {first} were queue[0], {inq} elsewhere in the queue; {early} pressed while the overlay thought it was on cooldown");
        return;
    }
    if feed {
        // combatfeed | azsim overlay <file> --feed | auraoverlay: pass every line through and add
        // {"type":"next",...} whenever the suggestion changes.
        use std::io::{BufRead, Write};
        let mut tr = Tracker::new(&o);
        let (stdin, stdout) = (std::io::stdin(), std::io::stdout());
        let mut out = stdout.lock();
        let mut last = String::new();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if writeln!(out, "{line}").is_err() { return; }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                tr.feed_line(&v);
                if let Some(n) = tr.next_line(next) {
                    let key = n["queue"].to_string();
                    if key != last {
                        last = key;
                        if writeln!(out, "{n}").is_err() { return; }
                    }
                }
            }
            if out.flush().is_err() { return; }
        }
        return;
    }
    if let Some(src) = replay {
        let text = if src == "-" {
            let mut s = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).expect("read stdin");
            s
        } else { std::fs::read_to_string(&src).expect("read events") };
        let mut tr = Tracker::new(&o);
        tr.player = player;
        let mut last: Vec<String> = vec![];
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let ev: LogEvent = serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"));
            tr.ingest(&ev);
            let picks: Vec<String> = tr.recommend(next).into_iter().map(|p| p.action).collect();
            if picks != last {
                println!("{}", json!({ "t": tr.now, "fury": tr.resource, "enemies": tr.active_enemies(), "next": picks }));
                last = picks;
            }
        }
        return;
    }
    let extra: Vec<(&str, &str, f64, f64)> = o.spec.name.contains("Havoc").then(|| vec![
        // what the game does on its own after these casts (Havoc): Eternal Hunt after Eye Beam's
        // last tick, the Demonsurge stacking buff with each Metamorphosis entry
        ("eye_beam", "eternal_hunt", 2.0, 8.0), ("abyssal_gaze", "eternal_hunt", 2.0, 8.0),
        ("eye_beam", "demonsurge", 0.7, 15.0), ("metamorphosis", "demonsurge", 0.7, 15.0),
    ]).unwrap_or_default();
    let d = Demo::run(&o, demo, targets, start, 1.3, 15.0, &extra);
    println!("{} — {:.0}s, {} target(s), haste {:.0}%", o.spec.name, demo, targets, o.spec.haste * 100.0);
    println!("{:>7}  {:>5}  {:<18} rule", "time", "fury", "press");
    for (t, a, f, r) in &d.presses {
        let cond = o.spec.rotation[*r].cond.as_deref().unwrap_or("");
        println!("{t:>7.2}  {f:>5.0}  {a:<18} {r:>2}  {cond}");
    }
    let mut counts: Vec<(String, usize)> = o.spec.spells.iter().map(|s| (s.name.clone(), d.count(&s.name))).filter(|c| c.1 > 0).collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{}", counts.iter().map(|(n, c)| format!("{n} {c}")).collect::<Vec<_>>().join(", "));
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("batch") {
        return batch_main(std::env::args().skip(2));
    }
    if std::env::args().nth(1).as_deref() == Some("overlay") {
        return overlay_main(std::env::args().skip(2));
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
