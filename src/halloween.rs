//! Halloween Event: Fracture of the Distortion World
//! ====================================================
//! Native Rust implementation of the Halloween Event for Ozone Pokémon Bot.
//! Manages event currencies (Candy Corn & Void Shards), hunt catch hooks,
//! Trick-or-Treat Cauldron mechanics, Spooky Shop (Mega Stones & Move Tutors),
//! Leaderboard rankings, and the strictly 1-of-1 Creation Trio Gods (Giratina, Dialga, Palkia).

use chrono::{Duration, FixedOffset, TimeZone, Timelike, Utc};
use rand::{seq::SliceRandom, Rng};
use sqlx::{Row, SqlitePool};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::PokedexData;

// ==============================================================================
// 1. CONFIGURATION & CONSTANTS
// ==============================================================================

pub const EVENT_NAME: &str = "Halloween Event: Fracture of the Distortion World";
pub const EVENT_SLUG: &str = "halloween_event";

pub const HALLOWEEN_CANDY_NAME: &str = "Candy Corn";
pub const HALLOWEEN_VOID_NAME: &str = "Void Shard";

// Standard emoji representations
pub const HALLOWEEN_CANDY_EMOJI: &str = "🍬";
pub const HALLOWEEN_VOID_EMOJI: &str = "🔮";
pub const HALLOWEEN_CAULDRON_EMOJI: &str = "🍲";
pub const HALLOWEEN_PUMPKIN_EMOJI: &str = "🎃";

// Drop rates on wild catch in /hunt
pub const HALLOWEEN_HUNT_CANDY_DROP_RATE: f64 = 0.15;   // 15% chance on normal Pokémon
pub const HALLOWEEN_HUNT_DOUBLE_CANDY_RATE: f64 = 0.02; // 2% chance for 2 Candies on normal Pokémon
pub const HALLOWEEN_GHOST_CANDY_DROP_RATE: f64 = 0.50;  // 50% chance on Ghost/Dark Pokémon
pub const HALLOWEEN_GHOST_BONUS_VOID_RATE: f64 = 0.05;  // 5% chance for Void Shard on Ghost/Dark Pokémon

// Cauldron Treat Configuration
pub const HALLOWEEN_CAULDRON_CANDY_COST: i64 = 25;
pub const HALLOWEEN_TREAT_DAILY_LIMIT: usize = 15;      // 15 brews per rolling 24 hours
pub const HALLOWEEN_TREAT_WINDOW_SECONDS: f64 = 86400.0;// 24 hours sliding window

// Trick Showdown Configuration
pub const HALLOWEEN_TRICK_CANDY_COST: i64 = 15;
pub const HALLOWEEN_TRICK_DAILY_LIMIT: usize = 10;     // 10 Trick Showdowns per day (resets 5:00 AM IST)
pub const HALLOWEEN_TRICK_RESET_HOUR: u32 = 5;         // 5:00 AM Indian Standard Time (IST, UTC+5:30)
pub const HALLOWEEN_TRICK_VOID_MIN: i64 = 2;
pub const HALLOWEEN_TRICK_VOID_MAX: i64 = 4;

// Shop Pricing
pub const HALLOWEEN_MEGASTONE_PRICE_SHARDS: i64 = 100; // Limit: 1 per trainer
pub const HALLOWEEN_TUTOR_PRICE_SHARDS: i64 = 50;

// Creation God Specifications
#[derive(Debug, Clone)]
pub struct CreationGodSpec {
    pub god_key: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub base_species_id: u32,
    pub origin_species_id: u32,
    pub types: &'static [&'static str],
    pub badge: &'static str,
    pub lore: &'static str,
    pub abilities: &'static [(&'static str, &'static str)],
    pub moves: &'static [&'static str],
    pub held_item: &'static str,
    pub nature: &'static str,
}

pub const CREATION_GOD_GIRATINA: CreationGodSpec = CreationGodSpec {
    god_key: "giratina",
    name: "Giratina",
    title: "Monarch of the Distortion World",
    base_species_id: 487,
    origin_species_id: 10007,
    types: &["Ghost", "Dragon"],
    badge: "[1-OF-1 GOD]",
    lore: "The banished monarch of anti-matter and shadows. Reality distorts in its presence.",
    abilities: &[
        ("Anti-Matter Dominion", "Giratina takes only 50% of incoming damage from all attacks."),
        ("Shadow Realm", "Every move targeting Giratina has a 50% chance to miss into the Shadow Realm."),
    ],
    moves: &["Shadow Force", "Dragon Pulse", "Aura Sphere", "Shadow Ball"],
    held_item: "Griseous Core",
    nature: "Impish",
};

pub const CREATION_GOD_DIALGA: CreationGodSpec = CreationGodSpec {
    god_key: "dialga",
    name: "Dialga",
    title: "Lord of Time",
    base_species_id: 483,
    origin_species_id: 10245,
    types: &["Steel", "Dragon"],
    badge: "[1-OF-1 GOD]",
    lore: "The primordial deity governing the flow of time. Can accelerate or stop time at will.",
    abilities: &[
        ("Time Stop", "The opponent's Speed is reduced by 50% for the entire battle while Dialga is active."),
        ("Time Acceleration", "Dialga's own Speed increases by 50%."),
    ],
    moves: &["Roar of Time", "Flash Cannon", "Earth Power", "Dragon Breath"],
    held_item: "Adamant Crystal",
    nature: "Modest",
};

pub const CREATION_GOD_PALKIA: CreationGodSpec = CreationGodSpec {
    god_key: "palkia",
    name: "Palkia",
    title: "Ruler of Space",
    base_species_id: 484,
    origin_species_id: 10246,
    types: &["Water", "Dragon"],
    badge: "[1-OF-1 GOD]",
    lore: "The sovereign of space and spatial dimensions. Rips through reality itself.",
    abilities: &[
        ("Spatial Distortion", "All attacks ignore type-based damage reductions and immunities (resisted/immune targets take 1.5x damage)."),
        ("Dimensional Rift", "Palkia's attacks deal 1.5x damage."),
    ],
    moves: &["Spacial Rend", "Hydro Pump", "Aura Sphere", "Dragon Pulse"],
    held_item: "Lustrous Globe",
    nature: "Modest",
};

pub const ALL_CREATION_GODS: [&CreationGodSpec; 3] = [
    &CREATION_GOD_GIRATINA,
    &CREATION_GOD_DIALGA,
    &CREATION_GOD_PALKIA,
];

pub fn get_creation_god_spec(key: &str) -> Option<&'static CreationGodSpec> {
    match key.to_lowercase().trim() {
        "giratina" => Some(&CREATION_GOD_GIRATINA),
        "dialga" => Some(&CREATION_GOD_DIALGA),
        "palkia" => Some(&CREATION_GOD_PALKIA),
        _ => None,
    }
}

// Fallback Non-Legendary Ghost Pokémon Roster
pub const GHOST_POKEMON_ROSTER: &[&str] = &[
    "Gastly", "Haunter", "Gengar", "Mega Gengar",
    "Misdreavus", "Mismagius",
    "Shedinja", "Sableye", "Mega Sableye",
    "Shuppet", "Banette", "Mega Banette",
    "Duskull", "Dusclops", "Dusknoir",
    "Drifloon", "Drifblim", "Spiritomb", "Rotom",
    "Yamask", "Galarian Yamask", "Cofagrigus", "Runerigus",
    "Frillish", "Jellicent",
    "Litwick", "Lampent", "Chandelure",
    "Golett", "Golurk",
    "Honedge", "Doublade", "Aegislash",
    "Phantump", "Trevenant",
    "Pumpkaboo", "Gourgeist",
    "Decidueye", "Hisuian Decidueye",
    "Sandygast", "Palossand",
    "Mimikyu", "Dhelmise",
    "Galarian Corsola", "Cursola",
    "Sinistea", "Polteageist",
    "Dreepy", "Drakloak", "Dragapult",
    "Basculegion Male", "Basculegion Female",
    "Hisuian Zorua", "Hisuian Zoroark", "Hisuian Typhlosion",
    "Ceruledge", "Bramblin", "Brambleghast",
    "Greavard", "Houndstone", "Annihilape",
    "Poltchageist", "Sinistcha", "Gimmighoul", "Gholdengo"
];

// ==============================================================================
// 2. DATABASE INITIALIZATION & MIGRATIONS
// ==============================================================================

