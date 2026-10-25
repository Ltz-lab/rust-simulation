//! Gear -> character stats from embedded game tables (`data/gamedata.json.gz`).

use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::sync::OnceLock;

const GAMEDATA_GZ: &[u8] = include_bytes!("../data/gamedata.json.gz");

const SLOTS: &[&str] = &[
    "head", "neck", "shoulder", "shoulders", "back", "chest", "wrist", "wrists",
    "hands", "waist", "legs", "feet", "finger1", "finger2", "trinket1", "trinket2",
    "main_hand", "off_hand",
];

#[derive(Deserialize)]
struct GameData {
    build: String,
    items: HashMap<String, Vec<f64>>,
    statsets: Vec<Vec<[i64; 2]>>,
    bonuses: HashMap<String, Vec<[i64; 4]>>,
    gems: HashMap<String, Vec<(i64, f64, i64, usize, i64)>>,
    enchants: HashMap<String, Vec<(i64, f64, i64, usize, i64)>>,
    rand_prop_points: Vec<[f64; 5]>,
    combat_rating_mult: Vec<[f64; 4]>,
    stamina_mult: Vec<[f64; 4]>,
    weapon_dps: WeaponDps,
    spell_scaling_item: Vec<f64>,
    combat_ratings: HashMap<String, HashMap<String, f64>>,
}

#[derive(Deserialize)]
struct WeaponDps {
    one_hand: Vec<[f64; 7]>,
    two_hand: Vec<[f64; 7]>,
}

#[derive(Deserialize)]
struct GearSpec {
    primary: String,
    base_primary: f64,
    primary_mult: f64,
    attack_power_mult: f64,
    crit_bonus: f64,
    haste_mult: f64,
    mastery_points: f64,
    mastery_coeff: f64,
    versatility_bonus: f64,
    weapon_ap_per_dps: f64,
}

fn gamedata() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut dec = GzDecoder::new(GAMEDATA_GZ);
        let mut s = String::new();
        dec.read_to_string(&mut s).expect("decompress gamedata.json.gz");
        serde_json::from_str(&s).expect("parse gamedata.json.gz")
    })
}

fn stat_name(id: i64) -> Option<&'static str> {
    match id {
        3 => Some("agility"),
        4 => Some("strength"),
        5 => Some("intellect"),
        7 => Some("stamina"),
        71 => Some("str/agi/int"),
        72 => Some("str/agi"),
        73 => Some("agi/int"),
        74 => Some("str/int"),
        32 => Some("crit_rating"),
        36 => Some("haste_rating"),
        49 => Some("mastery_rating"),
        40 => Some("versatility_rating"),
        61 => Some("speed_rating"),
        62 => Some("leech_rating"),
        63 => Some("avoidance_rating"),
        _ => None,
    }
}

/// Piecewise diminishing returns on rating percent points.
fn dr_percent(p: f64) -> f64 {
    const BANDS: &[(f64, f64, f64)] = &[
        (0.0, 30.0, 1.0),
        (30.0, 39.0, 0.9),
        (39.0, 47.0, 0.8),
        (47.0, 54.0, 0.7),
        (54.0, 66.0, 0.6),
        (66.0, 126.0, 0.5),
    ];
    let mut out = 0.0;
    for &(lo, hi, mult) in BANDS {
        if p > lo {
            out += (p.min(hi) - lo) * mult;
        }
    }
    out
}

fn parse_ids(s: &str) -> Vec<i64> {
    s.split('/').filter(|x| !x.is_empty()).filter_map(|x| x.parse().ok()).collect()
}

