//! Gear -> character stats from embedded game tables (`data/gamedata.json.gz`),
//! plus what trinkets and set bonuses do: spec fragments from effect
//! templates (`data/item_effects.toml`, a spec's `[[gear.effects]]`) with
//! their numbers read from the game data at the item's level.

use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::sync::OnceLock;

const GAMEDATA_GZ: &[u8] = include_bytes!("../data/gamedata.json.gz");
const ITEM_EFFECTS: &str = include_str!("../data/item_effects.toml");

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
    rand_prop_damage: Vec<[f64; 2]>,
    item_effects: HashMap<String, Vec<[i64; 4]>>,
    /// item bonus id -> effects it adds to its item (embellishments)
    #[serde(default)]
    bonus_effects: HashMap<String, Vec<[i64; 4]>>,
    item_sets: HashMap<String, i64>,
    set_bonuses: HashMap<String, Vec<[i64; 3]>>,
    spells: HashMap<String, SpellRec>,
}

/// One spell's numbers, as `tools/gamedata.py` keeps them.
#[derive(Deserialize)]
struct SpellRec {
    duration: i64,
    rppm: f64,
    rppm_mods: Vec<(i64, i64, f64)>,
    icd: i64,
    chance: f64,
    stacks: i64,
    cooldown: i64,
    /// [type, aura, base_points, coeff, scaling_class, ap_coeff, sp_coeff, misc0, trigger]
    effects: Vec<(i64, i64, f64, f64, i64, f64, f64, i64, i64)>,
}

/// What an item effect or set bonus does: a spec fragment (`text`, TOML with
/// `${...}` placeholders), or `none` when another entry already covers it.
#[derive(Deserialize, Clone)]
struct EffectTpl {
    spell: i64,
    #[serde(default)]
    none: bool,
    #[serde(default)]
    text: String,
    /// Only with a socketed gem of this colour somewhere in the gear
    /// ("garnet", "peridot", "amethyst" or "lapis"); several entries may share
    /// a spell, each adding its fragment when its colour is present.
    #[serde(default)]
    gem: Option<String>,
}

#[derive(Deserialize)]
struct Library {
    effect: Vec<EffectTpl>,
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
    /// Attack power per point of the primary stat (melee); omit for casters.
    #[serde(default)]
    attack_power_mult: Option<f64>,
    /// Spell power per point of the primary stat (casters); omit for melee.
    #[serde(default)]
    spell_power_per_primary: Option<f64>,
    crit_bonus: f64,
    haste_mult: f64,
    mastery_points: f64,
    mastery_coeff: f64,
    versatility_bonus: f64,
    #[serde(default)]
    weapon_ap_per_dps: f64,
    /// Passive "rating multiplier" auras (aura 405, e.g. Frost Mage's Winter's
    /// Blessing: +5% haste rating), by "crit" / "haste" / "mastery" / "versatility".
    /// Applied to the gear's rating before the kit's consumables are added.
    #[serde(default)]
    rating_mult: HashMap<String, f64>,
    #[serde(default)]
    kit: Option<Kit>,
    /// The spec's id (ChrSpecialization), to pick class set bonuses.
    #[serde(default)]
    spec_id: i64,
    /// Spec-specific effect templates (class set bonuses); they win over the library's.
    #[serde(default)]
    effects: Vec<EffectTpl>,
    /// Rotation conditions for on-use items by kind ("buff": a stat buff,
    /// "other"); placeholders refer to the item's own spell.
    #[serde(default)]
    use_items: HashMap<String, String>,
    /// On-use items go right after this rotation action (else first).
    #[serde(default)]
    use_after: Option<String>,
    /// Damage multipliers for the library's proc damage by "physical" /
    /// "magic": what the spec's calibrated sources carry in their fitted
    /// multipliers but a new, uncalibrated source would miss (raid debuffs).
    #[serde(default)]
    effect_mult: HashMap<String, f64>,
    /// 0 turns item effects (trinkets, rings, weapons...) / set bonuses off (stats stay).
    #[serde(default = "one")]
    trinket_effects: f64,
    #[serde(default = "one")]
    set_bonuses: f64,
    /// Time-averaged gear stat procs, added on top of the buffed stats.
    #[serde(default)]
    procs: ProcBonus,
    /// Omnium traits: which the calibration baked in, and what each is worth.
    #[serde(default)]
    omnium: Option<Omnium>,
}

/// `[gear.omnium]`: the profile's `omnium_talents=` traits as per-character
/// damage. `baseline` lists the traits the calibration reference had on (their
/// effect is inside the fitted multipliers); each `traits` entry gives one
/// trait's `global_mult`. A run with gear lines multiplies `[player].global_mult`
/// by (product over the character's traits) / (product over the baseline),
/// so the reference character is unchanged and one without a trait loses it.
#[derive(Deserialize, Default)]
struct Omnium {
    /// Trait ids as in the profile (`136822` for `136822:1`, or a rune name).
    #[serde(default)]
    baseline: Vec<String>,
    #[serde(default)]
    traits: Vec<OmniumTrait>,
}

#[derive(Deserialize)]
struct OmniumTrait {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default = "one")]
    global_mult: f64,
}

/// The traits of a profile's `omnium_talents=` line: ids without their rank
/// (`136822:1/136816:1` or `rune_of_overload/...`); none for a missing line.
fn omnium_traits(profile: &str) -> Vec<String> {
    for raw in profile.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if let Some(("omnium_talents", rhs)) = line.split_once('=').map(|(l, r)| (l.trim(), r.trim())) {
            return rhs
                .split('/')
                .map(|t| t.split(':').next().unwrap_or("").trim().to_string())
                .filter(|t| !t.is_empty() && t != "none")
                .collect();
        }
    }
    vec![]
}

/// Scale `[player].global_mult` for the traits, and report each trait's status.
fn apply_omnium(doc: &mut toml::Value, profile: &str, cfg: &GearSpec) -> Result<Value, String> {
    let have = omnium_traits(profile);
    let Some(om) = &cfg.omnium else {
        return Ok(json!({ "traits": have, "baseline": [], "status": "not configured", "factor": 1.0 }));
    };
    let value = |id: &str| om.traits.iter().find(|t| t.id == id).map(|t| t.global_mult);
    let mut rows = Vec::new();
    let mut factor = 1.0;
    for id in &have {
        let status = match value(id) {
            Some(m) => { factor *= m; "modelled" }
            None => "unmodelled",
        };
        let name = om.traits.iter().find(|t| &t.id == id).map(|t| t.name.clone()).unwrap_or_default();
        rows.push(json!({ "id": id, "name": name, "status": status, "in_baseline": om.baseline.contains(id) }));
    }
    for id in &om.baseline {
        factor /= value(id).ok_or_else(|| format!("[gear.omnium] baseline trait '{id}' has no entry in traits"))?;
    }
    if factor != 1.0 {
        let player = doc.get_mut("player").and_then(|v| v.as_table_mut()).ok_or("spec has no [player] table")?;
        let cur = player.get("global_mult").and_then(|v| v.as_float()).unwrap_or(1.0);
        player.insert("global_mult".into(), toml::Value::Float(cur * factor));
    }
    let unmodelled: Vec<&str> = have.iter().filter(|id| value(id).is_none()).map(|s| s.as_str()).collect();
    Ok(json!({ "traits": rows, "baseline": om.baseline, "unmodelled": unmodelled, "factor": factor }))
}