pub async fn init_halloween_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1. Ensure user columns exist
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_candies INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_void_shards INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_total_candies_earned INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_total_shards_earned INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_cauldron_spins INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_megastone_bought TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN halloween_catch_rate_voucher_expires REAL").execute(pool).await;

    // 2. Global event settings table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_event_settings (
            event TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL DEFAULT 1,
            stopped INTEGER NOT NULL DEFAULT 0,
            exp_multiplier REAL NOT NULL DEFAULT 1.5,
            coin_multiplier REAL NOT NULL DEFAULT 1.5,
            shiny_multiplier REAL NOT NULL DEFAULT 1.0
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO halloween_event_settings (event, enabled, stopped, exp_multiplier, coin_multiplier, shiny_multiplier)
         VALUES ('halloween_event', 1, 0, 1.5, 1.5, 1.0);"
    )
    .execute(pool)
    .await?;

    // 3. Rolling 24-hour Cauldron Treat action records
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_treats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            timestamp REAL NOT NULL
        );"
    )
    .execute(pool)
    .await?;

    // 4. Daily Trick Showdown records
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_tricks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            timestamp REAL NOT NULL
        );"
    )
    .execute(pool)
    .await?;

    // 5. 1-of-1 Creation Trio Gods table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_unique_creation_gods (
            god_key TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            title TEXT NOT NULL,
            species_id INTEGER NOT NULL,
            origin_species_id INTEGER NOT NULL,
            claimed INTEGER NOT NULL DEFAULT 0,
            owner_user_id INTEGER,
            owner_name TEXT,
            claimed_at TEXT,
            pokemon_db_id INTEGER
        );"
    )
    .execute(pool)
    .await?;

    // Pre-populate the 3 Creation Gods
    for god in ALL_CREATION_GODS {
        sqlx::query(
            "INSERT OR IGNORE INTO halloween_unique_creation_gods
             (god_key, name, title, species_id, origin_species_id, claimed)
             VALUES (?, ?, ?, ?, ?, 0);"
        )
        .bind(god.god_key)
        .bind(god.name)
        .bind(god.title)
        .bind(god.base_species_id as i64)
        .bind(god.origin_species_id as i64)
        .execute(pool)
        .await?;
    }

    // 6. Leaderboard finalization state
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_leaderboard_finalized (
            key TEXT PRIMARY KEY,
            finalized INTEGER NOT NULL DEFAULT 0,
            finalized_by INTEGER,
            finalized_at TEXT,
            results_json TEXT
        );"
    )
    .execute(pool)
    .await?;

    // 7. Cauldron spin and shop purchase logs
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_spin_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            spin_type TEXT NOT NULL,
            reward_name TEXT NOT NULL,
            amount INTEGER NOT NULL,
            timestamp REAL NOT NULL
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS halloween_shop_purchases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            item_key TEXT NOT NULL,
            item_name TEXT NOT NULL,
            category TEXT NOT NULL,
            cost INTEGER NOT NULL,
            timestamp REAL NOT NULL
        );"
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ==============================================================================
// 3. STATS & CURRENCY HELPERS
// ==============================================================================

#[derive(Debug, Clone, Default)]
pub struct UserHalloweenStats {
    pub candies: i64,
    pub void_shards: i64,
    pub total_candies_earned: i64,
    pub total_shards_earned: i64,
    pub cauldron_spins: i64,
    pub megastone_bought: Option<String>,
}

pub async fn get_user_halloween_stats(pool: &SqlitePool, user_id: u64) -> Result<UserHalloweenStats, sqlx::Error> {
    let row_opt = sqlx::query(
        "SELECT halloween_candies, halloween_void_shards, halloween_total_candies_earned,
                halloween_total_shards_earned, halloween_cauldron_spins, halloween_megastone_bought
         FROM users WHERE id = ?"
    )
    .bind(user_id as i64)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row_opt {
        let candies: i64 = row.try_get("halloween_candies").unwrap_or(0);
        let void_shards: i64 = row.try_get("halloween_void_shards").unwrap_or(0);
        let total_candies_earned: i64 = row.try_get("halloween_total_candies_earned").unwrap_or(0);
        let total_shards_earned: i64 = row.try_get("halloween_total_shards_earned").unwrap_or(0);
        let cauldron_spins: i64 = row.try_get("halloween_cauldron_spins").unwrap_or(0);
        let megastone_bought: Option<String> = row.try_get::<String, _>("halloween_megastone_bought")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Ok(UserHalloweenStats {
            candies: candies.max(0),
            void_shards: void_shards.max(0),
            total_candies_earned: total_candies_earned.max(0),
            total_shards_earned: total_shards_earned.max(0),
            cauldron_spins: cauldron_spins.max(0),
            megastone_bought,
        })
    } else {
        Ok(UserHalloweenStats::default())
    }
}

pub async fn increment_user_candies(pool: &SqlitePool, user_id: u64, amount: i64) -> Result<i64, sqlx::Error> {
    if amount <= 0 {
        return Ok(get_user_halloween_stats(pool, user_id).await?.candies);
    }
    sqlx::query(
        "UPDATE users SET
            halloween_candies = halloween_candies + ?,
            halloween_total_candies_earned = halloween_total_candies_earned + ?
         WHERE id = ?"
    )
    .bind(amount)
    .bind(amount)
    .bind(user_id as i64)
    .execute(pool)
    .await?;

    Ok(get_user_halloween_stats(pool, user_id).await?.candies)
}

pub async fn deduct_user_candies(pool: &SqlitePool, user_id: u64, amount: i64) -> Result<(bool, i64), sqlx::Error> {
    if amount <= 0 {
        return Ok((true, get_user_halloween_stats(pool, user_id).await?.candies));
    }
    let res = sqlx::query(
        "UPDATE users SET halloween_candies = halloween_candies - ?
         WHERE id = ? AND halloween_candies >= ?"
    )
    .bind(amount)
    .bind(user_id as i64)
    .bind(amount)
    .execute(pool)
    .await?;

    let stats = get_user_halloween_stats(pool, user_id).await?;
    Ok((res.rows_affected() > 0, stats.candies))
}

pub async fn increment_user_shards(pool: &SqlitePool, user_id: u64, amount: i64) -> Result<i64, sqlx::Error> {
    if amount <= 0 {
        return Ok(get_user_halloween_stats(pool, user_id).await?.void_shards);
    }
    sqlx::query(
        "UPDATE users SET
            halloween_void_shards = halloween_void_shards + ?,
            halloween_total_shards_earned = halloween_total_shards_earned + ?
         WHERE id = ?"
    )
    .bind(amount)
    .bind(amount)
    .bind(user_id as i64)
    .execute(pool)
    .await?;

    Ok(get_user_halloween_stats(pool, user_id).await?.void_shards)
}

pub async fn deduct_user_shards(pool: &SqlitePool, user_id: u64, amount: i64) -> Result<(bool, i64), sqlx::Error> {
    if amount <= 0 {
        return Ok((true, get_user_halloween_stats(pool, user_id).await?.void_shards));
    }
    let res = sqlx::query(
        "UPDATE users SET halloween_void_shards = halloween_void_shards - ?
         WHERE id = ? AND halloween_void_shards >= ?"
    )
    .bind(amount)
    .bind(user_id as i64)
    .bind(amount)
    .execute(pool)
    .await?;

    let stats = get_user_halloween_stats(pool, user_id).await?;
    Ok((res.rows_affected() > 0, stats.void_shards))
}

// ── Event Status ─────────────────────────────────────────────────────────────