/// Compute item stats and report JSON from SimC-format text and a `[gear]` table.
fn compute(profile: &str, cfg: &GearSpec) -> Result<(Value, [(&'static str, f64); 6]), String> {
    let gd = gamedata();
    let mut level: usize = 90;
    let mut item_lines = Vec::new();

    for raw in profile.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with("profileset.") {
            continue;
        }
        let Some((lhs, rhs)) = line.split_once('=') else { continue };
        let key = lhs.trim();
        let rhs = rhs.trim();
        if key == "level" {
            if let Ok(lv) = rhs.parse() {
                level = lv;
            }
            continue;
        }
        if SLOTS.contains(&key) {
            item_lines.push((key, rhs));
        }
    }

    let mut items_out = Vec::new();
    let mut totals: HashMap<&'static str, i64> = HashMap::new();
    let mut weapon_dps = 0.0;

    for (slot, rhs) in item_lines {
        let mut id: Option<i64> = None;
        let mut bonus_ids = Vec::new();
        let mut gem_ids = Vec::new();
        let mut enchant_id: Option<i64> = None;
        let mut redirected: Option<i64> = None;
        let mut crafted_stats = Vec::new();
        let mut ilevel_override: Option<usize> = None;

        for part in rhs.split(',').skip(1) {
            let Some((k, v)) = part.split_once('=') else { continue };
            match k.trim() {
                "id" => id = v.trim().parse().ok(),
                "bonus_id" => bonus_ids = parse_ids(v.trim()),
                "gem_id" => gem_ids = parse_ids(v.trim()),
                "enchant_id" => enchant_id = v.trim().parse().ok(),
                "redirected_base_stats" => redirected = v.trim().parse().ok(),
                "crafted_stats" => crafted_stats = parse_ids(v.trim()),
                "ilevel" => ilevel_override = v.trim().parse().ok(),
                _ => {}
            }
        }

        let Some(item_id) = id.filter(|&x| x > 0) else { continue };
        let row = gd.items.get(&item_id.to_string()).ok_or_else(|| {
            format!("slot '{slot}': unknown item id {item_id}")
        })?;
        let inv_type = row[0] as i64;
        let base_ilvl = row[1] as i64;
        let quality = row[2] as usize;
        let mut statset_idx = row[3] as usize;
        let subclass = row.get(5).map(|&x| x as i64).unwrap_or(-1);

        if let Some(redir_id) = redirected {
            let rrow = gd.items.get(&redir_id.to_string()).ok_or_else(|| {
                format!("slot '{slot}': unknown redirected_base_stats item id {redir_id}")
            })?;
            statset_idx = rrow[3] as usize;
        }

        // 1. Item level
        let mut add = 0i64;
        let mut post = 0i64;
        let mut scaled: Option<(i64, i64)> = None;
        let mut bonus_stats = Vec::new();
        let mut bonus_crafted = Vec::new();

        for bid in &bonus_ids {
            if let Some(entries) = gd.bonuses.get(&bid.to_string()) {
                for &[t, v0, v1, _] in entries {
                    match t {
                        1 => add += v0,
                        2 => bonus_stats.push([v0, v1]),
                        25 => bonus_crafted.push(v0),
                        49 => {
                            if scaled.is_none_or(|(_, prio)| v1 < prio) {
                                scaled = Some((v0, v1));
                            }
                        }
                        52 => post += v0,
                        _ => {}
                    }
                }
            }
        }

        let ilvl = ilevel_override.unwrap_or_else(|| {
            let base = scaled.map(|(v, _)| v).unwrap_or(base_ilvl + add);
            (base + post).max(0) as usize
        });

        // 2 & 3. Stat list and crafted stat replacement
        let mut raw_stats = gd.statsets[statset_idx].clone();
        raw_stats.extend(bonus_stats);
        let crafted = if !bonus_crafted.is_empty() { &bonus_crafted } else { &crafted_stats };

        let mut resolved_stats = Vec::with_capacity(raw_stats.len());
        for [sid, alloc] in raw_stats {
            let real_sid = match sid {
                24 => match crafted.first() {
                    Some(&s) => s,
                    None => continue,
                },
                25 => match crafted.get(1) {
                    Some(&s) => s,
                    None => continue,
                },
                s => s,
            };
            resolved_stats.push([real_sid, alloc]);
        }

        // 4. Slot type & budget
        let slot_type = match inv_type {
            1 | 5 | 20 | 7 | 17 | 15 => 0,
            26 => if subclass == 19 { 4 } else { 0 },
            3 | 6 | 8 | 10 | 12 => 1,
            2 | 9 | 11 | 16 => 2,
            13 | 14 | 21 | 22 | 23 => 3,
            _ => 0,
        };
        if ilvl >= gd.rand_prop_points.len() {
            return Err(format!("slot '{slot}': item level {ilvl} out of range"));
        }
        let budget = gd.rand_prop_points[ilvl][slot_type];

        // 5. Stat values
        let col = match inv_type {
            2 | 11 => 3,
            12 => 2,
            13 | 14 | 15 | 17 | 21 | 22 | 23 | 26 => 1,
            _ => 0,
        };

        let mut item_stats: BTreeMap<&'static str, i64> = BTreeMap::new();
        for [sid, alloc] in resolved_stats {
            let Some(name) = stat_name(sid) else { continue };
            let mult = match sid {
                32 | 36 | 49 | 40 | 61 | 62 | 63 => gd.combat_rating_mult[ilvl][col],
                7 => gd.stamina_mult[ilvl][col],
                _ => 1.0,
            };
            let val = (alloc as f64 * 0.0001 * budget * mult).round() as i64;
            *item_stats.entry(name).or_insert(0) += val;
        }

        // 6. Gems & enchants
        let eval_effect = |&(sid, coeff, scaling_class, max_level, points): &(i64, f64, i64, usize, i64)| -> Option<(&'static str, i64)> {
            let name = stat_name(sid)?;
            let val = if scaling_class < 0 {
                let lvl = if max_level > 0 { level.min(max_level) } else { level };
                (coeff * gd.spell_scaling_item[lvl]).round() as i64
            } else {
                points
            };
            Some((name, val))
        };

        for gid in gem_ids {
            if gid > 0
                && let Some(effects) = gd.gems.get(&gid.to_string())
            {
                for eff in effects {
                    if let Some((name, val)) = eval_effect(eff) {
                        *item_stats.entry(name).or_insert(0) += val;
                    }
                }
            }
        }
        if let Some(eid) = enchant_id.filter(|&x| x > 0)
            && let Some(effects) = gd.enchants.get(&eid.to_string())
        {
            for eff in effects {
                if let Some((name, val)) = eval_effect(eff) {
                    *item_stats.entry(name).or_insert(0) += val;
                }
            }
        }

        for (&k, &v) in &item_stats {
            *totals.entry(k).or_insert(0) += v;
        }

        // 8. Weapon DPS (main_hand only)
        if slot == "main_hand" {
            weapon_dps = match inv_type {
                13 | 21 | 22 => gd.weapon_dps.one_hand[ilvl][quality],
                17 | 15 => gd.weapon_dps.two_hand[ilvl][quality],
                26 if subclass != 19 => gd.weapon_dps.two_hand[ilvl][quality],
                _ => 0.0,
            };
        }

        items_out.push(json!({
            "slot": slot,
            "id": item_id,
            "ilvl": ilvl,
            "stats": item_stats,
        }));
    }

    let get = |k: &str| *totals.get(k).unwrap_or(&0);
    let strength = get("strength") + get("str/agi/int") + get("str/agi") + get("str/int");
    let agility = get("agility") + get("str/agi/int") + get("str/agi") + get("agi/int");
    let intellect = get("intellect") + get("str/agi/int") + get("agi/int") + get("str/int");
    let stamina = get("stamina");
    let crit_rating = get("crit_rating");
    let haste_rating = get("haste_rating");
    let mastery_rating = get("mastery_rating");
    let versatility_rating = get("versatility_rating");
    let speed_rating = get("speed_rating");
    let leech_rating = get("leech_rating");
    let avoidance_rating = get("avoidance_rating");

    let gear_primary = match cfg.primary.as_str() {
        "strength" => strength,
        "agility" => agility,
        "intellect" => intellect,
        other => return Err(format!("unknown primary stat '{other}'")),
    } as f64;

    let cr = gd.combat_ratings.get(&level.to_string()).ok_or_else(|| {
        format!("no combat_ratings entry for level {level}")
    })?;

    let crit_dr = dr_percent(crit_rating as f64 / cr["crit"]);
    let haste_dr = dr_percent(haste_rating as f64 / cr["haste"]);
    let mastery_dr = dr_percent(mastery_rating as f64 / cr["mastery"]);
    let vers_dr = dr_percent(versatility_rating as f64 / cr["versatility"]);

    let primary = ((cfg.base_primary + gear_primary) * cfg.primary_mult).floor();
    let attack_power = (primary * cfg.attack_power_mult).floor();
    let weapon_attack_power = weapon_dps * cfg.weapon_ap_per_dps;
    let crit = crit_dr / 100.0 + cfg.crit_bonus;
    let haste = (1.0 + haste_dr / 100.0) * cfg.haste_mult - 1.0;
    let mastery = (mastery_dr + cfg.mastery_points) * cfg.mastery_coeff / 100.0;
    let versatility = vers_dr / 100.0 + cfg.versatility_bonus;

    let player_pairs = [
        ("attack_power", attack_power),
        ("weapon_attack_power", weapon_attack_power),
        ("crit", crit),
        ("haste", haste),
        ("mastery", mastery),
        ("versatility", versatility),
    ];

    let report = json!({
        "build": gd.build,
        "level": level,
        "items": items_out,
        "gear": {
            "strength": strength,
            "agility": agility,
            "intellect": intellect,
            "stamina": stamina,
            "crit_rating": crit_rating,
            "haste_rating": haste_rating,
            "mastery_rating": mastery_rating,
            "versatility_rating": versatility_rating,
            "speed_rating": speed_rating,
            "leech_rating": leech_rating,
            "avoidance_rating": avoidance_rating,
            "weapon_dps": weapon_dps,
        },
        "player": {
            "attack_power": attack_power,
            "weapon_attack_power": weapon_attack_power,
            "crit": crit,
            "haste": haste,
            "mastery": mastery,
            "versatility": versatility,
        },
    });

    Ok((report, player_pairs))
}

/// Parse a SimC gear profile, overwrite `player.*` stats in `doc`, and return
/// both the updated TOML document and the JSON breakdown report.
pub fn apply(mut doc: toml::Value, path: &str) -> Result<(toml::Value, Value), String> {
    let gear_tbl = doc.get("gear").cloned().ok_or_else(|| {
        "spec has no [gear] table".to_string()
    })?;
    let cfg: GearSpec = gear_tbl.try_into().map_err(|e| format!("[gear]: {e}"))?;
    let profile = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let (report, player_pairs) = compute(&profile, &cfg)?;

    let player = doc
        .get_mut("player")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| "spec has no [player] table".to_string())?;
    for (k, v) in player_pairs {
        player.insert(k.to_string(), toml::Value::Float(v));
    }
    Ok((doc, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RET_GEAR_TOML: &str = r#"
        primary = "strength"
        base_primary = 622
        primary_mult = 1.092
        attack_power_mult = 1.05
        crit_bonus = 0.09
        haste_mult = 1.02
        mastery_points = 14
        mastery_coeff = 1.35
        versatility_bonus = 0.03
        weapon_ap_per_dps = 6.7909
    "#;

    const RET_CORE_PROFILE: &str = "\
level=90
head=warhelm_of_the_consecrated_flame,id=271465,bonus_id=12854/13692/13698/13750,gem_id=240983,redirected_base_stats=268229
neck=aqirbane_reliquary,id=268265,bonus_id=4786/13750/13848/13987,ilevel=344,gem_id=240892/240892
shoulders=pauldrons_of_the_consecrated_flame,id=271463,bonus_id=12854/13694/13697,redirected_base_stats=251138
back=silken_voodoo_drape,id=268253,bonus_id=13662/13848
chest=bulwark_of_the_consecrated_flame,id=271468,bonus_id=13690/13698/13848,enchant_id=7987,redirected_base_stats=268222
wrists=spellbreakers_bracers,id=237834,bonus_id=8790/8960/13454/13750/13751/13836/9627,gem_id=240892,crafted_stats=32/49
hands=gauntlets_of_the_consecrated_flame,id=271466,bonus_id=12854/13691/13697,ilevel=334,redirected_base_stats=251214
waist=girdle_of_toxic_regret,id=268259,bonus_id=13668,ilevel=344,gem_id=240892
legs=greaves_of_the_consecrated_flame,id=271464,bonus_id=13693/13698/13708/13848,enchant_id=8159,redirected_base_stats=271878
feet=scaled_fiends_warboots,id=268260,ilevel=334,enchant_id=7963
finger1=vile_alchemists_band,id=268249,bonus_id=1808/13668,ilevel=334,gem_id=240892,enchant_id=7967
finger2=loa_worshipers_band,id=251513,bonus_id=8960/12214/13751/13836/9627,gem_id=240892,enchant_id=7967
main_hand=mazeroa_warlords_fury,id=268213,ilevel=344,enchant_id=8689
";

    #[test]
    fn ret_paladin_items_and_totals() {
        let cfg: GearSpec = toml::from_str(RET_GEAR_TOML).unwrap();
        let (report, _) = compute(RET_CORE_PROFILE, &cfg).unwrap();
        let items = report["items"].as_array().unwrap();

        let by_slot = |s: &str| items.iter().find(|it| it["slot"] == s).unwrap();

        let head = by_slot("head");
        assert_eq!(head["ilvl"], 334);
        assert_eq!(head["stats"]["crit_rating"], 61);
        assert_eq!(head["stats"]["haste_rating"], 140);
        assert_eq!(head["stats"]["str/int"], 189);
        assert_eq!(head["stats"]["stamina"], 3910);
        assert_eq!(head["stats"]["str/agi/int"], 32); // gem 240983

        let neck = by_slot("neck");
        assert_eq!(neck["ilvl"], 344);
        assert_eq!(neck["stats"]["crit_rating"], 107);
        assert_eq!(neck["stats"]["haste_rating"], 107 + 2 * 16); // two 240892 gems
        assert_eq!(neck["stats"]["mastery_rating"], 107 + 2 * 7);
        assert_eq!(neck["stats"]["versatility_rating"], 107);

        let wrists = by_slot("wrists");
        assert_eq!(wrists["ilvl"], 331);
        assert_eq!(wrists["stats"]["crit_rating"], 56);
        // bonus 8790 assigns crit/haste, overriding crafted_stats=32/49
        assert_eq!(wrists["stats"]["haste_rating"], 56 + 16);
        assert_eq!(wrists["stats"]["mastery_rating"], 7);
        assert_eq!(wrists["stats"]["str/int"], 103);

        let finger1 = by_slot("finger1");
        assert_eq!(finger1["ilvl"], 334);
        assert_eq!(finger1["stats"]["crit_rating"], 73);
        assert_eq!(finger1["stats"]["mastery_rating"], 332 + 7);

        let mh = by_slot("main_hand");
        assert_eq!(mh["ilvl"], 344);
        assert_eq!(mh["stats"]["crit_rating"], 148);
        assert_eq!(mh["stats"]["mastery_rating"], 61);
        assert_eq!(mh["stats"]["str/agi"], 207);

        let gear = &report["gear"];
        assert_eq!(gear["crit_rating"], 1051);
        assert_eq!(gear["haste_rating"], 816);
        assert_eq!(gear["mastery_rating"], 1062);
        assert_eq!(gear["versatility_rating"], 107);
        assert_eq!(gear["strength"], 1730);
        assert!((gear["weapon_dps"].as_f64().unwrap() - 155.80).abs() < 1e-2);

        let player = &report["player"];
        assert_eq!(player["attack_power"].as_f64().unwrap(), 2696.0);
        assert!((player["weapon_attack_power"].as_f64().unwrap() - 1058.0).abs() < 1.0);
        assert!((player["crit"].as_f64().unwrap() - 0.318478).abs() < 1e-4);
        assert!((player["haste"].as_f64().unwrap() - 0.209164).abs() < 1e-4);
        assert!((player["mastery"].as_f64().unwrap() - 0.500674).abs() < 1e-4);
        assert!((player["versatility"].as_f64().unwrap() - 0.049815).abs() < 1e-4);

        // With trinkets
        let with_trinkets = format!(
            "{RET_CORE_PROFILE}\n\
             trinket1=zuljins_guillotine_technique,id=270173,bonus_id=13335,ilevel=344\n\
             trinket2=voracious_heart_of_ulatek,id=270175,bonus_id=13335,ilevel=344\n"
        );
        let (rep_trink, _) = compute(&with_trinkets, &cfg).unwrap();
        assert_eq!(rep_trink["gear"]["crit_rating"], 1200);
        assert_eq!(rep_trink["gear"]["strength"], 1927);
    }
}