/// `[gear.procs]`: final-stat additions for proc uptime (same units as `[player]`).
#[derive(Deserialize, Default)]
struct ProcBonus {
    #[serde(default)]
    attack_power: f64,
    #[serde(default)]
    spell_power: f64,
    #[serde(default)]
    crit: f64,
    #[serde(default)]
    haste: f64,
    #[serde(default)]
    mastery: f64,
    #[serde(default)]
    versatility: f64,
}

impl ProcBonus {
    fn get(&self, stat: &str) -> f64 {
        match stat {
            "attack_power" => self.attack_power,
            "spell_power" => self.spell_power,
            "crit" => self.crit,
            "haste" => self.haste,
            "mastery" => self.mastery,
            "versatility" => self.versatility,
            _ => 0.0,
        }
    }
}

/// Raid-night extras on top of the gear (consumables, Bloodlust): only
/// applied with gear lines, and switchable with `gear.kit.enabled = 0`.
#[derive(Deserialize)]
struct Kit {
    #[serde(default = "one")]
    enabled: f64,
    /// Flat additions before rating conversion and stat multipliers:
    /// "primary", "crit_rating", "haste_rating", "mastery_rating", "versatility_rating".
    #[serde(default)]
    stats: HashMap<String, f64>,
    /// Abilities appended to the spec's `precombat` list.
    #[serde(default)]
    precombat: Vec<String>,
    /// Buff name -> flat primary stat it grants; written into that buff's
    /// `power_pct` from the character's computed attack power.
    #[serde(default)]
    primary_buffs: HashMap<String, f64>,
}

fn one() -> f64 { 1.0 }

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
fn compute(profile: &str, cfg: &GearSpec) -> Result<Computed, String> {
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
    let mut equipped = Vec::new();
    let mut totals: HashMap<&'static str, i64> = HashMap::new();
    let mut weapon_dps = 0.0;
    let mut gems_worn: Vec<i64> = Vec::new();

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

        gems_worn.extend(gem_ids.iter().copied());
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

        equipped.push(Equipped { slot: slot.to_string(), id: item_id, ilvl, inv_type, bonus_ids: bonus_ids.clone() });
        items_out.push(json!({
            "slot": slot,
            "id": item_id,
            "ilvl": ilvl,
            "stats": item_stats,
        }));
    }

    let kit = cfg.kit.as_ref().filter(|k| k.enabled > 0.0);
    let kit_stat = |k: &str| kit.and_then(|kit| kit.stats.get(k).copied()).unwrap_or(0.0);
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

    let rating_mult = |k: &str| cfg.rating_mult.get(k).copied().unwrap_or(1.0);
    let total_crit = crit_rating as f64 * rating_mult("crit") + kit_stat("crit_rating");
    let total_haste = haste_rating as f64 * rating_mult("haste") + kit_stat("haste_rating");
    let total_mastery = mastery_rating as f64 * rating_mult("mastery") + kit_stat("mastery_rating");
    let total_vers = versatility_rating as f64 * rating_mult("versatility") + kit_stat("versatility_rating");
    let crit_dr = dr_percent(total_crit / cr["crit"]);
    let haste_dr = dr_percent(total_haste / cr["haste"]);
    let mastery_dr = dr_percent(total_mastery / cr["mastery"]);
    let vers_dr = dr_percent(total_vers / cr["versatility"]);

    let primary = ((cfg.base_primary + gear_primary + kit_stat("primary")) * cfg.primary_mult).floor();
    let crit = crit_dr / 100.0 + cfg.crit_bonus;
    let haste = (1.0 + haste_dr / 100.0) * cfg.haste_mult - 1.0;
    let mastery = (mastery_dr + cfg.mastery_points) * cfg.mastery_coeff / 100.0;
    let versatility = vers_dr / 100.0 + cfg.versatility_bonus;

    // Buffed stats (the reference's snapshot), then proc averages on top.
    let mut buffed: Vec<(&'static str, f64)> = Vec::new();
    if let Some(ap_mult) = cfg.attack_power_mult {
        buffed.push(("attack_power", (primary * ap_mult).floor()));
        buffed.push(("weapon_attack_power", weapon_dps * cfg.weapon_ap_per_dps));
    }
    if let Some(sp_mult) = cfg.spell_power_per_primary {
        buffed.push(("spell_power", (primary * sp_mult).floor()));
    }
    buffed.extend([("crit", crit), ("haste", haste), ("mastery", mastery), ("versatility", versatility)]);
    let player_pairs: Vec<(&'static str, f64)> =
        buffed.iter().map(|&(k, v)| (k, v + cfg.procs.get(k))).collect();
    let to_obj = |pairs: &[(&'static str, f64)]| -> Value {
        Value::Object(pairs.iter().map(|&(k, v)| (k.to_string(), json!(v))).collect())
    };

    // Flat primary-stat buffs as a share of the engine's total attack (or spell) power.
    let power_per_primary = cfg.attack_power_mult.or(cfg.spell_power_per_primary).unwrap_or(0.0);
    let total_power: f64 = player_pairs
        .iter()
        .filter(|(k, _)| matches!(*k, "attack_power" | "weapon_attack_power" | "spell_power"))
        .map(|(_, v)| v)
        .sum();
    let power_buffs: Vec<(String, f64)> = kit
        .map(|k| {
            k.primary_buffs
                .iter()
                .map(|(b, v)| (b.clone(), v * cfg.primary_mult * power_per_primary / total_power * 100.0))
                .collect()
        })
        .unwrap_or_default();

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
        "kit": kit.map(|k| json!({
            "stats": k.stats,
            "precombat": k.precombat,
            "power_pct": power_buffs.iter().cloned().collect::<BTreeMap<_, _>>(),
        })),
        "primary": primary,
        "buffed": to_obj(&buffed),
        "player": to_obj(&player_pairs),
    });

    let kit_out = kit.map(|k| KitOut { precombat: k.precombat.clone(), power_buffs });
    let ctx = StatCtx {
        ratings: [
            total_crit,
            total_haste,
            total_mastery,
            total_vers,
        ],
        per_pct: [cr["crit"], cr["haste"], cr["mastery"], cr["versatility"]],
        mastery_coeff: cfg.mastery_coeff,
        power_pct_per_primary: cfg.primary_mult * power_per_primary / total_power * 100.0,
    };
    Ok(Computed { report, player_pairs, kit: kit_out, equipped, gems: gems_worn, ctx })
}