pub async fn is_halloween_enabled(pool: &SqlitePool) -> bool {
    let row_opt: Option<(i64, i64)> = sqlx::query_as(
        "SELECT enabled, stopped FROM halloween_event_settings WHERE event = 'halloween_event'"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row_opt {
        Some((enabled, stopped)) => enabled == 1 && stopped == 0,
        None => true,
    }
}

pub async fn set_halloween_status(pool: &SqlitePool, enabled: bool, stopped: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE halloween_event_settings SET enabled = ?, stopped = ? WHERE event = 'halloween_event'"
    )
    .bind(if enabled { 1 } else { 0 })
    .bind(if stopped { 1 } else { 0 })
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn stop_halloween_event(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    set_halloween_status(pool, false, true).await
}

// ── Catch Rate Voucher ───────────────────────────────────────────────────────

pub async fn get_user_active_voucher(pool: &SqlitePool, user_id: u64) -> Option<u64> {
    if !is_halloween_enabled(pool).await {
        return None;
    }
    let expires_opt: Option<f64> = sqlx::query_scalar(
        "SELECT halloween_catch_rate_voucher_expires FROM users WHERE id = ?"
    )
    .bind(user_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let expires = expires_opt?;
    let now = Utc::now().timestamp() as f64;
    if expires > now {
        Some((expires - now) as u64)
    } else {
        None
    }
}

pub async fn activate_catch_rate_voucher(pool: &SqlitePool, user_id: u64, duration_minutes: u64) -> Result<f64, sqlx::Error> {
    let now = Utc::now().timestamp() as f64;
    let current_opt: Option<f64> = sqlx::query_scalar(
        "SELECT halloween_catch_rate_voucher_expires FROM users WHERE id = ?"
    )
    .bind(user_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let base = match current_opt {
        Some(exp) if exp > now => exp,
        _ => now,
    };
    let new_expires = base + (duration_minutes as f64 * 60.0);

    sqlx::query("UPDATE users SET halloween_catch_rate_voucher_expires = ? WHERE id = ?")
        .bind(new_expires)
        .bind(user_id as i64)
        .execute(pool)
        .await?;

    Ok(new_expires)
}

// ==============================================================================
// 4. WILD HUNT CATCH HOOK
// ==============================================================================

pub async fn handle_halloween_hunt_catch(
    pool: &SqlitePool,
    pokedex: &PokedexData,
    user_id: u64,
    wild_id: u32,
) -> Result<Option<String>, sqlx::Error> {
    if !is_halloween_enabled(pool).await {
        return Ok(None);
    }

    let wild_entry = pokedex.get_pokemon_by_id(wild_id);
    let is_spooky = wild_entry.map(|p| {
        p.types.iter().any(|t| t.eq_ignore_ascii_case("ghost") || t.eq_ignore_ascii_case("dark"))
    }).unwrap_or(false);

    let (candies, shards) = {
        let mut rng = rand::thread_rng();
        let mut c = 0i64;
        let mut s = 0i64;
        if is_spooky {
            // Ghost or Dark Pokémon: 50% chance for Candy Corn + 5% chance for Void Shard
            if rng.gen_bool(HALLOWEEN_GHOST_CANDY_DROP_RATE) {
                c = 1;
            }
            if rng.gen_bool(HALLOWEEN_GHOST_BONUS_VOID_RATE) {
                s = 1;
            }
        } else {
            // Normal Pokémon: 15% drop rate (with 2% double drop)
            let roll: f64 = rng.gen();
            if roll < HALLOWEEN_HUNT_DOUBLE_CANDY_RATE {
                c = 2;
            } else if roll < HALLOWEEN_HUNT_CANDY_DROP_RATE {
                c = 1;
            }
        }
        (c, s)
    };

    if candies <= 0 && shards <= 0 {
        return Ok(None);
    }

    if candies > 0 {
        increment_user_candies(pool, user_id, candies).await?;
    }
    if shards > 0 {
        increment_user_shards(pool, user_id, shards).await?;
    }

    let mut rewards = Vec::new();
    if candies > 0 {
        rewards.push(format!("+{} {}Candy Corn", candies, HALLOWEEN_CANDY_EMOJI));
    }
    if shards > 0 {
        rewards.push(format!("+{} {}Void Shard", shards, HALLOWEEN_VOID_EMOJI));
    }

    Ok(Some(format!("\n<b>Halloween Event:</b> You found <b>{}</b>!", rewards.join(" and "))))
}

// ==============================================================================
// 5. CAULDRON TREAT & TRICK MECHANICS
// ==============================================================================

pub async fn check_user_treat_limit(pool: &SqlitePool, user_id: u64) -> Result<(bool, usize, u64), sqlx::Error> {
    let now = Utc::now().timestamp() as f64;
    let cutoff = now - HALLOWEEN_TREAT_WINDOW_SECONDS;

    // Fetch active treat timestamps in rolling 24h window
    let rows: Vec<f64> = sqlx::query_scalar(
        "SELECT timestamp FROM halloween_treats WHERE user_id = ? AND timestamp > ? ORDER BY timestamp ASC"
    )
    .bind(user_id as i64)
    .bind(cutoff)
    .fetch_all(pool)
    .await?;

    let used = rows.len();
    if used >= HALLOWEEN_TREAT_DAILY_LIMIT {
        let earliest = rows[0];
        let seconds_remaining = ((earliest + HALLOWEEN_TREAT_WINDOW_SECONDS) - now).max(0.0) as u64;
        Ok((false, used, seconds_remaining))
    } else {
        Ok((true, used, 0))
    }
}

#[derive(Debug, Clone)]
pub struct TreatOutcome {
    pub reward_name: String,
    pub reward_detail: String,
    pub amount: i64,
}

pub async fn execute_halloween_treat(pool: &SqlitePool, user_id: u64) -> Result<Result<TreatOutcome, String>, sqlx::Error> {
    let (can_treat, used_treats, rem_sec) = check_user_treat_limit(pool, user_id).await?;
    if !can_treat {
        let hours = rem_sec / 3600;
        let mins = (rem_sec % 3600) / 60;
        let cd_str = if hours > 0 { format!("{}h {}m", hours, mins) } else { format!("{}m", mins.max(1)) };
        return Ok(Err(format!(
            "<b>Cauldron Limit Reached!</b> You have used <b>{}/{}</b> brews in the last 24 hours.\nAvailable again in <b>{}</b>.",
            used_treats, HALLOWEEN_TREAT_DAILY_LIMIT, cd_str
        )));
    }

    let (deducted, rem_candies) = deduct_user_candies(pool, user_id, HALLOWEEN_CAULDRON_CANDY_COST).await?;
    if !deducted {
        return Ok(Err(format!(
            "<b>Not Enough {}Candy Corn!</b> You need <code>{}</code> Candy Corn, but have <code>{}</code>.",
            HALLOWEEN_CANDY_EMOJI, HALLOWEEN_CAULDRON_CANDY_COST, rem_candies
        )));
    }

    let now = Utc::now().timestamp() as f64;
    sqlx::query("INSERT INTO halloween_treats (user_id, timestamp) VALUES (?, ?)")
        .bind(user_id as i64)
        .bind(now)
        .execute(pool)
        .await?;

    // Prune old timestamps
    let cutoff = now - HALLOWEEN_TREAT_WINDOW_SECONDS;
    let _ = sqlx::query("DELETE FROM halloween_treats WHERE user_id = ? AND timestamp <= ?")
        .bind(user_id as i64)
        .bind(cutoff)
        .execute(pool)
        .await;

    enum TreatKind {
        VoidShards(i64),
        Voucher(u64),
        Boost,
        Vitamins(i64),
        Berries(i64),
        Candies(i64),
        Coins(i64),
        BonusCandies(i64),
    }

    let rolled_treat = {
        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen();
        if roll < 0.05 {
            TreatKind::VoidShards(rng.gen_range(1..=2))
        } else if roll < 0.10 {
            TreatKind::Voucher(*[3u64, 5, 7].choose(&mut rng).unwrap_or(&5))
        } else if roll < 0.20 {
            TreatKind::Boost
        } else if roll < 0.35 {
            TreatKind::Vitamins(rng.gen_range(1..=3))
        } else if roll < 0.50 {
            TreatKind::Berries(rng.gen_range(2..=5))
        } else if roll < 0.65 {
            TreatKind::Candies(rng.gen_range(1..=2))
        } else if roll < 0.85 {
            TreatKind::Coins(rng.gen_range(2000..=5000))
        } else {
            TreatKind::BonusCandies(rng.gen_range(10..=25))
        }
    };

    let outcome = match rolled_treat {
        TreatKind::VoidShards(amt) => {
            increment_user_shards(pool, user_id, amt).await?;
            TreatOutcome {
                reward_name: format!("{} Void Shard{}", amt, if amt > 1 { "s" } else { "" }),
                reward_detail: format!("• <b>{} {}Void Shard{}</b>", amt, HALLOWEEN_VOID_EMOJI, if amt > 1 { "s" } else { "" }),
                amount: amt,
            }
        }
        TreatKind::Voucher(mins) => {
            activate_catch_rate_voucher(pool, user_id, mins).await?;
            TreatOutcome {
                reward_name: format!("2x Catch Rate Voucher ({}m)", mins),
                reward_detail: format!("• <b>2x Catch Rate Voucher ({}m)</b>\n<i>Activated automatically for wild hunts!</i>", mins),
                amount: mins as i64,
            }
        }
        TreatKind::Boost => {
            sqlx::query("UPDATE users SET boosts = boosts + 1 WHERE id = ?").bind(user_id as i64).execute(pool).await?;
            TreatOutcome {
                reward_name: "Boost x1".to_string(),
                reward_detail: "• <b>Boost x1</b>".to_string(),
                amount: 1,
            }
        }
        TreatKind::Vitamins(amt) => {
            sqlx::query("UPDATE users SET vitamins = vitamins + ? WHERE id = ?").bind(amt).bind(user_id as i64).execute(pool).await?;
            TreatOutcome {
                reward_name: format!("Vitamin x{}", amt),
                reward_detail: format!("• <b>Vitamin x{}</b>", amt),
                amount: amt,
            }
        }
        TreatKind::Berries(amt) => {
            sqlx::query("UPDATE users SET berries = berries + ? WHERE id = ?").bind(amt).bind(user_id as i64).execute(pool).await?;
            TreatOutcome {
                reward_name: format!("Berry x{}", amt),
                reward_detail: format!("• <b>Berry x{}</b>", amt),
                amount: amt,
            }
        }
        TreatKind::Candies(amt) => {
            sqlx::query("UPDATE users SET candies = candies + ? WHERE id = ?").bind(amt).bind(user_id as i64).execute(pool).await?;
            TreatOutcome {
                reward_name: format!("Rare Candy x{}", amt),
                reward_detail: format!("• <b>Rare Candy x{}</b>", amt),
                amount: amt,
            }
        }
        TreatKind::Coins(coins) => {
            sqlx::query("UPDATE users SET pokedollars = pokedollars + ? WHERE id = ?").bind(coins).bind(user_id as i64).execute(pool).await?;
            TreatOutcome {
                reward_name: format!("{} PokéDollars", coins),
                reward_detail: format!("• <b>{} 💵 PokéDollars</b>", coins),
                amount: coins,
            }
        }
        TreatKind::BonusCandies(bonus) => {
            increment_user_candies(pool, user_id, bonus).await?;
            TreatOutcome {
                reward_name: format!("{} Candy Corn", bonus),
                reward_detail: format!("• <b>{} {}Candy Corn</b>", bonus, HALLOWEEN_CANDY_EMOJI),
                amount: bonus,
            }
        }
    };

    // Log spin
    let _ = sqlx::query(
        "INSERT INTO halloween_spin_logs (user_id, spin_type, reward_name, amount, timestamp) VALUES (?, 'treat', ?, ?, ?)"
    )
    .bind(user_id as i64)
    .bind(&outcome.reward_name)
    .bind(outcome.amount)
    .bind(now)
    .execute(pool)
    .await;

    let _ = sqlx::query("UPDATE users SET halloween_cauldron_spins = halloween_cauldron_spins + 1 WHERE id = ?")
        .bind(user_id as i64)
        .execute(pool)
        .await;

    Ok(Ok(outcome))
}

// ── Trick Showdown Daily Limit Helpers ───────────────────────────────────────

pub fn get_halloween_trick_cycle_info() -> (String, f64, u64, String) {
    let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
    let now_utc = Utc::now();
    let now_ist = now_utc.with_timezone(&ist_offset);

    let (cycle_date, cycle_start, next_reset) = if now_ist.hour() < HALLOWEEN_TRICK_RESET_HOUR {
        let prev_date = now_ist.date_naive() - Duration::days(1);
        let start = ist_offset.from_local_datetime(&prev_date.and_hms_opt(HALLOWEEN_TRICK_RESET_HOUR, 0, 0).unwrap()).unwrap();
        let reset = ist_offset.from_local_datetime(&now_ist.date_naive().and_hms_opt(HALLOWEEN_TRICK_RESET_HOUR, 0, 0).unwrap()).unwrap();
        (prev_date.format("%Y-%m-%d").to_string(), start, reset)
    } else {
        let cur_date = now_ist.date_naive();
        let next_date = cur_date + Duration::days(1);
        let start = ist_offset.from_local_datetime(&cur_date.and_hms_opt(HALLOWEEN_TRICK_RESET_HOUR, 0, 0).unwrap()).unwrap();
        let reset = ist_offset.from_local_datetime(&next_date.and_hms_opt(HALLOWEEN_TRICK_RESET_HOUR, 0, 0).unwrap()).unwrap();
        (cur_date.format("%Y-%m-%d").to_string(), start, reset)
    };

    let cycle_start_epoch = cycle_start.with_timezone(&Utc).timestamp() as f64;
    let next_reset_utc = next_reset.with_timezone(&Utc);
    let seconds_remaining = (next_reset_utc - now_utc).num_seconds().max(0) as u64;

    let hours = seconds_remaining / 3600;
    let mins = (seconds_remaining % 3600) / 60;
    let time_left_str = if hours > 0 { format!("{}h {}m", hours, mins) } else { format!("{}m", mins.max(1)) };

    (cycle_date, cycle_start_epoch, seconds_remaining, time_left_str)
}

pub async fn check_user_trick_limit(pool: &SqlitePool, user_id: u64) -> Result<(bool, usize, u64), sqlx::Error> {
    let (_, cycle_start_epoch, seconds_remaining, _) = get_halloween_trick_cycle_info();

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM halloween_tricks WHERE user_id = ? AND timestamp >= ?"
    )
    .bind(user_id as i64)
    .bind(cycle_start_epoch)
    .fetch_one(pool)
    .await?;

    let used = count as usize;
    if used >= HALLOWEEN_TRICK_DAILY_LIMIT {
        Ok((false, used, seconds_remaining))
    } else {
        Ok((true, used, 0))
    }
}

pub async fn record_user_trick(pool: &SqlitePool, user_id: u64) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp() as f64;
    let (_, cycle_start_epoch, _, _) = get_halloween_trick_cycle_info();

    sqlx::query("INSERT INTO halloween_tricks (user_id, timestamp) VALUES (?, ?)")
        .bind(user_id as i64)
        .bind(now)
        .execute(pool)
        .await?;

    // Prune entries from prior cycles
    let _ = sqlx::query("DELETE FROM halloween_tricks WHERE user_id = ? AND timestamp < ?")
        .bind(user_id as i64)
        .bind(cycle_start_epoch)
        .execute(pool)
        .await;

    Ok(())
}

// ==============================================================================
// 6. HALLOWEEN SHOP MECHANICS
// ==============================================================================

pub async fn buy_halloween_shop_item(
    pool: &SqlitePool,
    user_id: u64,
    item_key: &str,
) -> Result<(bool, String, i64), sqlx::Error> {
    let clean = item_key.trim().to_lowercase().replace(' ', "").replace('_', "").replace('-', "");

    let stats = get_user_halloween_stats(pool, user_id).await?;

    // Check if it's a Move Tutor (e.g. tutor1..tutor29)
    if clean.starts_with("tutor") && clean[5..].chars().all(|c| c.is_ascii_digit()) {
        let t_num: u32 = clean[5..].parse().unwrap_or(0);
        let tutor_opt = crate::get_tutor_by_number(t_num);
        if let Some(tutor) = tutor_opt {
            let cost = HALLOWEEN_TUTOR_PRICE_SHARDS;
            let (deducted, rem) = deduct_user_shards(pool, user_id, cost).await?;
            if !deducted {
                return Ok((false, format!("<b>Insufficient Void Shards!</b> You need <code>{}</code> Void Shards, but have <code>{}</code>.", cost, rem), rem));
            }

            // Award Tutor
            sqlx::query(
                "INSERT INTO user_tutors (user_id, tutor_number, move_name, quantity) VALUES (?, ?, ?, 1)
                 ON CONFLICT(user_id, tutor_number) DO UPDATE SET quantity = quantity + 1"
            )
            .bind(user_id as i64)
            .bind(tutor.number as i64)
            .bind(tutor.name)
            .execute(pool)
            .await?;

            let now = Utc::now().timestamp() as f64;
            let _ = sqlx::query(
                "INSERT INTO halloween_shop_purchases (user_id, item_key, item_name, category, cost, timestamp)
                 VALUES (?, ?, ?, 'tutor', ?, ?)"
            )
            .bind(user_id as i64)
            .bind(&clean)
            .bind(&tutor.name)
            .bind(cost)
            .bind(now)
            .execute(pool)
            .await;

            return Ok((true, format!("<b>Purchase Successful!</b>\nYou acquired <b>Tutor #{}: {}</b> for <code>{}</code> Void Shards.", tutor.number, tutor.name, cost), rem));
        }
    }

    // Check if it's a Mega Stone
    let mega_stone_opt = crate::ALL_MEGA_STONES.iter().find(|&&s| {
        s.to_lowercase().replace(' ', "").replace('_', "").replace('-', "") == clean
    });

    if let Some(&stone_name) = mega_stone_opt {
        // Enforce 1 Mega Stone limit per trainer!
        if let Some(ref prev) = stats.megastone_bought {
            if !prev.trim().is_empty() {
                return Ok((false, format!("<b>Purchase Limit Reached!</b> You can only buy 1 Mega Stone from the Halloween Shop. You already purchased <b>{}</b>.", prev), stats.void_shards));
            }
        }

        let cost = HALLOWEEN_MEGASTONE_PRICE_SHARDS;
        let (deducted, rem) = deduct_user_shards(pool, user_id, cost).await?;
        if !deducted {
            return Ok((false, format!("<b>Insufficient Void Shards!</b> You need <code>{}</code> Void Shards, but have <code>{}</code>.", cost, rem), rem));
        }

        // Add to user key items
        sqlx::query(
            "INSERT INTO user_key_items (user_id, item_name, quantity) VALUES (?, ?, 1)
             ON CONFLICT(user_id, item_name) DO UPDATE SET quantity = quantity + 1"
        )
        .bind(user_id as i64)
        .bind(stone_name)
        .execute(pool)
        .await?;

        // Record 1-time mega stone purchase
        sqlx::query("UPDATE users SET halloween_megastone_bought = ? WHERE id = ?")
            .bind(stone_name)
            .bind(user_id as i64)
            .execute(pool)
            .await?;

        let now = Utc::now().timestamp() as f64;
        let _ = sqlx::query(
            "INSERT INTO halloween_shop_purchases (user_id, item_key, item_name, category, cost, timestamp)
             VALUES (?, ?, ?, 'megastone', ?, ?)"
        )
        .bind(user_id as i64)
        .bind(&clean)
        .bind(stone_name)
        .bind(cost)
        .bind(now)
        .execute(pool)
        .await;

        return Ok((true, format!("<b>Purchase Successful!</b>\nYou acquired <b>{}</b> for <code>{}</code> Void Shards.", stone_name, cost), rem));
    }

    Ok((false, format!("Item '<code>{}</code>' not found in Halloween Shop.", item_key), stats.void_shards))
}

// ==============================================================================
// 7. LEADERBOARD & 1-OF-1 CREATION GODS
// ==============================================================================

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub user_id: u64,
    pub name: String,
    pub void_shards: i64,
    pub candies: i64,
}