struct Computed {
    report: Value,
    player_pairs: Vec<(&'static str, f64)>,
    kit: Option<KitOut>,
    equipped: Vec<Equipped>,
    /// every socketed gem id
    gems: Vec<i64>,
    ctx: StatCtx,
}

struct KitOut {
    precombat: Vec<String>,
    power_buffs: Vec<(String, f64)>,
}

struct Equipped {
    slot: String,
    id: i64,
    ilvl: usize,
    inv_type: i64,
    bonus_ids: Vec<i64>,
}

/// Bit set of the gem colours (see `GEM_COLORS`) among gem ids.
fn gem_colors(gems: &[i64]) -> u8 {
    let gd = gamedata();
    gems.iter()
        .filter_map(|g| gd.gems.get(&g.to_string()))
        .filter_map(|e| e.first())
        .filter_map(|e| GEM_COLORS.iter().position(|c| c.1 == e.0))
        .fold(0u8, |m, i| m | 1 << i)
}

/// Gem colours by the stat their gem carries first (SimC's gem colours:
/// Garnet crit, Peridot haste, Amethyst mastery, Lapis versatility).
/// The embellishment that doubles every other embellishment's values.
const BANDOLIER: i64 = 1251905;

const GEM_COLORS: [(&str, i64); 4] = [("garnet", 32), ("peridot", 36), ("amethyst", 49), ("lapis", 40)];

/// What a buff's gear `stats` are worth on this character, at the margin.
struct StatCtx {
    /// crit, haste, mastery, versatility rating (gear + kit)
    ratings: [f64; 4],
    /// rating per 1% before diminishing returns, same order
    per_pct: [f64; 4],
    mastery_coeff: f64,
    /// `power_pct` per point of primary stat
    power_pct_per_primary: f64,
}

impl StatCtx {
    /// Extra percent points (after diminishing returns) from `x` more of rating `k`.
    fn dr_gain(&self, k: usize, x: f64) -> f64 {
        let r = self.ratings[k];
        dr_percent((r + x) / self.per_pct[k]) - dr_percent(r / self.per_pct[k])
    }

    /// Buff fields (and values) for per-stack gear `stats`.
    fn buff_fields(&self, stats: &toml::Table) -> Result<Vec<(&'static str, f64)>, String> {
        let mut out = Vec::new();
        for (k, v) in stats {
            let x = v.as_float().or_else(|| v.as_integer().map(|i| i as f64)).ok_or_else(|| format!("stats.{k} is not a number"))?;
            out.push(match k.as_str() {
                "primary" => ("power_pct", x * self.power_pct_per_primary),
                "crit_rating" => ("crit_pct", self.dr_gain(0, x)),
                // haste multiplies: (1 + new) / (1 + old)
                "haste_rating" => {
                    let h = dr_percent(self.ratings[1] / self.per_pct[1]);
                    ("haste_pct", ((100.0 + h + self.dr_gain(1, x)) / (100.0 + h) - 1.0) * 100.0)
                }
                "mastery_rating" => ("mastery_pct", self.dr_gain(2, x) * self.mastery_coeff),
                "versatility_rating" => ("versatility_pct", self.dr_gain(3, x)),
                other => return Err(format!("unknown gear stat '{other}'")),
            });
        }
        Ok(out)
    }
}

/// One item effect or set bonus in play, and the item level its numbers scale with.
struct Active {
    spell: i64,
    ilvl: usize,
    inv_type: i64,
    spec: i64,
    /// gem colours worn anywhere in the gear (bit per `GEM_COLORS`)
    colors: u8,
    /// an embellishment that doubles the others is worn (Bandolier, 1251905)
    bandolier: bool,
    /// "trinket1", or "set 2062 (4)"
    from: String,
}

/// The effects the equipped gear carries: (in play, report rows).
fn active_effects(eq: &[Equipped], gems: &[i64], cfg: &GearSpec) -> (Vec<Active>, Vec<Value>) {
    let gd = gamedata();
    active_effects_with(eq, gem_colors(gems), cfg, |id| gd.item_effects.get(&id.to_string()).map(|v| v.as_slice()).unwrap_or(&[]))
}

/// DB2 `ItemEffect` trigger type as the report names it.
fn trigger_name(t: i64) -> &'static str {
    match t {
        0 => "use",
        1 => "equip",
        2 => "proc",
        _ => "other",
    }
}

/// `active_effects` with the item -> effects lookup given. Items of any slot
/// (trinkets, rings, necks, weapons...) carry effects the same way.
fn active_effects_with<'a>(
    eq: &[Equipped],
    colors: u8,
    cfg: &GearSpec,
    effects_of: impl Fn(i64) -> &'a [[i64; 4]],
) -> (Vec<Active>, Vec<Value>) {
    let gd = gamedata();
    let mut act = Vec::new();
    let mut rows = Vec::new();
    // an item's own effects, then those its bonuses add
    let of = |it: &Equipped| -> Vec<[i64; 4]> {
        let mut v = effects_of(it.id).to_vec();
        for b in &it.bonus_ids {
            v.extend(gd.bonus_effects.get(&b.to_string()).into_iter().flatten().copied());
        }
        v
    };
    let bandolier = eq.iter().any(|it| of(it).iter().any(|e| e[1] == BANDOLIER));
    let status = |f: f64| if f > 0.0 { Value::Null } else { json!("off") };
    for it in eq {
        for [trigger, spell, _, _] in of(it) {
            rows.push(json!({"from": it.slot, "item": it.id, "spell": spell, "trigger": trigger_name(trigger), "status": status(cfg.trinket_effects)}));
            if cfg.trinket_effects > 0.0 {
                act.push(Active { spell, ilvl: it.ilvl, inv_type: it.inv_type, spec: cfg.spec_id, colors, bandolier, from: it.slot.clone() });
            }
        }
    }
    // pieces per set, and the highest item level among them
    let mut sets: BTreeMap<i64, (u32, usize, i64)> = BTreeMap::new();
    for it in eq {
        if let Some(&set) = gd.item_sets.get(&it.id.to_string()) {
            let e = sets.entry(set).or_insert((0, 0, it.inv_type));
            e.0 += 1;
            e.1 = e.1.max(it.ilvl);
        }
    }
    for (set, (n, ilvl, inv_type)) in sets {
        for &[pieces, spec, spell] in gd.set_bonuses.get(&set.to_string()).map(|v| v.as_slice()).unwrap_or(&[]) {
            if n as i64 >= pieces && (spec == 0 || spec == cfg.spec_id) {
                let from = format!("set {set} ({pieces})");
                rows.push(json!({"from": from, "set": set, "pieces": pieces, "spell": spell, "status": status(cfg.set_bonuses)}));
                if cfg.set_bonuses > 0.0 {
                    act.push(Active { spell, ilvl, inv_type, spec: cfg.spec_id, colors, bandolier, from });
                }
            }
        }
    }
    (act, rows)
}

fn library() -> &'static [EffectTpl] {
    static LIB: OnceLock<Vec<EffectTpl>> = OnceLock::new();
    LIB.get_or_init(|| toml::from_str::<Library>(ITEM_EFFECTS).expect("parse data/item_effects.toml").effect)
}

/// Value of one `${...}` placeholder: `[spell.]field[:pct|:ap|:sp]`, where the
/// spell defaults to the template's own and `field` is `eN` (effect N, from 1:
/// its coefficient times the item-level budget of its scaling class, else its
/// base points), `rppm`, `rppm_haste`, `duration`, `icd`, `cooldown` (seconds),
/// `stacks`, `chance` or `ilvl`. Modifiers (chainable): `pct` (/100), `ap`/`sp`
/// (the coefficient instead), `lvl` (scale at the character's level, for spells
/// without the item-level flag), `band` (x2 with the Bandolier embellishment)
/// `per_option` (divided by one plus the gem colours worn) and `divN` (divided
/// by the integer N, e.g. `div12`). A field `eN/eM` is the ratio of two
/// effects, rounded to a whole number (a stack count).
fn placeholder(expr: &str, a: &Active) -> Result<String, String> {
    let gd = gamedata();
    let (body, mods) = match expr.split_once(':') {
        Some((b, m)) => (b, m.split(':').collect::<Vec<_>>()),
        None => (expr, vec![]),
    };
    let has = |m: &str| mods.contains(&m);
    // the one modifier that picks the form of the value (`ap`, `sp`, `lvl`) and the ones that scale it
    let modifier = mods.iter().copied().find(|m| ["ap", "sp", "lvl"].contains(m));
    let (spell, field) = match body.split_once('.') {
        Some((s, f)) => (s.parse::<i64>().map_err(|_| format!("bad spell id in ${{{expr}}}"))?, f),
        None => (a.spell, body),
    };
    if field == "ilvl" {
        return Ok(a.ilvl.to_string());
    }
    if let Some((n, d)) = field.split_once('/') {
        let get = |f: &str| -> Result<f64, String> {
            placeholder(&format!("{spell}.{f}"), a)?.parse::<f64>().map_err(|e| format!("${{{expr}}}: {e}"))
        };
        return Ok(((get(n)? / get(d)?).round().max(1.0) as i64).to_string());
    }
    let sp = gd.spells.get(&spell.to_string()).ok_or_else(|| {
        format!("spell {spell} isn't in the game data (rerun tools/gamedata.py after adding a template)")
    })?;
    let secs = |ms: i64| ms as f64 / 1000.0;
    let v = match field {
        "rppm" => {
            let mut r = sp.rppm;
            for &(kind, param, coeff) in &sp.rppm_mods {
                // spec modifier
                if kind == 4 && param == a.spec {
                    r *= 1.0 + coeff;
                }
            }
            r
        }
        "rppm_haste" => return Ok(sp.rppm_mods.iter().any(|m| m.0 == 1).to_string()),
        "duration" => secs(sp.duration),
        "icd" => secs(sp.icd),
        "cooldown" => secs(sp.cooldown),
        "stacks" => sp.stacks.max(1) as f64,
        "chance" => sp.chance / 100.0,
        f if f.starts_with('e') => {
            let n: usize = f[1..].parse().map_err(|_| format!("bad field in ${{{expr}}}"))?;
            let e = sp.effects.get(n.wrapping_sub(1)).ok_or_else(|| format!("spell {spell} has no effect {n}"))?;
            let (_, _, bp, coeff, class, ap, spc, _, _) = *e;
            match modifier {
                Some("ap") => ap,
                Some("sp") => spc,
                // spells without the 'scales with the casting item's level' flag are scaled at the character's level
                Some("lvl") if coeff != 0.0 => {
                    let lvl = gd.spell_scaling_item.len() - 1;
                    // rating effects also take the trinket combat-rating multiplier (SimC: SCALE7)
                    let crm = if class == -7 { gd.combat_rating_mult[lvl][2] } else { 1.0 };
                    coeff * gd.spell_scaling_item[lvl] * crm
                }
                _ if coeff != 0.0 => coeff * budget(class, a.ilvl, a.inv_type)?,
                _ => bp,
            }
        }
        _ => return Err(format!("unknown field in ${{{expr}}}")),
    };
    let div = |m: &str| m.strip_prefix("div").and_then(|n| n.parse::<f64>().ok()).filter(|n| *n != 0.0);
    if let Some(m) = mods.iter().find(|m| !["ap", "sp", "lvl", "pct", "per_option", "band"].contains(m) && div(m).is_none()) {
        return Err(format!("unknown modifier ':{m}' in ${{{expr}}}"));
    }
    let mut v = v;
    if has("pct") {
        v /= 100.0;
    }
    for m in &mods {
        if let Some(n) = div(m) {
            v /= n;
        }
    }
    if has("band") && a.bandolier {
        v *= 2.0;
    }
    // one of the always-available option and one per gem colour worn
    if has("per_option") {
        v /= (1 + a.colors.count_ones()) as f64;
    }
    Ok(v.to_string())
}

/// Item-level budget an effect coefficient multiplies, by scaling class (SimC's
/// `spelleffect_data_t::average(item)`, epic budgets).
fn budget(class: i64, ilvl: usize, inv_type: i64) -> Result<f64, String> {
    let gd = gamedata();
    if ilvl >= gd.rand_prop_points.len() {
        return Err(format!("item level {ilvl} out of range"));
    }
    let col = match inv_type {
        2 | 11 => 3,
        12 => 2,
        13 | 14 | 15 | 17 | 21 | 22 | 23 | 26 => 1,
        _ => 0,
    };
    Ok(match class {
        -7 => gd.rand_prop_points[ilvl][0] * gd.combat_rating_mult[ilvl][col],
        -8 => gd.rand_prop_damage[ilvl][0],
        -9 | 0 => gd.rand_prop_damage[ilvl][1],
        _ => gd.rand_prop_points[ilvl][0],
    })
}