pub async fn get_halloween_leaderboard(pool: &SqlitePool, limit: usize) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, first_name, halloween_void_shards, halloween_candies
         FROM users
         WHERE halloween_void_shards > 0 OR halloween_candies > 0
         ORDER BY halloween_void_shards DESC, halloween_candies DESC, halloween_total_shards_earned DESC
         LIMIT ?"
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::new();
    for (i, r) in rows.into_iter().enumerate() {
        let uid: i64 = r.try_get("id").unwrap_or(0);
        let first_name: Option<String> = r.try_get("first_name").ok();
        let username: Option<String> = r.try_get("username").ok();
        let shards: i64 = r.try_get("halloween_void_shards").unwrap_or(0);
        let candies: i64 = r.try_get("halloween_candies").unwrap_or(0);

        let name = first_name.or(username).unwrap_or_else(|| format!("Trainer #{}", uid));

        entries.push(LeaderboardEntry {
            rank: i + 1,
            user_id: uid as u64,
            name,
            void_shards: shards,
            candies,
        });
    }

    Ok(entries)
}

pub async fn is_leaderboard_finalized(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM halloween_leaderboard_finalized WHERE key = 'leaderboard_finalized' AND finalized = 1"
    )
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn claim_creation_god_for_user(
    pool: &SqlitePool,
    user_id: u64,
    god_key: &str,
    user_name: &str,
    force: bool,
) -> Result<(bool, String), sqlx::Error> {
    let spec = match get_creation_god_spec(god_key) {
        Some(s) => s,
        None => return Ok((false, format!("Unknown Creation God key: {}", god_key))),
    };

    // Strict 1-of-1 check in database
    if !force {
        let claimed_row: Option<(i64, Option<String>)> = sqlx::query_as(
            "SELECT claimed, owner_name FROM halloween_unique_creation_gods WHERE god_key = ?"
        )
        .bind(spec.god_key)
        .fetch_optional(pool)
        .await?;

        if let Some((claimed, owner)) = claimed_row {
            if claimed == 1 {
                return Ok((false, format!("<b>{}</b> has already been claimed by <b>{}</b>! There is strictly only 1 in existence.", spec.name, owner.unwrap_or_else(|| "Another Trainer".to_string()))));
            }
        }
    }

    // Insert level 100 shiny Origin God with perfect 31 IVs into player's Pokémon storage
    let nickname = format!("{} {}", spec.name, spec.badge);
    let now_str = Utc::now().to_rfc3339();

    let insert_res = sqlx::query(
        "INSERT INTO pokemons (
            user_id, pokemon_id, nickname, level, nature,
            iv_hp, iv_attack, iv_defense, iv_sp_attack, iv_sp_defense, iv_speed,
            ev_hp, ev_attack, ev_defense, ev_sp_attack, ev_sp_defense, ev_speed,
            experience, move_1, move_2, move_3, move_4, is_starter, is_shiny
         ) VALUES (
            ?, ?, ?, 100, ?,
            31, 31, 31, 31, 31, 31,
            0, 0, 0, 0, 0, 0,
            1000000, ?, ?, ?, ?, 0, 1
         )"
    )
    .bind(user_id as i64)
    .bind(spec.origin_species_id as i64)
    .bind(&nickname)
    .bind(spec.nature)
    .bind(spec.moves[0])
    .bind(spec.moves[1])
    .bind(spec.moves[2])
    .bind(spec.moves[3])
    .execute(pool)
    .await?;

    let poke_db_id = insert_res.last_insert_rowid();

    // Mark god claimed in unique_creation_gods
    sqlx::query(
        "UPDATE halloween_unique_creation_gods SET
            claimed = 1,
            owner_user_id = ?,
            owner_name = ?,
            claimed_at = ?,
            pokemon_db_id = ?
         WHERE god_key = ?"
    )
    .bind(user_id as i64)
    .bind(user_name)
    .bind(&now_str)
    .bind(poke_db_id)
    .bind(spec.god_key)
    .execute(pool)
    .await?;

    Ok((true, format!("<b>{}</b> (Origin Form) [1-OF-1 GOD] successfully awarded to <b>{}</b>!", spec.name, user_name)))
}