/// Fill a template's placeholders.
fn fill(text: &str, a: &Active) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("${") {
        out.push_str(&rest[..i]);
        let j = rest[i..].find('}').ok_or("unclosed ${ in template")?;
        out.push_str(&placeholder(&rest[i + 2..i + j], a)?);
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Add the fragments for the gear's effects to the spec document. Returns
/// the report rows, each marked with its status.
fn apply_effects(doc: &mut toml::Value, eq: &[Equipped], gems: &[i64], cfg: &GearSpec) -> Result<Vec<Value>, String> {
    let (act, mut rows) = active_effects(eq, gems, cfg);
    let own = |spell: i64| cfg.effects.iter().any(|t| t.spell == spell);
    // the spec's own templates for a spell win over the library's; a template
    // with a `gem` colour counts only where that colour is worn
    let tpls = |spell: i64, colors: u8| -> Vec<&EffectTpl> {
        let pick = |list: &'static [EffectTpl]| -> Vec<&EffectTpl> {
            list.iter()
                .filter(|t| t.spell == spell)
                .filter(|t| t.gem.as_deref().is_none_or(|g| GEM_COLORS.iter().position(|c| c.0 == g).is_some_and(|i| colors >> i & 1 == 1)))
                .collect()
        };
        let own: Vec<&EffectTpl> = cfg.effects.iter().filter(|t| t.spell == spell).collect();
        if own.is_empty() { pick(library()) } else { own }
    };
    let mut patches = Vec::new();
    let mut rotation = Vec::new();
    for a in &act {
        let row = rows.iter_mut().find(|r| r["spell"] == a.spell && r["from"] == a.from.as_str()).unwrap();
        let ts = tpls(a.spell, a.colors);
        if ts.is_empty() {
            row["status"] = json!("unmodelled");
            continue;
        }
        row["status"] = json!(if ts.iter().all(|t| t.none) { "covered" } else { "modelled" });
        let ctx = |e: String| format!("effect {} ({}): {e}", a.spell, a.from);
        for t in ts {
        if t.none {
            continue;
        }
        let text = fill(&t.text, a).map_err(ctx)?;
        let frag: toml::Table = toml::from_str(&text).map_err(|e| ctx(e.to_string()))?;
        let root = doc.as_table_mut().ok_or("spec is not a table")?;
        for (k, v) in frag {
            let toml::Value::Array(mut items) = v else { return Err(ctx(format!("'{k}' must be a list"))) };
            if k == "abilities" && !own(a.spell) {
                for ab in &mut items {
                    if let Some(d) = ab.get_mut("damage").and_then(|d| d.as_table_mut()) {
                        let school = d.get("school").and_then(|x| x.as_str()).unwrap_or("physical");
                        let m = cfg.effect_mult.get(if school == "physical" { "physical" } else { "magic" }).copied().unwrap_or(1.0);
                        let cur = d.get("mult").and_then(|x| x.as_float()).unwrap_or(1.0);
                        d.insert("mult".into(), toml::Value::Float(cur * m));
                    }
                }
            }
            match k.as_str() {
                "abilities" | "buffs" | "procs" | "pets" | "precombat" => {
                    let list = root.entry(k.as_str()).or_insert_with(|| toml::Value::Array(vec![]));
                    list.as_array_mut().ok_or_else(|| ctx(format!("spec's '{k}' is not a list")))?.extend(items);
                }
                "rotation" => {
                    for r in items {
                        let mut r = r.as_table().cloned().ok_or_else(|| ctx("rotation entries are tables".into()))?;
                        if let Some(kind) = r.remove("use") {
                            let kind = kind.as_str().unwrap_or("").to_string();
                            if let Some(c) = cfg.use_items.get(&kind) {
                                let c = fill(c, a).map_err(ctx)?;
                                let own = r.get("if").and_then(|x| x.as_str()).map(|s| s.to_string());
                                r.insert("if".into(), toml::Value::String(match own {
                                    Some(o) => format!("({o})&({c})"),
                                    None => c,
                                }));
                            }
                            r.entry("after").or_insert_with(|| toml::Value::String(cfg.use_after.clone().unwrap_or_default()));
                        }
                        rotation.push(r);
                    }
                }
                "patch" => patches.extend(items),
                _ => return Err(ctx(format!("unknown fragment list '{k}'"))),
            }
        }
        }
    }
    let root = doc.as_table_mut().ok_or("spec is not a table")?;
    for p in patches {
        patch(root, p)?;
    }
    if !rotation.is_empty() {
        let list = root
            .get_mut("rotation")
            .and_then(|r| r.as_array_mut())
            .ok_or("spec has no rotation")?;
        for mut r in rotation {
            let action = |e: &toml::Value| e.get("action").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let cond = |e: &toml::Value| e.get("if").and_then(|x| x.as_str()).map(|s| s.to_string());
            if let Some(before) = r.remove("before") {
                // in front of every entry for that action, under its condition too
                let before = before.as_str().unwrap_or("").to_string();
                let mut i = 0;
                while i < list.len() {
                    if action(&list[i]) == before {
                        let mut e = r.clone();
                        if let Some(theirs) = cond(&list[i]) {
                            let mine = e.get("if").and_then(|x| x.as_str()).map(|s| s.to_string());
                            e.insert("if".into(), toml::Value::String(match mine {
                                Some(m) => format!("({m})&({theirs})"),
                                None => theirs,
                            }));
                        }
                        list.insert(i, toml::Value::Table(e));
                        i += 1;
                    }
                    i += 1;
                }
            } else {
                let after = r.remove("after").and_then(|x| x.as_str().map(|s| s.to_string())).unwrap_or_default();
                let at = list.iter().position(|e| action(e) == after).map_or(0, |i| i + 1);
                list.insert(at, toml::Value::Table(r));
            }
        }
    }
    Ok(rows)
}