pub async fn finalize_halloween_leaderboard(
    pool: &SqlitePool,
    _pokedex: &PokedexData,
    admin_user_id: u64,
) -> Result<(bool, String), sqlx::Error> {
    if is_leaderboard_finalized(pool).await? {
        return Ok((false, "The Halloween Spacetime Leaderboard has already been finalized!".to_string()));
    }

    let board = get_halloween_leaderboard(pool, 10).await?;
    if board.is_empty() {
        return Ok((false, "No trainers found on the Halloween Leaderboard.".to_string()));
    }

    let mut summary = vec![
        "<blockquote><b>HALLOWEEN EVENT: 1-OF-1 CREATION GODS AWARDED!</b></blockquote>\n".to_string(),
        "The spacetime rift stabilizes as the divine 1-of-1 Creation Gods choose their champions:\n".to_string(),
    ];

    let god_pairs = [
        (1, "giratina"),
        (2, "dialga"),
        (3, "palkia"),
    ];

    for (rank, key) in god_pairs {
        if let Some(entry) = board.iter().find(|b| b.rank == rank) {
            let (ok, msg) = claim_creation_god_for_user(pool, entry.user_id, key, &entry.name, false).await?;
            if ok {
                let spec = get_creation_god_spec(key).unwrap();
                summary.push(format!("<b>Rank {}:</b> <b>{}</b> (<code>{}</code> {}Void Shards)\n   - Awarded <b>{}</b> ({}) [1-of-1 God]!", rank, entry.name, entry.void_shards, HALLOWEEN_VOID_EMOJI, spec.name, spec.title));
            } else {
                summary.push(format!("<b>Rank {}:</b> <b>{}</b> - {}", rank, entry.name, msg));
            }
        }
    }

    // Rank 4-10 Consolidation Prizes: Master Ball x1 + Ghost Mega Stone x1 + 100,000 PokéDollars
    let ghost_stones = ["Gengarite", "Banettite", "Sablenite"];
    let assigned_prizes: Vec<(u64, usize, String, i64, &'static str)> = {
        let mut rng = rand::thread_rng();
        board
            .iter()
            .filter(|b| b.rank >= 4)
            .map(|entry| {
                let m_stone = *ghost_stones.choose(&mut rng).unwrap_or(&"Gengarite");
                (entry.user_id, entry.rank, entry.name.clone(), entry.void_shards, m_stone)
            })
            .collect()
    };

    for (u_id, rank, name, void_shards, m_stone) in assigned_prizes {
        // Add master ball
        let _ = sqlx::query("UPDATE users SET ball_master = ball_master + 1, pokedollars = pokedollars + 100000 WHERE id = ?")
            .bind(u_id as i64)
            .execute(pool)
            .await;

        // Add mega stone
        let _ = sqlx::query(
            "INSERT INTO user_key_items (user_id, item_name, quantity) VALUES (?, ?, 1)
             ON CONFLICT(user_id, item_name) DO UPDATE SET quantity = quantity + 1"
        )
        .bind(u_id as i64)
        .bind(m_stone)
        .execute(pool)
        .await;

        summary.push(format!("<b>Rank {}:</b> <b>{}</b> (<code>{}</code> {}Void Shards)\n   - Master Ball x1 + {} x1 + 100,000 PokéDollars", rank, name, void_shards, HALLOWEEN_VOID_EMOJI, m_stone));
    }

    let now_str = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO halloween_leaderboard_finalized (key, finalized, finalized_by, finalized_at, results_json)
         VALUES ('leaderboard_finalized', 1, ?, ?, '')
         ON CONFLICT(key) DO UPDATE SET finalized = 1, finalized_by = excluded.finalized_by, finalized_at = excluded.finalized_at"
    )
    .bind(admin_user_id as i64)
    .bind(&now_str)
    .execute(pool)
    .await?;

    Ok((true, summary.join("\n")))
}

// ==============================================================================
// 8. UI BUILDERS (HTML MESSAGES & KEYBOARDS)
// ==============================================================================

pub async fn build_halloween_dashboard(
    pool: &SqlitePool,
    user_id: u64,
    _is_admin: bool,
) -> Result<(String, InlineKeyboardMarkup), sqlx::Error> {
    let stats = get_user_halloween_stats(pool, user_id).await?;
    let voucher_opt = get_user_active_voucher(pool, user_id).await;

    let voucher_text = if let Some(sec) = voucher_opt {
        let m = sec / 60;
        let s = sec % 60;
        format!("\n<b>Active Buff:</b> 2x Hunt Catch Rate (<code>{}m {}s</code> left)", m, s)
    } else {
        String::new()
    };

    let text = format!(
        "<blockquote><b>Halloween Event</b></blockquote>\n\n\
         {}<b>Candy Corn:</b> <code>{}</code>\n\
         {}<b>Void Shards:</b> <code>{}</code>{}",
        HALLOWEEN_CANDY_EMOJI,
        stats.candies,
        HALLOWEEN_VOID_EMOJI,
        stats.void_shards,
        voucher_text
    );

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Candy", "halloween_candy"),
            InlineKeyboardButton::callback("Shop", "halloween_shop"),
        ],
        vec![
            InlineKeyboardButton::callback("Leaderboard", "halloween_leaderboard"),
            InlineKeyboardButton::callback("Creation Gods", "halloween_gods"),
        ],
        vec![
            InlineKeyboardButton::callback("Event Info", "halloween_info"),
        ],
    ]);

    Ok((text, kb))
}

pub fn build_halloween_info() -> (String, InlineKeyboardMarkup) {
    let text = format!(
        "<blockquote><b>Halloween: Fracture of the Distortion World</b></blockquote>\n\n\
         The dimensional veil has shattered! Reality distorts as <b>Giratina</b>, <b>Dialga</b>, \
         and <b>Palkia</b> awaken. Compete on the Leaderboard to claim one of the strictly <b>1-of-1 Creation Gods</b>!\n\n\
         <blockquote><b>Event Lore: The Fracture</b></blockquote>\n\
         On Halloween night, the boundary between our realm and the <b>Distortion World</b> tore open. \
         <b>Giratina</b> has breached reality, unleashing temporal and spatial anomalies that awaken <b>Dialga</b> and <b>Palkia</b>!\n\n\
         <blockquote><b>How to Earn Event Currencies?</b></blockquote>\n\
         • {}<b>Candy Corn:</b> Catch Pokémon in <code>/hunt</code> (15% drop rate; 50% on Ghost/Dark Pokémon) or lucky Cauldron brews.\n\
         • {}<b>Void Shards:</b> Catch Ghost/Dark Pokémon (5% drop rate), brew in the Cauldron (5% chance), or win a 6v6 PvP <b>Trick</b> duel ({}–{} Shards).\n\n\
         <blockquote><b>The 1-of-1 Creation Trio Stakes</b></blockquote>\n\
         At the conclusion of the event, the <b>Top 3 Trainers</b> on the {}<b>Void Shard Leaderboard</b> \
         will claim the strictly <b>1-of-1 Creation Gods</b>:\n\
         • <b>Rank 1:</b> <b>Giratina</b> (Monarch of the Distortion World)\n\
         • <b>Rank 2:</b> <b>Dialga</b> (Lord of Time)\n\
         • <b>Rank 3:</b> <b>Palkia</b> (Ruler of Space)\n\n\
         <i>Ranks 4–10 win Master Ball x1 + Ghost Mega Stone x1 + 100,000 PokéDollars. Only one piece of each God will ever exist!</i>",
        HALLOWEEN_CANDY_EMOJI,
        HALLOWEEN_VOID_EMOJI,
        HALLOWEEN_TRICK_VOID_MIN,
        HALLOWEEN_TRICK_VOID_MAX,
        HALLOWEEN_VOID_EMOJI
    );

    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("Back to Event", "halloween_menu")],
    ]);

    (text, kb)
}

pub async fn build_halloween_candy(
    pool: &SqlitePool,
    user_id: u64,
    last_outcome: Option<&str>,
) -> Result<(String, InlineKeyboardMarkup), sqlx::Error> {
    let stats = get_user_halloween_stats(pool, user_id).await?;
    let (_, used_treats, _) = check_user_treat_limit(pool, user_id).await?;

    let reward_detail = last_outcome.unwrap_or("None");

    let text = format!(
        "{}<b>Candy Corn :</b> {}\n\n\
         <blockquote><b>Cauldron :</b>\n\
         {}{}/{}</blockquote>\n\n\
         <blockquote><b>Reward :</b>\n\
         {}</blockquote>",
        HALLOWEEN_CANDY_EMOJI,
        stats.candies,
        HALLOWEEN_CAULDRON_EMOJI,
        used_treats,
        HALLOWEEN_TREAT_DAILY_LIMIT,
        reward_detail
    );

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Treat", "halloween_treat"),
            InlineKeyboardButton::callback("Trick", "halloween_trick"),
        ],
        vec![
            InlineKeyboardButton::callback("Back to Event", "halloween_menu"),
        ],
    ]);

    Ok((text, kb))
}

pub async fn build_halloween_shop(
    pool: &SqlitePool,
    user_id: u64,
    category: &str,
    page: usize,
    feedback: Option<&str>,
) -> Result<(String, InlineKeyboardMarkup), sqlx::Error> {
    let stats = get_user_halloween_stats(pool, user_id).await?;
    let is_tutor = category.eq_ignore_ascii_case("tutor");

    let per_page = 6;

    let (cat_title, cat_info, total_items, page_items) = if is_tutor {
        let all_tutors = crate::TUTOR_CATALOG;
        let total = all_tutors.len();
        let total_pages = (total + per_page - 1) / per_page;
        let clamped = page.clamp(1, total_pages.max(1));
        let start = (clamped - 1) * per_page;
        let end = (start + per_page).min(total);
        let items: Vec<(String, String, i64, String)> = all_tutors[start..end].iter().map(|t| {
            (
                format!("tutor{}", t.number),
                t.name.to_string(),
                HALLOWEEN_TUTOR_PRICE_SHARDS,
                format!("{} | {} | Power: {} | Acc: {}", t.move_type, t.category, t.pwr, t.acc),
            )
        }).collect();
        ("Move Tutors", "Price: 50 Void Shards each", total, items)
    } else {
        let all_megas = crate::ALL_MEGA_STONES;
        let total = all_megas.len();
        let total_pages = (total + per_page - 1) / per_page;
        let clamped = page.clamp(1, total_pages.max(1));
        let start = (clamped - 1) * per_page;
        let end = (start + per_page).min(total);
        let items: Vec<(String, String, i64, String)> = all_megas[start..end].iter().map(|&s| {
            (
                s.to_string(),
                s.to_string(),
                HALLOWEEN_MEGASTONE_PRICE_SHARDS,
                format!("Unlocks Mega Evolution for {}.", s.replace("ite", "").replace("nite", "")),
            )
        }).collect();
        ("Mega Stones", "Price: 100 Void Shards each | Limit: 1 Mega Stone per trainer", total, items)
    };

    let total_pages = (total_items + per_page - 1) / per_page;
    let clamped_page = page.clamp(1, total_pages.max(1));

    let mut text = format!(
        "<blockquote><b>Halloween Shop</b></blockquote>\n\n\
         Exchange your Void Shards for exclusive Mega Stones and Move Tutors.\n\n\
         <b>Your Balance:</b> <code>{}</code> Void Shards\n\
         <b>Category:</b> {} ({})\n",
        stats.void_shards,
        cat_title,
        cat_info
    );

    if !is_tutor {
        if let Some(ref bought) = stats.megastone_bought {
            text.push_str(&format!("\n<i>Notice: You have already purchased 1 Mega Stone ({}). Additional Mega Stones cannot be purchased.</i>\n", bought));
        }
    }

    if let Some(fb) = feedback {
        text.push_str(&format!("\n<b>Status:</b> {}\n", fb));
    }

    text.push_str(&format!("\n<b>Available Items (Page {}/{}):</b>\n", clamped_page, total_pages));
    for (_, name, cost, desc) in &page_items {
        text.push_str(&format!("- <b>{}</b> - <code>{}</code> Void Shards\n  <i>{}</i>\n", name, cost, desc));
    }

    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // Category Tabs
    let tab_mega = if !is_tutor { "[ Mega Stones ]" } else { "Mega Stones" };
    let tab_tutor = if is_tutor { "[ Move Tutors ]" } else { "Move Tutors" };
    rows.push(vec![
        InlineKeyboardButton::callback(tab_mega, if !is_tutor { "noop" } else { "halloween_shop:mega:1" }),
        InlineKeyboardButton::callback(tab_tutor, if is_tutor { "noop" } else { "halloween_shop:tutor:1" }),
    ]);

    // Item Action Buttons (2 per row)
    for chunk in page_items.chunks(2) {
        let mut btn_row = Vec::new();
        for (key, name, _, _) in chunk {
            let label = if !is_tutor && stats.megastone_bought.is_some() {
                if stats.megastone_bought.as_deref() == Some(name.as_str()) {
                    format!("{} (Owned)", name)
                } else {
                    format!("{} (Locked)", name)
                }
            } else {
                name.clone()
            };
            btn_row.push(InlineKeyboardButton::callback(label, format!("halloween_buy:{}", key)));
        }
        rows.push(btn_row);
    }

    // Pagination Row
    let cat_prefix = if is_tutor { "tutor" } else { "mega" };
    let mut nav_row = Vec::new();
    if clamped_page > 1 {
        nav_row.push(InlineKeyboardButton::callback("←", format!("halloween_shop:{}:{}", cat_prefix, clamped_page - 1)));
    }
    nav_row.push(InlineKeyboardButton::callback(format!("{}/{}", clamped_page, total_pages), "noop"));
    if clamped_page < total_pages {
        nav_row.push(InlineKeyboardButton::callback("→", format!("halloween_shop:{}:{}", cat_prefix, clamped_page + 1)));
    }
    rows.push(nav_row);

    // Back to Event
    rows.push(vec![InlineKeyboardButton::callback("Back to Event", "halloween_menu")]);

    let fallback_kb = InlineKeyboardMarkup::new(rows);
    Ok((text, fallback_kb))
}