/// `[[patch]]`: append lists to an existing ability's, buff's or pet's (`ability =` /
/// `buff =` / `pet =`), e.g. `on_cast`; skipped when the target or anything in
/// `requires` (ability names) isn't in the spec. `set = { field = value }`
/// overwrites scalar fields instead.
fn patch(root: &mut toml::Table, p: toml::Value) -> Result<(), String> {
    let mut p = p.as_table().cloned().ok_or("patch entries are tables")?;
    let (kind, name) = match (p.remove("ability"), p.remove("buff"), p.remove("pet")) {
        (Some(n), None, None) => ("abilities", n),
        (None, Some(n), None) => ("buffs", n),
        (None, None, Some(n)) => ("pets", n),
        _ => return Err("a patch names one ability, one buff or one pet".into()),
    };
    let name = name.as_str().unwrap_or("").to_string();
    let has = |root: &toml::Table, kind: &str, n: &str| {
        root.get(kind).and_then(|l| l.as_array()).is_some_and(|l| l.iter().any(|e| e.get("name").and_then(|x| x.as_str()) == Some(n)))
    };
    let requires: Vec<String> = p
        .remove("requires")
        .and_then(|r| r.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    if !has(root, kind, &name) || requires.iter().any(|r| !has(root, "abilities", r)) {
        return Ok(());
    }
    let target = root
        .get_mut(kind)
        .and_then(|l| l.as_array_mut())
        .and_then(|l| l.iter_mut().find(|e| e.get("name").and_then(|x| x.as_str()) == Some(name.as_str())))
        .and_then(|e| e.as_table_mut())
        .unwrap();
    for (k, v) in p {
        // `set = { field = value, .. }` overwrites scalar fields (e.g. a buff's max_stacks)
        if k == "set" {
            let toml::Value::Table(fields) = v else { return Err(format!("patch '{name}': 'set' must be a table")) };
            for (f, x) in fields {
                target.insert(f, x);
            }
            continue;
        }
        let toml::Value::Array(items) = v else { return Err(format!("patch '{name}': '{k}' must be a list")) };
        let list = target.entry(k.as_str()).or_insert_with(|| toml::Value::Array(vec![]));
        list.as_array_mut().ok_or_else(|| format!("patch '{name}': '{k}' is not a list"))?.extend(items);
    }
    Ok(())
}

/// Turn every buff's gear `stats` into the matching percent fields.
fn apply_buff_stats(doc: &mut toml::Value, ctx: &StatCtx) -> Result<(), String> {
    let Some(buffs) = doc.get_mut("buffs").and_then(|b| b.as_array_mut()) else { return Ok(()) };
    for b in buffs {
        let Some(t) = b.as_table_mut() else { continue };
        let Some(stats) = t.get("stats").and_then(|s| s.as_table()).cloned() else { continue };
        let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("?").to_string();
        for (field, v) in ctx.buff_fields(&stats).map_err(|e| format!("buff '{name}': {e}"))? {
            let cur = t.get(field).and_then(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).unwrap_or(0.0);
            t.insert(field.into(), toml::Value::Float(cur + v));
        }
    }
    Ok(())
}

/// Parse a SimC gear profile, overwrite `player.*` stats in `doc`, and return
/// both the updated TOML document and the JSON breakdown report.
pub fn apply(doc: toml::Value, path: &str) -> Result<(toml::Value, Value), String> {
    let profile = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    apply_text(doc, &profile)
}

/// Like [`apply`], but takes the profile text (SimC gear lines) directly.
pub fn apply_text(mut doc: toml::Value, profile: &str) -> Result<(toml::Value, Value), String> {
    let gear_tbl = doc.get("gear").cloned().ok_or_else(|| {
        "spec has no [gear] table".to_string()
    })?;
    let cfg: GearSpec = gear_tbl.try_into().map_err(|e| format!("[gear]: {e}"))?;
    let Computed { mut report, player_pairs, kit, equipped, gems, ctx } = compute(profile, &cfg)?;
    let effects = apply_effects(&mut doc, &equipped, &gems, &cfg)?;
    apply_buff_stats(&mut doc, &ctx)?;
    report["unmodelled"] = json!(effects.iter().filter(|r| r["status"] == "unmodelled").map(|r| r["from"].clone()).collect::<Vec<_>>());
    report["effects"] = json!(effects);
    report["omnium"] = apply_omnium(&mut doc, profile, &cfg)?;

    let player = doc
        .get_mut("player")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| "spec has no [player] table".to_string())?;
    for (k, v) in player_pairs {
        player.insert(k.to_string(), toml::Value::Float(v));
    }
    if let Some(kit) = kit {
        let root = doc.as_table_mut().ok_or("spec is not a table")?;
        let pre = root.entry("precombat").or_insert_with(|| toml::Value::Array(vec![]));
        let pre = pre.as_array_mut().ok_or("precombat is not a list")?;
        pre.extend(kit.precombat.into_iter().map(toml::Value::String));
        for (name, pct) in kit.power_buffs {
            let buff = root
                .get_mut("buffs")
                .and_then(|b| b.as_array_mut())
                .and_then(|bs| bs.iter_mut().find(|b| b.get("name").and_then(|n| n.as_str()) == Some(name.as_str())))
                .and_then(|b| b.as_table_mut())
                .ok_or_else(|| format!("[gear.kit] primary_buffs: no buff '{name}'"))?;
            buff.insert("power_pct".into(), toml::Value::Float(pct));
        }
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
        let report = compute(RET_CORE_PROFILE, &cfg).unwrap().report;
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
        let rep_trink = compute(&with_trinkets, &cfg).unwrap().report;
        assert_eq!(rep_trink["gear"]["crit_rating"], 1200);
        assert_eq!(rep_trink["gear"]["strength"], 1927);
    }

    fn ret_with(profile: &str, sets: &[(&str, f64)]) -> (crate::spec::Spec, Value) {
        let mut doc: toml::Value = toml::from_str(include_str!("../specs/retribution_herald.toml")).unwrap();
        for (k, v) in sets {
            doc["gear"].as_table_mut().unwrap().insert(k.to_string(), toml::Value::Float(*v));
        }
        let (out, rep) = apply_text(doc, profile).unwrap();
        (out.try_into().unwrap(), rep)
    }

    const RET_TRINKETS: &str = "trinket1=,id=270173,bonus_id=13335,ilevel=344\ntrinket2=,id=270175,bonus_id=13335,ilevel=344\n";

    #[test]
    fn trinket_effects_and_set_bonuses_from_gear() {
        let (sp, rep) = ret_with(&format!("{RET_CORE_PROFILE}{RET_TRINKETS}"), &[]);
        let status: Vec<(String, String)> = rep["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["from"].as_str().unwrap().to_string(), r["status"].as_str().unwrap().to_string()))
            .collect();
        let want = [
            ("wrists", "covered"), // Bandolier
            ("finger2", "modelled"), // Loa Worshiper's Band
            ("trinket1", "modelled"),
            ("trinket2", "modelled"),
            ("trinket2", "covered"),
            ("set 2062 (2)", "modelled"),
            ("set 2062 (4)", "modelled"),
            ("set 2070 (2)", "modelled"),
        ];
        assert_eq!(status, want.map(|(a, b)| (a.to_string(), b.to_string())));
        assert_eq!(rep["unmodelled"], json!([]));

        // Loa Worshiper's Band (item level 331), Peridot gems and the Bandolier: two options
        // (Capybara, Akil'zon) share the 2 PPM, and values are doubled
        let rate = |n: &str| sp.procs.iter().find(|p| p.ability == n).unwrap_or_else(|| panic!("no {n}")).per_minute;
        assert_eq!((rate("loa_capybara"), rate("loa_akilzon")), (1.0, 1.0));
        assert!(sp.procs.iter().all(|p| p.ability != "loa_halazzi" && p.ability != "loa_nalorakk"));
        let (cap, akil) = (sp.buffs.iter().find(|b| b.name == "loa_capybara").unwrap(), sp.buffs.iter().find(|b| b.name == "loa_akilzon").unwrap());
        assert_eq!((cap.duration, akil.duration), (15.0, 15.0));
        assert!(cap.power_pct > 0.0 && akil.haste_pct > 0.0);

        let ability = |n: &str| sp.abilities.iter().find(|a| a.name == n).unwrap_or_else(|| panic!("no {n}"));
        let buff = |n: &str| sp.buffs.iter().find(|b| b.name == n).unwrap();
        // values from the game data at item level 344, plus the spec's raid-debuff multiplier
        let g = ability("guillotine").damage.clone().unwrap();
        assert!((g.amount - 62.78858947754 * 804.937988).abs() < 0.01);
        assert!((g.mult - 1.0815).abs() < 1e-12);
        assert_eq!(g.scale.as_deref(), Some("1+0.02*(100-target.health.pct)"));
        assert!((ability("devour_morsel").damage.clone().unwrap().amount - 19.86443710327 * 804.937988).abs() < 0.01);
        // primary stat per stack: 1.2183 and 0.1364 x the 344 budget, as attack power share
        let (vh, ds) = (buff("voracious_heart").power_pct, buff("devoured_strength").power_pct);
        assert!(vh > 5.0 && vh < 20.0, "{vh}");
        assert!((ds / vh - 0.13637375832 / 1.21833908558).abs() < 1e-9);
        assert_eq!(ability("voracious_heart").cooldown, 90.0);
        assert_eq!(ability("divine_arbiter").damage.clone().unwrap().ap_coeff, 10.125);
        // Bite of Zul'jan 2pc: Guillotine applies Venomfang
        assert!(matches!(&ability("guillotine").on_cast[..], [crate::spec::Effect::Trigger { ability, .. }] if ability == "venomfang_proc"));
        // 2pc (Divine Power) and 4pc (arming the Arbiter) on Final Verdict's free casts
        assert_eq!(ability("final_verdict").on_free.len(), 2);
        // on-use after the potion; hard-cast Divine Storm ahead of each Final Verdict, under its condition
        let rot: Vec<&str> = sp.rotation.iter().map(|r| r.action.as_str()).collect();
        let at = |n: &str| rot.iter().position(|a| *a == n).unwrap();
        assert_eq!(at("voracious_heart"), at("potion") + 1);
        let fv: Vec<usize> = rot.iter().enumerate().filter(|(_, a)| **a == "final_verdict").map(|(i, _)| i).collect();
        assert_eq!(fv.len(), 2);
        for i in fv {
            assert_eq!(rot[i - 1], "divine_storm_cast");
            let (ds, fv) = (sp.rotation[i - 1].condition.clone().unwrap_or_default(), sp.rotation[i].condition.clone());
            assert!(fv.is_none_or(|c| ds.ends_with(&format!("&({c})"))), "{ds}");
        }
    }

    #[test]
    fn omnium_traits_parse_and_scale_against_the_baseline() {
        assert_eq!(omnium_traits("omnium_talents=136822:1/136816:1\n"), ["136822", "136816"]);
        assert_eq!(omnium_traits("omnium_talents=rune_of_overload/rune_of_lingering"), ["rune_of_overload", "rune_of_lingering"]);
        assert!(omnium_traits("level=90\n").is_empty());

        let with = |profile: &str| {
            let mut doc: toml::Value = toml::from_str(include_str!("../specs/retribution_herald.toml")).unwrap();
            let om: toml::Value = toml::from_str(
                "baseline = [\"1\", \"2\"]\ntraits = [{id=\"1\", global_mult=1.02}, {id=\"2\", global_mult=1.01}, {id=\"3\", global_mult=1.05}]",
            )
            .unwrap();
            doc["gear"].as_table_mut().unwrap().insert("omnium".into(), om);
            let (out, rep) = apply_text(doc, profile).unwrap();
            (out["player"].get("global_mult").and_then(|v| v.as_float()).unwrap_or(1.0), rep["omnium"].clone())
        };
        // the reference's own traits: unchanged
        let (m, rep) = with(&format!("{RET_CORE_PROFILE}omnium_talents=1:1/2:1\n"));
        assert!((m - 1.0).abs() < 1e-12, "{m}");
        assert_eq!(rep["unmodelled"], json!([]));
        // none: the baseline comes out; an extra trait goes on; an unknown one is reported
        let (m, _) = with(RET_CORE_PROFILE);
        assert!((m - 1.0 / (1.02 * 1.01)).abs() < 1e-12);
        let (m, _) = with(&format!("{RET_CORE_PROFILE}omnium_talents=1:1/2:1/3:1/9:1\n"));
        assert!((m - 1.05).abs() < 1e-12);
        let (_, rep) = with(&format!("{RET_CORE_PROFILE}omnium_talents=9:1\n"));
        assert_eq!(rep["unmodelled"], json!(["9"]));
        // a spec without [gear.omnium] never moves
        let mut doc: toml::Value = toml::from_str(include_str!("../specs/retribution_herald.toml")).unwrap();
        doc["gear"].as_table_mut().unwrap().remove("omnium");
        let (_, rep) = apply_text(doc, &format!("{RET_CORE_PROFILE}omnium_talents=1:1\n")).unwrap();
        assert_eq!(rep["omnium"]["factor"], 1.0);
        // the shipped Ret table: the sample profile's own five traits change nothing, none loses 3.5%
        let ret = |extra: &str| {
            let doc: toml::Value = toml::from_str(include_str!("../specs/retribution_herald.toml")).unwrap();
            let (out, rep) = apply_text(doc, &format!("{RET_CORE_PROFILE}{extra}")).unwrap();
            (out["player"].get("global_mult").and_then(|v| v.as_float()).unwrap_or(1.0), rep["omnium"].clone())
        };
        let (m_all, rep) = ret("omnium_talents=136822:1/136816:1/136817:1/136821:1/136814:1\n");
        assert_eq!(rep["unmodelled"], json!([]));
        let (m_none, _) = ret("");
        assert!((m_none / m_all - 127_168.0 / 131_721.0).abs() < 1e-4, "{}", m_none / m_all);
        // per trait: dropping Burning Haste alone costs its own 1.01331, not the lump
        let (m_less, rep) = ret("omnium_talents=136822:1/136816:1/136817:1/136814:1\n");
        assert!((m_all / m_less - 1.01331).abs() < 1e-9, "{}", m_all / m_less);
        assert_eq!(rep["unmodelled"], json!([]));
        // Unleashed Fire alone is worth its own 1.01901, not the whole set
        let (m_fire, _) = ret("omnium_talents=136822:1\n");
        assert!((m_fire / m_none - 1.01901).abs() < 1e-9, "{}", m_fire / m_none);
    }

    #[test]
    fn effects_switch_off_count_pieces_and_report_unmodelled() {
        let profile = format!("{RET_CORE_PROFILE}{RET_TRINKETS}");
        let (sp, rep) = ret_with(&profile, &[("trinket_effects", 0.0), ("set_bonuses", 0.0)]);
        assert!(sp.abilities.iter().all(|a| a.name != "guillotine" && a.name != "divine_arbiter"));
        assert!(rep["effects"].as_array().unwrap().iter().all(|r| r["status"] == "off"));

        // three tier pieces: the 2pc only
        let three: String = profile.lines().filter(|l| !l.starts_with("head") && !l.starts_with("shoulders")).map(|l| format!("{l}\n")).collect();
        let (sp, _) = ret_with(&three, &[]);
        assert!(sp.buffs.iter().any(|b| b.name == "divine_power"));
        assert!(sp.abilities.iter().all(|a| a.name != "divine_arbiter"));

        // a trinket with an effect no template covers
        let lib: Vec<i64> = library().iter().map(|t| t.spell).collect();
        let other = gamedata()
            .item_effects
            .iter()
            .filter(|(id, v)| gamedata().items[*id][0] == 12.0 && v.iter().all(|e| !lib.contains(&e[1])))
            .map(|(id, _)| id.parse::<i64>().unwrap())
            .min()
            .unwrap();
        let (_, rep) = ret_with(&format!("{RET_CORE_PROFILE}trinket1=,id={other}\n"), &[]);
        assert_eq!(rep["unmodelled"], json!(["trinket1"]));
    }

    #[test]
    fn effects_of_any_slot_are_collected() {
        // a ring with an on-equip effect, a weapon proc and a trinket; the
        // lookup stands in for ItemEffect (which the committed data only has
        // for trinkets until tools/gamedata.py is rerun)
        let eq = |slot: &str, id: i64, inv_type: i64| Equipped { slot: slot.into(), id, ilvl: 344, inv_type, bonus_ids: vec![] };
        let items = [eq("finger1", 1, 11), eq("main_hand", 2, 13), eq("trinket1", 3, 12)];
        let table: HashMap<i64, Vec<[i64; 4]>> = HashMap::from([
            (1, vec![[1, 1297760, 0, 0]]),
            (2, vec![[2, 1297761, 0, 0]]),
            (3, vec![[0, 1297761, 90000, 0]]),
        ]);
        let cfg: GearSpec = toml::from_str(RET_GEAR_TOML).unwrap();
        let (act, rows) = active_effects_with(&items, 0, &cfg, |id| table.get(&id).map(|v| v.as_slice()).unwrap_or(&[]));
        let from: Vec<&str> = act.iter().map(|a| a.from.as_str()).collect();
        assert_eq!(from, ["finger1", "main_hand", "trinket1"]);
        let trig: Vec<&str> = rows.iter().map(|r| r["trigger"].as_str().unwrap()).collect();
        assert_eq!(trig, ["equip", "proc", "use"]);
        // rating coefficients (class -7) use the slot's combat-rating column: jewelry for a ring, trinket for a trinket

        assert_ne!(budget(-7, 344, 11).unwrap(), budget(-7, 344, 12).unwrap());
        assert_eq!(act[0].inv_type, 11);
    }

    /// A caster spec's own `[gear]` table against its core-kit profile (no trinkets).
    fn caster(spec: &str, profile: &str) -> Value {
        let doc: toml::Value = toml::from_str(spec).unwrap();
        let mut cfg: GearSpec = doc["gear"].clone().try_into().unwrap();
        cfg.kit = None; // core kit: these tests compare the reference's own snapshot
        let core: String = profile
            .lines()
            .filter(|l| !l.starts_with("trinket"))
            .map(|l| format!("{l}\n"))
            .collect();
        compute(&core, &cfg).unwrap().report
    }

    #[test]
    fn fire_mage_spell_power_and_procs() {
        let rep = caster(
            include_str!("../specs/fire_mage.toml"),
            include_str!("../profiles/MID2_Mage_Fire.simc"),
        );
        assert_eq!(rep["gear"]["intellect"], 2412);
        assert_eq!(rep["gear"]["crit_rating"], 170);
        assert_eq!(rep["gear"]["haste_rating"], 1204);
        let b = &rep["buffed"];
        assert_eq!(b["spell_power"].as_f64().unwrap(), 3341.0);
        assert!(b.get("attack_power").is_none());
        assert!((b["crit"].as_f64().unwrap() - 0.106957).abs() < 1e-5);
        assert!((b["haste"].as_f64().unwrap() - 0.299109).abs() < 1e-5);
        assert!((b["mastery"].as_f64().unwrap() - 0.175035).abs() < 1e-5);
        assert!((b["versatility"].as_f64().unwrap() - 0.140185).abs() < 1e-5);
        // final = buffed + [gear.procs]
        assert!((rep["player"]["haste"].as_f64().unwrap() - (0.299109 + 0.0278)).abs() < 1e-5);
    }

    #[test]
    fn destruction_warlock_spell_power_and_procs() {
        let rep = caster(
            include_str!("../specs/destruction_hellcaller.toml"),
            include_str!("../profiles/MID2_Warlock_Destruction.simc"),
        );
        let b = &rep["buffed"];
        assert_eq!(b["spell_power"].as_f64().unwrap(), 3375.0);
        assert!((b["crit"].as_f64().unwrap() - 0.359783).abs() < 1e-5);
        assert!((b["haste"].as_f64().unwrap() - 0.198732).abs() < 1e-5);
        assert!((b["mastery"].as_f64().unwrap() - 0.735652).abs() < 1e-5);
        assert!((b["versatility"].as_f64().unwrap() - 0.052778).abs() < 1e-5);
        assert_eq!(rep["player"]["spell_power"].as_f64().unwrap(), 3413.0);
    }

    #[test]
    fn subtlety_rogue_agility_and_weapon_power() {
        let rep = caster(
            include_str!("../specs/subtlety_deathstalker.toml"),
            include_str!("../profiles/MID2_Rogue_Subtlety.simc"),
        );
        let g = &rep["gear"];
        assert_eq!(g["agility"], 1708);
        assert_eq!(g["crit_rating"], 367);
        assert_eq!(g["haste_rating"], 993);
        assert_eq!(g["mastery_rating"], 1241);
        assert_eq!(g["versatility_rating"], 432);
        let b = &rep["buffed"];
        assert_eq!(b["attack_power"].as_f64().unwrap(), 2640.0);
        assert!((b["weapon_attack_power"].as_f64().unwrap() - 702.0).abs() < 0.5);
        assert!((b["crit"].as_f64().unwrap() - 0.199783).abs() < 1e-5);
        assert!((b["haste"].as_f64().unwrap() - 0.262452).abs() < 1e-5);
        assert!((b["mastery"].as_f64().unwrap() - 0.905967).abs() < 1e-5);
        assert!((b["versatility"].as_f64().unwrap() - 0.11).abs() < 1e-5);
    }

    /// With the raid-night kit on, the stats are SimC's buffed ones for the sample profile
    /// (run with default consumables, no trinkets): the flask differs by spec.
    fn with_kit(spec: &str, profile: &str) -> Value {
        let doc: toml::Value = toml::from_str(spec).unwrap();
        let cfg: GearSpec = doc["gear"].clone().try_into().unwrap();
        let core: String = profile.lines().filter(|l| !l.starts_with("trinket")).map(|l| format!("{l}\n")).collect();
        compute(&core, &cfg).unwrap().report
    }

    #[test]
    fn kits_match_simc_buffed_stats() {
        // (spec, profile, spell/attack power, crit, haste, mastery, versatility) from SimC's report
        let cases: [(&str, &str, f64, f64, f64, f64, f64); 4] = [
            (include_str!("../specs/fire_mage.toml"), include_str!("../profiles/MID2_Mage_Fire.simc"), 3424.0, 0.1104, 0.3028, 0.1750, 0.1707),
            (include_str!("../specs/frost_mage_spellslinger.toml"), include_str!("../profiles/MID2_Mage_Frost.simc"), 3402.0, 0.3367, 0.2669, 0.5621, 0.0528),
            (include_str!("../specs/demonology_diabolist.toml"), include_str!("../profiles/MID2_Warlock_Demonology.simc"), 3507.0, 0.3849, 0.1946, 0.4684, 0.0724),
            (include_str!("../specs/destruction_hellcaller.toml"), include_str!("../profiles/MID2_Warlock_Destruction.simc"), 3458.0, 0.3633, 0.2024, 0.8074, 0.0528),
        ];
        for (spec, profile, power, crit, haste, mastery, vers) in cases {
            let rep = with_kit(spec, profile);
            let b = &rep["buffed"];
            let name = spec.lines().next().unwrap();
            assert!((b["spell_power"].as_f64().unwrap() - power).abs() <= 3.0, "{name} spell power");
            for (k, want) in [("crit", crit), ("haste", haste), ("mastery", mastery), ("versatility", vers)] {
                assert!((b[k].as_f64().unwrap() - want).abs() < 6e-4, "{name} {k}: {} vs {want}", b[k]);
            }
        }
    }
}