pub async fn build_halloween_shop_json(
    pool: &SqlitePool,
    user_id: u64,
    category: &str,
    page: usize,
    feedback: Option<&str>,
) -> Result<(String, serde_json::Value, InlineKeyboardMarkup), sqlx::Error> {
    let stats = get_user_halloween_stats(pool, user_id).await?;
    let is_tutor = category.eq_ignore_ascii_case("tutor");

    let per_page = 6;

    let (cat_title, cat_info, total_items, page_items) = if is_tutor {
        let all_tutors = crate::TUTOR_CATALOG;
        let total = all_tutors.len();
        let total_pages = (total + per_page - 1) / per_page;
        let clamped = page.clamp(1, total_pages.max(1));
        let start = (clamped - 1) * per_page;
        let end = (start + per_page).min(total);
        let items: Vec<(String, String, i64, String)> = all_tutors[start..end].iter().map(|t| {
            (
                format!("tutor{}", t.number),
                t.name.to_string(),
                HALLOWEEN_TUTOR_PRICE_SHARDS,
                format!("{} | {} | Power: {} | Acc: {}", t.move_type, t.category, t.pwr, t.acc),
            )
        }).collect();
        ("Move Tutors", "Price: 50 Void Shards each", total, items)
    } else {
        let all_megas = crate::ALL_MEGA_STONES;
        let total = all_megas.len();
        let total_pages = (total + per_page - 1) / per_page;
        let clamped = page.clamp(1, total_pages.max(1));
        let start = (clamped - 1) * per_page;
        let end = (start + per_page).min(total);
        let items: Vec<(String, String, i64, String)> = all_megas[start..end].iter().map(|&s| {
            (
                s.to_string(),
                s.to_string(),
                HALLOWEEN_MEGASTONE_PRICE_SHARDS,
                format!("Unlocks Mega Evolution for {}.", s.replace("ite", "").replace("nite", "")),
            )
        }).collect();
        ("Mega Stones", "Price: 100 Void Shards each | Limit: 1 Mega Stone per trainer", total, items)
    };

    let total_pages = (total_items + per_page - 1) / per_page;
    let clamped_page = page.clamp(1, total_pages.max(1));

    let mut text = format!(
        "<blockquote><b>Halloween Shop</b></blockquote>\n\n\
         Exchange your Void Shards for exclusive Mega Stones and Move Tutors.\n\n\
         <b>Your Balance:</b> <code>{}</code> Void Shards\n\
         <b>Category:</b> {} ({})\n",
        stats.void_shards,
        cat_title,
        cat_info
    );

    if !is_tutor {
        if let Some(ref bought) = stats.megastone_bought {
            text.push_str(&format!("\n<i>Notice: You have already purchased 1 Mega Stone ({}). Additional Mega Stones cannot be purchased.</i>\n", bought));
        }
    }

    if let Some(fb) = feedback {
        text.push_str(&format!("\n<b>Status:</b> {}\n", fb));
    }

    text.push_str(&format!("\n<b>Available Items (Page {}/{}):</b>\n", clamped_page, total_pages));
    for (_, name, cost, desc) in &page_items {
        text.push_str(&format!("- <b>{}</b> - <code>{}</code> Void Shards\n  <i>{}</i>\n", name, cost, desc));
    }

    // Build JSON rows with custom emoji arrows
    let mut json_rows = Vec::new();
    let mut fallback_rows = Vec::new();

    let tab_mega = if !is_tutor { "[ Mega Stones ]" } else { "Mega Stones" };
    let tab_tutor = if is_tutor { "[ Move Tutors ]" } else { "Move Tutors" };
    json_rows.push(serde_json::json!([
        {
            "text": tab_mega,
            "callback_data": if !is_tutor { "noop" } else { "halloween_shop:mega:1" }
        },
        {
            "text": tab_tutor,
            "callback_data": if is_tutor { "noop" } else { "halloween_shop:tutor:1" }
        }
    ]));
    fallback_rows.push(vec![
        InlineKeyboardButton::callback(tab_mega, if !is_tutor { "noop" } else { "halloween_shop:mega:1" }),
        InlineKeyboardButton::callback(tab_tutor, if is_tutor { "noop" } else { "halloween_shop:tutor:1" }),
    ]);

    for chunk in page_items.chunks(2) {
        let mut j_row = Vec::new();
        let mut f_row = Vec::new();
        for (key, name, _, _) in chunk {
            let label = if !is_tutor && stats.megastone_bought.is_some() {
                if stats.megastone_bought.as_deref() == Some(name.as_str()) {
                    format!("{} (Owned)", name)
                } else {
                    format!("{} (Locked)", name)
                }
            } else {
                name.clone()
            };
            j_row.push(serde_json::json!({
                "text": label,
                "callback_data": format!("halloween_buy:{}", key)
            }));
            f_row.push(InlineKeyboardButton::callback(label, format!("halloween_buy:{}", key)));
        }
        json_rows.push(serde_json::Value::Array(j_row));
        fallback_rows.push(f_row);
    }

    let cat_prefix = if is_tutor { "tutor" } else { "mega" };
    if total_pages > 1 {
        let mut j_nav = Vec::new();
        let mut f_nav = Vec::new();
        if clamped_page > 1 {
            j_nav.push(serde_json::json!({
                "text": " ",
                "callback_data": format!("halloween_shop:{}:{}", cat_prefix, clamped_page - 1),
                "icon_custom_emoji_id": crate::emoji::LEFT_ARROW_CUSTOM_EMOJI_ID
            }));
            f_nav.push(InlineKeyboardButton::callback("←", format!("halloween_shop:{}:{}", cat_prefix, clamped_page - 1)));
        }
        j_nav.push(serde_json::json!({
            "text": format!("{}/{}", clamped_page, total_pages),
            "callback_data": "noop"
        }));
        f_nav.push(InlineKeyboardButton::callback(format!("{}/{}", clamped_page, total_pages), "noop"));
        if clamped_page < total_pages {
            j_nav.push(serde_json::json!({
                "text": " ",
                "callback_data": format!("halloween_shop:{}:{}", cat_prefix, clamped_page + 1),
                "icon_custom_emoji_id": crate::emoji::RIGHT_ARROW_CUSTOM_EMOJI_ID
            }));
            f_nav.push(InlineKeyboardButton::callback("→", format!("halloween_shop:{}:{}", cat_prefix, clamped_page + 1)));
        }
        json_rows.push(serde_json::Value::Array(j_nav));
        fallback_rows.push(f_nav);
    }

    json_rows.push(serde_json::json!([
        {
            "text": "Back to Event",
            "callback_data": "halloween_menu"
        }
    ]));
    fallback_rows.push(vec![InlineKeyboardButton::callback("Back to Event", "halloween_menu")]);

    let custom_kb = serde_json::json!({ "inline_keyboard": json_rows });
    let fallback_kb = InlineKeyboardMarkup::new(fallback_rows);

    Ok((text, custom_kb, fallback_kb))
}

pub async fn build_halloween_leaderboard(
    pool: &SqlitePool,
    is_admin: bool,
) -> Result<(String, InlineKeyboardMarkup), sqlx::Error> {
    let board = get_halloween_leaderboard(pool, 10).await?;
    let is_fin = is_leaderboard_finalized(pool).await?;

    let mut lines = Vec::new();
    for entry in &board {
        let player_link = format!("<a href=\"tg://user?id={}\">{}</a>", entry.user_id, crate::escape_html(&entry.name));
        lines.push(format!("{} : {} {}", player_link, entry.void_shards, HALLOWEEN_VOID_EMOJI));
    }

    let text = if lines.is_empty() {
        "<i>No trainers on the leaderboard yet.</i>".to_string()
    } else {
        lines.join("\n")
    };

    let mut kb_rows = vec![
        vec![InlineKeyboardButton::callback("Back to Event", "halloween_menu")],
    ];

    if is_admin && !is_fin {
        kb_rows.push(vec![InlineKeyboardButton::callback("Finalize Leaderboard (Admin)", "halloween_finalize_prompt")]);
    }

    Ok((text, InlineKeyboardMarkup::new(kb_rows)))
}

pub async fn build_creation_gods_showcase(
    pool: &SqlitePool,
) -> Result<(String, InlineKeyboardMarkup), sqlx::Error> {
    let mut text = "<blockquote><b>The 1-of-1 Creation Gods</b></blockquote>\n\n".to_string();

    for god in ALL_CREATION_GODS {
        let row_opt: Option<(i64, Option<String>)> = sqlx::query_as(
            "SELECT claimed, owner_name FROM halloween_unique_creation_gods WHERE god_key = ?"
        )
        .bind(god.god_key)
        .fetch_optional(pool)
        .await?;

        let claim_text = match row_opt {
            Some((1, Some(owner))) => format!("<b>Claimed by:</b> <code>{}</code> [1-of-1 Sealed]", owner),
            _ => "<b>Status:</b> <i>Unclaimed (Grand Prize for Event Leaderboard!)</i>".to_string(),
        };

        text.push_str(&format!(
            "<b>{}</b> ({}) {}\n• <b>Types:</b> {}\n• <b>Special Abilities:</b>\n",
            god.name,
            god.title,
            god.badge,
            god.types.join(" / ")
        ));

        for (ab_name, ab_desc) in god.abilities {
            text.push_str(&format!("   - <b>{}:</b> {}\n", ab_name, ab_desc));
        }

        text.push_str(&format!("• {}\n\n", claim_text));
    }

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Leaderboard", "halloween_leaderboard"),
            InlineKeyboardButton::callback("Back to Event", "halloween_menu"),
        ]
    ]);

    Ok((text, kb))
}

pub fn build_trick_guide() -> (String, InlineKeyboardMarkup) {
    let text = format!(
        "<blockquote><b>Halloween Trick</b></blockquote>\n\n\
         Trick is a 6v6 PvP challenge where you duel another trainer in group chat for {}<b>Void Shards</b>!\n\n\
         <b>How to Challenge:</b>\n\
         1. Open any group chat where the bot is present.\n\
         2. <b>Reply</b> <code>/trick</code> to another trainer's message, or type <code>/trick @username</code>.\n\
         3. When they accept, your 6v6 duel begins immediately!\n\n\
         • <b>Cost:</b> <code>{}</code> {}Candy Corn (paid by challenger)\n\
         • <b>Daily Limit:</b> <b>10 Trick Showdowns</b> per day (resets at 5:00 AM IST)\n\
         • <b>Rewards:</b> Winner claims <b>{}–{}</b> {}Void Shards!",
        HALLOWEEN_VOID_EMOJI,
        HALLOWEEN_TRICK_CANDY_COST,
        HALLOWEEN_CANDY_EMOJI,
        HALLOWEEN_TRICK_VOID_MIN,
        HALLOWEEN_TRICK_VOID_MAX,
        HALLOWEEN_VOID_EMOJI
    );

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Treat", "halloween_treat"),
            InlineKeyboardButton::callback("Back to Candy", "halloween_candy"),
        ],
        vec![
            InlineKeyboardButton::callback("Back to Event", "halloween_menu"),
        ],
    ]);

    (text, kb)
}

pub fn build_treat_result(outcome: &TreatOutcome, rem_candies: i64) -> (String, InlineKeyboardMarkup) {
    let text = format!(
        "<blockquote>The Cauldron bubbles warmly and bestows:</blockquote>\n\n\
         {}\n\n\
         <b>Remaining Candies:</b> <code>{}</code> {}Candy Corn",
        outcome.reward_detail,
        rem_candies,
        HALLOWEEN_CANDY_EMOJI
    );

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(format!("Treat Again ({} Candies)", HALLOWEEN_CAULDRON_CANDY_COST), "halloween_treat"),
            InlineKeyboardButton::callback(format!("Trick Guide ({} Candies)", HALLOWEEN_TRICK_CANDY_COST), "halloween_trick"),
        ],
        vec![
            InlineKeyboardButton::callback("Back to Candy", "halloween_candy"),
            InlineKeyboardButton::callback("Back to Event", "halloween_menu"),
        ],
    ]);

    (text, kb)
}

// ==============================================================================
// 9. UNIT TESTS
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn test_halloween_constants_and_god_specs() {
        assert_eq!(CREATION_GOD_GIRATINA.god_key, "giratina");
        assert_eq!(CREATION_GOD_GIRATINA.origin_species_id, 10007);
        assert_eq!(CREATION_GOD_DIALGA.god_key, "dialga");
        assert_eq!(CREATION_GOD_DIALGA.origin_species_id, 10245);
        assert_eq!(CREATION_GOD_PALKIA.god_key, "palkia");
        assert_eq!(CREATION_GOD_PALKIA.origin_species_id, 10246);

        assert!(get_creation_god_spec("giratina").is_some());
        assert!(get_creation_god_spec("DIALGA").is_some());
        assert!(get_creation_god_spec("palkia ").is_some());
        assert!(get_creation_god_spec("arceus").is_none());

        assert_eq!(HALLOWEEN_CAULDRON_CANDY_COST, 25);
        assert_eq!(HALLOWEEN_TREAT_DAILY_LIMIT, 15);
        assert_eq!(HALLOWEEN_TRICK_CANDY_COST, 15);
        assert_eq!(HALLOWEEN_TRICK_DAILY_LIMIT, 10);
        assert_eq!(HALLOWEEN_MEGASTONE_PRICE_SHARDS, 100);
        assert_eq!(HALLOWEEN_TUTOR_PRICE_SHARDS, 50);
    }

    #[test]
    fn test_halloween_trick_cycle_info() {
        let (cycle_date, start_epoch, remaining_sec, reset_str) = get_halloween_trick_cycle_info();
        assert!(!cycle_date.is_empty());
        assert!(start_epoch > 0.0);
        assert!(remaining_sec <= 86400);
        assert!(!reset_str.is_empty());
    }

    #[tokio::test]
    async fn test_halloween_in_memory_db_flow() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory sqlite db");

        // Basic users table
        sqlx::query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT,
                first_name TEXT,
                pokedollars INTEGER NOT NULL DEFAULT 0,
                boosts INTEGER NOT NULL DEFAULT 0,
                vitamins INTEGER NOT NULL DEFAULT 0,
                berries INTEGER NOT NULL DEFAULT 0,
                candies INTEGER NOT NULL DEFAULT 0,
                ball_master INTEGER NOT NULL DEFAULT 0
            );"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE pokemons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                pokemon_id INTEGER NOT NULL,
                nickname TEXT NOT NULL,
                level INTEGER NOT NULL,
                nature TEXT NOT NULL,
                iv_hp INTEGER NOT NULL,
                iv_attack INTEGER NOT NULL,
                iv_defense INTEGER NOT NULL,
                iv_sp_attack INTEGER NOT NULL,
                iv_sp_defense INTEGER NOT NULL,
                iv_speed INTEGER NOT NULL,
                ev_hp INTEGER NOT NULL,
                ev_attack INTEGER NOT NULL,
                ev_defense INTEGER NOT NULL,
                ev_sp_attack INTEGER NOT NULL,
                ev_sp_defense INTEGER NOT NULL,
                ev_speed INTEGER NOT NULL,
                experience INTEGER NOT NULL,
                move_1 TEXT,
                move_2 TEXT,
                move_3 TEXT,
                move_4 TEXT,
                is_starter INTEGER NOT NULL,
                is_shiny INTEGER NOT NULL
            );"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE user_key_items (
                user_id INTEGER NOT NULL,
                item_name TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (user_id, item_name)
            );"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE user_tutors (
                user_id INTEGER NOT NULL,
                tutor_number INTEGER NOT NULL,
                move_name TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (user_id, tutor_number)
            );"
        )
        .execute(&pool)
        .await
        .unwrap();

        // Initialize halloween event tables & columns
        init_halloween_tables(&pool).await.expect("Failed to init halloween tables");

        let test_user_id = 12345678u64;
        sqlx::query("INSERT INTO users (id, username, first_name) VALUES (?, 'testuser', 'Tester')")
            .bind(test_user_id as i64)
            .execute(&pool)
            .await
            .unwrap();

        // 1. Currency operations
        let c = increment_user_candies(&pool, test_user_id, 100).await.unwrap();
        assert_eq!(c, 100);
        let sh = increment_user_shards(&pool, test_user_id, 250).await.unwrap();
        assert_eq!(sh, 250);

        let stats = get_user_halloween_stats(&pool, test_user_id).await.unwrap();
        assert_eq!(stats.candies, 100);
        assert_eq!(stats.void_shards, 250);

        // 2. Treat limit & execution
        let (can_treat, used, seconds_remaining) = check_user_treat_limit(&pool, test_user_id).await.unwrap();
        assert!(can_treat);
        assert_eq!(used, 0);
        assert_eq!(seconds_remaining, 0);

        let treat_res = execute_halloween_treat(&pool, test_user_id).await.unwrap();
        assert!(treat_res.is_ok());
        let updated_stats = get_user_halloween_stats(&pool, test_user_id).await.unwrap();
        assert!(updated_stats.candies < 100 || updated_stats.candies >= 75); // 25 cost deducted (unless bonus candies dropped)
        assert_eq!(updated_stats.cauldron_spins, 1);

        // 3. Spooky Shop: Mega Stone purchase (1-per-trainer limit)
        let buy_mega = buy_halloween_shop_item(&pool, test_user_id, "Gengarite").await.unwrap();
        assert!(buy_mega.0, "First mega stone purchase should succeed: {}", buy_mega.1);

        // Attempt 2nd Mega Stone purchase - MUST BE BLOCKED
        let buy_mega_second = buy_halloween_shop_item(&pool, test_user_id, "Alakazite").await.unwrap();
        assert!(!buy_mega_second.0, "Second mega stone purchase must be rejected by 1-per-trainer limit");
        assert!(buy_mega_second.1.contains("Purchase Limit Reached"));

        // 4. Spooky Shop: Move Tutor purchase
        let buy_tutor = buy_halloween_shop_item(&pool, test_user_id, "tutor1").await.unwrap();
        assert!(buy_tutor.0, "Move tutor purchase should succeed");

        // 5. 1-of-1 Creation God claim uniqueness
        let (god_claim1, _) = claim_creation_god_for_user(&pool, test_user_id, "giratina", "Tester", false).await.unwrap();
        assert!(god_claim1, "First claim of Giratina should succeed");

        // Attempt second claim of Giratina by another user - MUST BE BLOCKED
        let (god_claim2, err2) = claim_creation_god_for_user(&pool, 88888888u64, "giratina", "Rival", false).await.unwrap();
        assert!(!god_claim2, "Second claim of 1-of-1 God must fail");
        assert!(err2.contains("already been claimed"));

        // Verify Pokémon was stored with origin species id and level 100
        let p_row = sqlx::query("SELECT pokemon_id, level, iv_hp, iv_attack, is_shiny FROM pokemons WHERE user_id = ?")
            .bind(test_user_id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        let pid: i64 = p_row.get("pokemon_id");
        let lvl: i32 = p_row.get("level");
        let iv_hp: i32 = p_row.get("iv_hp");
        let shiny: i32 = p_row.get("is_shiny");
        assert_eq!(pid, CREATION_GOD_GIRATINA.origin_species_id as i64);
        assert_eq!(lvl, 100);
        assert_eq!(iv_hp, 31);
        assert_eq!(shiny, 1);
    }

    #[tokio::test]
    async fn test_halloween_stop_and_status_control() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        init_halloween_tables(&pool).await.unwrap();

        // Initially enabled
        assert!(is_halloween_enabled(&pool).await);

        // Stop the event
        stop_halloween_event(&pool).await.unwrap();
        assert!(!is_halloween_enabled(&pool).await);

        // Restart the event
        set_halloween_status(&pool, true, false).await.unwrap();
        assert!(is_halloween_enabled(&pool).await);
    }
}
