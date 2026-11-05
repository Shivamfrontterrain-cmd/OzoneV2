//! Centralized Universal Emoji Registry & Formatter for HexaMon / Ozone
//! ========================================================================
//! Single source of truth for all custom Telegram emojis (<tg-emoji>) and standard
//! unicode emojis used across the Telegram bot, battle engine, inventory,
//! gyms, and menus. Converted from `emoji.py`.

// ==============================================================================
// 1. CORE HELPER FUNCTIONS
// ==============================================================================

/// Formats a custom Telegram emoji tag if html=true and emoji_id is present and non-empty,
/// otherwise returns the fallback unicode emoji.
pub fn format_custom_emoji(emoji_id: Option<&str>, fallback: &str, html: bool) -> String {
    if html {
        if let Some(id) = emoji_id {
            if !id.is_empty() {
                return format!(r#"<tg-emoji emoji-id="{}">{}</tg-emoji>"#, id, fallback);
            }
        }
    }
    fallback.to_string()
}

// ==============================================================================
// 2. NAVIGATION & GENERAL UI EMOJIS
// ==============================================================================

pub const BACKWARD_CUSTOM_EMOJI_ID: &str = "5283240108328244731";
pub const BACKWARD_FALLBACK: &str = "⬅️";
pub const BACKWARD_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5283240108328244731">⬅️</tg-emoji>"#;
pub const BACKWARD_EMOJI_PLAIN: &str = BACKWARD_FALLBACK;

// Mandatory custom emoji arrows aliases as per bot policy
pub const LEFT_ARROW_CUSTOM_EMOJI_ID: &str = BACKWARD_CUSTOM_EMOJI_ID;
pub const LEFT_ARROW_TG_EMOJI: &str = r#"<tg-emoji emoji-id="5283240108328244731">←</tg-emoji>"#;

pub const FORWARD_CUSTOM_EMOJI_ID: &str = "5282981997973621943";
pub const FORWARD_FALLBACK: &str = "➡️";
pub const FORWARD_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5282981997973621943">➡️</tg-emoji>"#;
pub const FORWARD_EMOJI_PLAIN: &str = FORWARD_FALLBACK;

pub const RIGHT_ARROW_CUSTOM_EMOJI_ID: &str = FORWARD_CUSTOM_EMOJI_ID;
pub const RIGHT_ARROW_TG_EMOJI: &str = r#"<tg-emoji emoji-id="5282981997973621943">→</tg-emoji>"#;

pub const INVENTORY_CUSTOM_EMOJI_ID: &str = "5893255507380014983";
pub const INVENTORY_FALLBACK: &str = "💼";
pub const INVENTORY_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5893255507380014983">💼</tg-emoji>"#;
pub const INVENTORY_EMOJI_PLAIN: &str = INVENTORY_FALLBACK;

pub const EVENTS_CUSTOM_EMOJI_ID: &str = "5791632037081256151";
pub const EVENTS_FALLBACK: &str = "⚪️";
pub const EVENTS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5791632037081256151">⚪️</tg-emoji>"#;
pub const EVENTS_EMOJI_PLAIN: &str = EVENTS_FALLBACK;

pub const ADS_CUSTOM_EMOJI_ID: &str = "5379872891053558877";
pub const ADS_FALLBACK: &str = "🤑";
pub const ADS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5379872891053558877">🤑</tg-emoji>"#;
pub const ADS_EMOJI_PLAIN: &str = ADS_FALLBACK;

pub const GYM_CONQUERED_EMOJI_ID: &str = "5372957680174384345";
pub const GYM_CONQUERED_FALLBACK: &str = "🤝";
pub const GYM_CONQUERED_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5372957680174384345">🤝</tg-emoji>"#;
pub const GYM_CONQUERED_EMOJI_PLAIN: &str = GYM_CONQUERED_FALLBACK;

pub const LOCKED_GYM_EMOJI_ID: &str = "5429405838345265327";
pub const LOCKED_GYM_FALLBACK: &str = "🔓";
pub const LOCKED_GYM_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5429405838345265327">🔓</tg-emoji>"#;
pub const LOCKED_GYM_EMOJI_PLAIN: &str = LOCKED_GYM_FALLBACK;

pub const SHOP_CUSTOM_EMOJI_ID: &str = "5278702045883292456";
pub const SHOP_FALLBACK: &str = "🏪";
pub const SHOP_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5278702045883292456">🏪</tg-emoji>"#;
pub const SHOP_EMOJI_PLAIN: &str = SHOP_FALLBACK;

pub fn get_backward_emoji(html: bool) -> &'static str {
    if html { BACKWARD_EMOJI_HTML } else { BACKWARD_EMOJI_PLAIN }
}

pub fn get_forward_emoji(html: bool) -> &'static str {
    if html { FORWARD_EMOJI_HTML } else { FORWARD_EMOJI_PLAIN }
}

pub fn get_inventory_emoji(html: bool) -> &'static str {
    if html { INVENTORY_EMOJI_HTML } else { INVENTORY_EMOJI_PLAIN }
}

pub fn get_events_emoji(html: bool) -> &'static str {
    if html { EVENTS_EMOJI_HTML } else { EVENTS_EMOJI_PLAIN }
}

pub fn get_ad_emoji(html: bool) -> &'static str {
    if html { ADS_EMOJI_HTML } else { ADS_EMOJI_PLAIN }
}

pub fn get_ads_emoji(html: bool) -> &'static str {
    get_ad_emoji(html)
}

pub fn get_conquered_emoji(html: bool) -> &'static str {
    if html { GYM_CONQUERED_EMOJI_HTML } else { GYM_CONQUERED_EMOJI_PLAIN }
}

pub fn get_locked_gym_emoji(html: bool) -> &'static str {
    if html { LOCKED_GYM_EMOJI_HTML } else { LOCKED_GYM_EMOJI_PLAIN }
}

pub fn get_shop_emoji(html: bool) -> &'static str {
    if html { SHOP_EMOJI_HTML } else { SHOP_EMOJI_PLAIN }
}

// ==============================================================================
// 3. ECONOMY & CURRENCY EMOJIS
// ==============================================================================

pub const CURRENCY_CUSTOM_EMOJI_ID: &str = "";
pub const CURRENCY_FALLBACK: &str = "💵";
pub const CURRENCY_EMOJI_HTML: &str = "💵";
pub const CURRENCY_EMOJI_PLAIN: &str = "💵";

pub const EXP_CUSTOM_EMOJI_ID: &str = "5463122435425448565";
pub const EXP_FALLBACK: &str = "⬆️";
pub const EXP_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5463122435425448565">⬆️</tg-emoji>"#;
pub const EXP_EMOJI_PLAIN: &str = EXP_FALLBACK;

pub const UP_ARROW_CUSTOM_EMOJI_ID: &str = "5390945398347032435";
pub const UP_ARROW_FALLBACK: &str = "⬆️";
pub const UP_ARROW_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5390945398347032435">⬆️</tg-emoji>"#;
pub const UP_ARROW_EMOJI_PLAIN: &str = UP_ARROW_FALLBACK;

pub const DOWN_ARROW_CUSTOM_EMOJI_ID: &str = "5389093764931282521";
pub const DOWN_ARROW_FALLBACK: &str = "⬇️";
pub const DOWN_ARROW_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5389093764931282521">⬇️</tg-emoji>"#;
pub const DOWN_ARROW_EMOJI_PLAIN: &str = DOWN_ARROW_FALLBACK;

pub fn get_currency_emoji(_html: bool) -> &'static str {
    "💵"
}

pub fn get_pokedollars_emoji(html: bool) -> &'static str {
    get_currency_emoji(html)
}

pub fn get_exp_emoji(html: bool) -> &'static str {
    if html { EXP_EMOJI_HTML } else { EXP_EMOJI_PLAIN }
}

pub fn get_up_arrow_emoji(html: bool) -> &'static str {
    if html { UP_ARROW_EMOJI_HTML } else { UP_ARROW_EMOJI_PLAIN }
}

pub fn get_down_arrow_emoji(html: bool) -> &'static str {
    if html { DOWN_ARROW_EMOJI_HTML } else { DOWN_ARROW_EMOJI_PLAIN }
}

// ==============================================================================
// 4. KEY ITEMS & SPECIAL EQUIPMENT
// ==============================================================================

pub const OMNI_RING_CUSTOM_EMOJI_ID: &str = "4904530220247155856";
pub const OMNI_RING_FALLBACK: &str = "🎰";
pub const OMNI_RING_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="4904530220247155856">🎰</tg-emoji>"#;
pub const OMNI_RING_EMOJI_PLAIN: &str = OMNI_RING_FALLBACK;

pub const KEY_ITEMS_CUSTOM_EMOJI_ID: &str = "5314784791986453963";
pub const KEY_ITEMS_FALLBACK: &str = "🎫";
pub const KEY_ITEMS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5314784791986453963">🎫</tg-emoji>"#;
pub const KEY_ITEMS_EMOJI_PLAIN: &str = KEY_ITEMS_FALLBACK;

pub const SAFARI_PASS_CUSTOM_EMOJI_ID: &str = "5314784791986453963";
pub const SAFARI_PASS_FALLBACK: &str = "🎫";
pub const SAFARI_PASS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5314784791986453963">🎫</tg-emoji>"#;
pub const SAFARI_PASS_EMOJI_PLAIN: &str = SAFARI_PASS_FALLBACK;

pub const INCUBATOR_CUSTOM_EMOJI_ID: &str = "6199289128564039060";
pub const INCUBATOR_FALLBACK: &str = "🌟";
pub const INCUBATOR_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="6199289128564039060">🌟</tg-emoji>"#;
pub const INCUBATOR_EMOJI_PLAIN: &str = INCUBATOR_FALLBACK;

pub const EGG_CUSTOM_EMOJI_ID: &str = "5199761466105818538";
pub const EGG_FALLBACK: &str = "🥚";
pub const EGG_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5199761466105818538">🥚</tg-emoji>"#;
pub const EGG_EMOJI_PLAIN: &str = "🥚";

pub const HATCHED_EGG_CUSTOM_EMOJI_ID: &str = "5199761466105818538";
pub const HATCHED_EGG_FALLBACK: &str = "🐣";
pub const HATCHED_EGG_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5199761466105818538">🐣</tg-emoji>"#;
pub const HATCHED_EGG_EMOJI_PLAIN: &str = "🐣";

pub const TMS_CUSTOM_EMOJI_ID: &str = "6192525164958196815";
pub const TMS_FALLBACK: &str = "💿";
pub const TMS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="6192525164958196815">💿</tg-emoji>"#;
pub const TMS_EMOJI_PLAIN: &str = TMS_FALLBACK;

pub const TUTORS_CUSTOM_EMOJI_ID: &str = "5393113918744847961";
pub const TUTORS_FALLBACK: &str = "📜";
pub const TUTORS_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5393113918744847961">📜</tg-emoji>"#;
pub const TUTORS_EMOJI_PLAIN: &str = TUTORS_FALLBACK;

pub const POKEBALLS_HEADER_CUSTOM_EMOJI_ID: &str = "5798459411584257481";
pub const POKEBALLS_HEADER_FALLBACK: &str = "⚪️";
pub const POKEBALLS_HEADER_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5798459411584257481">⚪️</tg-emoji>"#;
pub const POKEBALLS_HEADER_EMOJI_PLAIN: &str = POKEBALLS_HEADER_FALLBACK;
pub const POKEBALLS_CUSTOM_EMOJI_ID: &str = POKEBALLS_HEADER_CUSTOM_EMOJI_ID;
pub const POKEBALLS_EMOJI_HTML: &str = POKEBALLS_HEADER_EMOJI_HTML;
pub const POKEBALLS_EMOJI_PLAIN: &str = POKEBALLS_HEADER_EMOJI_PLAIN;

pub fn get_omni_ring_emoji(html: bool) -> &'static str {
    if html { OMNI_RING_EMOJI_HTML } else { OMNI_RING_EMOJI_PLAIN }
}

pub fn get_key_items_emoji(html: bool) -> &'static str {
    if html { KEY_ITEMS_EMOJI_HTML } else { KEY_ITEMS_EMOJI_PLAIN }
}

pub fn get_safari_pass_emoji(html: bool) -> &'static str {
    if html { SAFARI_PASS_EMOJI_HTML } else { SAFARI_PASS_EMOJI_PLAIN }
}

pub fn get_incubator_emoji(html: bool) -> &'static str {
    if html { INCUBATOR_EMOJI_HTML } else { INCUBATOR_EMOJI_PLAIN }
}

pub fn get_egg_emoji(html: bool) -> &'static str {
    if html { EGG_EMOJI_HTML } else { EGG_EMOJI_PLAIN }
}

pub fn get_hatched_egg_emoji(html: bool) -> &'static str {
    if html { HATCHED_EGG_EMOJI_HTML } else { HATCHED_EGG_EMOJI_PLAIN }
}

pub fn get_tms_emoji(html: bool) -> &'static str {
    if html { TMS_EMOJI_HTML } else { TMS_EMOJI_PLAIN }
}

pub fn get_tutors_emoji(html: bool) -> &'static str {
    if html { TUTORS_EMOJI_HTML } else { TUTORS_EMOJI_PLAIN }
}

pub fn get_pokeballs_header_emoji(html: bool) -> &'static str {
    if html { POKEBALLS_HEADER_EMOJI_HTML } else { POKEBALLS_HEADER_EMOJI_PLAIN }
}

// ==============================================================================
// 5. POKÉBALLS
// ==============================================================================

pub const POKEBALL_CUSTOM_EMOJI_IDS: &[(&str, &str)] = &[
    ("pokeball", "5425017773338164606"),
    ("greatball", "5427369899292913690"),
    ("ultraball", "5424651623081212062"),
    ("masterball", "5425124632124488025"),
    ("safariball", "5425134072462603163"),
    ("nestball", "5424950320876779633"),
    ("repeatball", "5425091681135393389"),
    ("duskball", "5427110101016147967"),
    ("netball", "5424670903189403539"),
    ("diveball", "5427391301114951718"),
    ("timerball", "5424989495273488721"),
    ("quickball", "5424759023033415966"),
    ("healball", "5425129549862041524"),
    ("premierball", "5424626579126910023"),
    ("luxuryball", "5424985891795923166"),
    ("fastball", "5425046562503947502"),
    ("levelball", "5427172425286578795"),
    ("lureball", "5426879761920063423"),
    ("heavyball", "5424773557202747996"),
    ("loveball", "5424739339198298944"),
    ("friendball", "5425130241351776014"),
    ("moonball", "5424696368050503555"),
    ("sportball", "5424991127361058505"),
    ("parkball", "5424882619307291644"),
    ("dreamball", "5424825577846635536"),
    ("beastball", "5425006142566726351"),
    ("cherishball", "5424991878980335741"),
    ("monsoonball", "5427391301114951718"),
    ("chibiball", "5424991878980335741"),
];

pub fn get_pokeball_custom_emoji_id(ball_key: &str) -> Option<&'static str> {
    let clean = ball_key.to_lowercase().replace([' ', '_', '-'], "");
    let k = match clean.as_str() {
        "regularball" | "poke" | "ball" | "regular" => "pokeball",
        "safaripass" | "safari" => "safariball",
        "great" | "gt" => "greatball",
        "ultra" | "ub" => "ultraball",
        "master" | "mb" => "masterball",
        "monsoon" => "monsoonball",
        other => other,
    };
    for &(name, id) in POKEBALL_CUSTOM_EMOJI_IDS {
        if name == k {
            return Some(id);
        }
    }
    None
}

pub fn get_pokeball_emoji(ball_key: &str, html: bool) -> &'static str {
    let clean = ball_key.to_lowercase().replace([' ', '_', '-'], "");
    let k = match clean.as_str() {
        "regularball" | "poke" | "ball" | "regular" => "pokeball",
        "safaripass" | "safari" => "safariball",
        "great" | "gt" => "greatball",
        "ultra" | "ub" => "ultraball",
        "master" | "mb" => "masterball",
        "monsoon" => "monsoonball",
        other => other,
    };

    if !html {
        return "🎱";
    }

    match k {
        "pokeball" => r#"<tg-emoji emoji-id="5425017773338164606">🎱</tg-emoji>"#,
        "greatball" => r#"<tg-emoji emoji-id="5427369899292913690">🎱</tg-emoji>"#,
        "ultraball" => r#"<tg-emoji emoji-id="5424651623081212062">🎱</tg-emoji>"#,
        "masterball" => r#"<tg-emoji emoji-id="5425124632124488025">🎱</tg-emoji>"#,
        "safariball" => r#"<tg-emoji emoji-id="5425134072462603163">🎱</tg-emoji>"#,
        "nestball" => r#"<tg-emoji emoji-id="5424950320876779633">🎱</tg-emoji>"#,
        "repeatball" => r#"<tg-emoji emoji-id="5425091681135393389">🎱</tg-emoji>"#,
        "duskball" => r#"<tg-emoji emoji-id="5427110101016147967">🎱</tg-emoji>"#,
        "netball" => r#"<tg-emoji emoji-id="5424670903189403539">🎱</tg-emoji>"#,
        "diveball" => r#"<tg-emoji emoji-id="5427391301114951718">🎱</tg-emoji>"#,
        "timerball" => r#"<tg-emoji emoji-id="5424989495273488721">🎱</tg-emoji>"#,
        "quickball" => r#"<tg-emoji emoji-id="5424759023033415966">🎱</tg-emoji>"#,
        "healball" => r#"<tg-emoji emoji-id="5425129549862041524">🎱</tg-emoji>"#,
        "premierball" => r#"<tg-emoji emoji-id="5424626579126910023">🎱</tg-emoji>"#,
        "luxuryball" => r#"<tg-emoji emoji-id="5424985891795923166">🎱</tg-emoji>"#,
        "fastball" => r#"<tg-emoji emoji-id="5425046562503947502">🎱</tg-emoji>"#,
        "levelball" => r#"<tg-emoji emoji-id="5427172425286578795">🎱</tg-emoji>"#,
        "lureball" => r#"<tg-emoji emoji-id="5426879761920063423">🎱</tg-emoji>"#,
        "heavyball" => r#"<tg-emoji emoji-id="5424773557202747996">🎱</tg-emoji>"#,
        "loveball" => r#"<tg-emoji emoji-id="5424739339198298944">🎱</tg-emoji>"#,
        "friendball" => r#"<tg-emoji emoji-id="5425130241351776014">🎱</tg-emoji>"#,
        "moonball" => r#"<tg-emoji emoji-id="5424696368050503555">🎱</tg-emoji>"#,
        "sportball" => r#"<tg-emoji emoji-id="5424991127361058505">🎱</tg-emoji>"#,
        "parkball" => r#"<tg-emoji emoji-id="5424882619307291644">🎱</tg-emoji>"#,
        "dreamball" => r#"<tg-emoji emoji-id="5424825577846635536">🎱</tg-emoji>"#,
        "beastball" => r#"<tg-emoji emoji-id="5425006142566726351">🎱</tg-emoji>"#,
        "cherishball" => r#"<tg-emoji emoji-id="5424991878980335741">🎱</tg-emoji>"#,
        "monsoonball" => r#"<tg-emoji emoji-id="5427391301114951718">🎱</tg-emoji>"#,
        "chibiball" => r#"<tg-emoji emoji-id="5424991878980335741">🎱</tg-emoji>"#,
        _ => "🎱",
    }
}

// ==============================================================================
// 6. POKÉMON MOVE & ELEMENTAL TYPES
// ==============================================================================

pub const MOVE_TYPE_PLAIN_EMOJIS: &[(&str, &str)] = &[
    ("Normal", "🎁"),
    ("Fire", "🔥"),
    ("Water", "💧"),
    ("Electric", "⚡"),
    ("Grass", "🌿"),
    ("Ice", "❄️"),
    ("Fighting", "🥊"),
    ("Poison", "🧪"),
    ("Ground", "🪨"),
    ("Flying", "🪽"),
    ("Psychic", "🔮"),
    ("Bug", "🐛"),
    ("Rock", "🪨"),
    ("Ghost", "👻"),
    ("Dragon", "🐉"),
    ("Dark", "🌑"),
    ("Steel", "🔩"),
    ("Fairy", "🧚"),
];

pub const MOVE_TYPE_CUSTOM_EMOJI_IDS: &[(&str, &str)] = &[
    ("Normal", "6120921634862800671"),
    ("Fire", "6122676369521385632"),
    ("Water", "6122856740967950386"),
    ("Electric", "6120423087943984078"),
    ("Grass", "6120509726024277548"),
    ("Ice", "6120931706561111199"),
    ("Fighting", "6122836039225581659"),
    ("Poison", "6120510391744208596"),
    ("Ground", "6122668629990319764"),
    ("Flying", "6122860864136553639"),
    ("Psychic", "6120626703753552440"),
    ("Bug", "6122889240985478227"),
    ("Rock", "6123197808615891197"),
    ("Ghost", "6120875811856721604"),
    ("Dragon", "6120513333796807923"),
    ("Dark", "6120930220502426354"),
    ("Steel", "6120534877352763496"),
    ("Fairy", "6120623229125009484"),
];

pub fn get_type_emoji_plain(type_name: &str) -> &'static str {
    get_move_type_emoji(type_name, false)
}

pub fn get_type_emoji(type_name: &str, html: bool) -> &'static str {
    get_move_type_emoji(type_name, html)
}

pub fn get_move_type_emoji(type_name: &str, html: bool) -> &'static str {
    let t_clean = type_name.trim();
    if !html {
        return match t_clean {
            s if s.eq_ignore_ascii_case("Normal") => "🎁",
            s if s.eq_ignore_ascii_case("Fire") => "🔥",
            s if s.eq_ignore_ascii_case("Water") => "💧",
            s if s.eq_ignore_ascii_case("Electric") => "⚡",
            s if s.eq_ignore_ascii_case("Grass") => "🌿",
            s if s.eq_ignore_ascii_case("Ice") => "❄️",
            s if s.eq_ignore_ascii_case("Fighting") => "🥊",
            s if s.eq_ignore_ascii_case("Poison") => "🧪",
            s if s.eq_ignore_ascii_case("Ground") => "🪨",
            s if s.eq_ignore_ascii_case("Flying") => "🪽",
            s if s.eq_ignore_ascii_case("Psychic") => "🔮",
            s if s.eq_ignore_ascii_case("Bug") => "🐛",
            s if s.eq_ignore_ascii_case("Rock") => "🪨",
            s if s.eq_ignore_ascii_case("Ghost") => "👻",
            s if s.eq_ignore_ascii_case("Dragon") => "🐉",
            s if s.eq_ignore_ascii_case("Dark") => "🌑",
            s if s.eq_ignore_ascii_case("Steel") => "🔩",
            s if s.eq_ignore_ascii_case("Fairy") => "🧚",
            _ => "🔘",
        };
    }

    match t_clean {
        s if s.eq_ignore_ascii_case("Normal") => r#"<tg-emoji emoji-id="6120921634862800671">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Fire") => r#"<tg-emoji emoji-id="6122676369521385632">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Water") => r#"<tg-emoji emoji-id="6122856740967950386">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Electric") => r#"<tg-emoji emoji-id="6120423087943984078">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Grass") => r#"<tg-emoji emoji-id="6120509726024277548">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Ice") => r#"<tg-emoji emoji-id="6120931706561111199">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Fighting") => r#"<tg-emoji emoji-id="6122836039225581659">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Poison") => r#"<tg-emoji emoji-id="6120510391744208596">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Ground") => r#"<tg-emoji emoji-id="6122668629990319764">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Flying") => r#"<tg-emoji emoji-id="6122860864136553639">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Psychic") => r#"<tg-emoji emoji-id="6120626703753552440">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Bug") => r#"<tg-emoji emoji-id="6122889240985478227">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Rock") => r#"<tg-emoji emoji-id="6123197808615891197">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Ghost") => r#"<tg-emoji emoji-id="6120875811856721604">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Dragon") => r#"<tg-emoji emoji-id="6120513333796807923">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Dark") => r#"<tg-emoji emoji-id="6120930220502426354">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Steel") => r#"<tg-emoji emoji-id="6120534877352763496">🎃</tg-emoji>"#,
        s if s.eq_ignore_ascii_case("Fairy") => r#"<tg-emoji emoji-id="6120623229125009484">🎃</tg-emoji>"#,
        _ => "🔘",
    }
}

// ==============================================================================
// 7. CATCH, HUNT & WILD ENCOUNTERS
// ==============================================================================

pub const CATCH_STAR_CUSTOM_EMOJI_ID: &str = "5220064167356025824";
pub const CATCH_STAR_FALLBACK: &str = "⭐️";
pub const CATCH_STAR_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5220064167356025824">⭐️</tg-emoji>"#;
pub const CATCH_STAR_EMOJI_PLAIN: &str = CATCH_STAR_FALLBACK;

pub const CAUGHT_POKEMON_CUSTOM_EMOJI_ID: &str = "5258185631355378853";
pub const CAUGHT_POKEMON_FALLBACK: &str = "⭐️";
pub const CAUGHT_POKEMON_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5258185631355378853">⭐️</tg-emoji>"#;
pub const CAUGHT_POKEMON_EMOJI_PLAIN: &str = CAUGHT_POKEMON_FALLBACK;

pub const SHINY_CUSTOM_EMOJI_ID: &str = "5251676977785483690";
pub const SHINY_FALLBACK: &str = "✨";
pub const SHINY_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5251676977785483690">✨</tg-emoji>"#;
pub const SHINY_EMOJI_PLAIN: &str = SHINY_FALLBACK;

pub const CHIBI_CUSTOM_EMOJI_ID: &str = "6159003421954287505";
pub const CHIBI_FALLBACK: &str = "🧸";
pub const CHIBI_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="6159003421954287505">🧸</tg-emoji>"#;
pub const CHIBI_EMOJI_PLAIN: &str = CHIBI_FALLBACK;

pub const CATCH_CUSTOM_EMOJI_ID: &str = "5425134072462603163";
pub const CATCH_FALLBACK: &str = "🎯";
pub const CATCH_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5425134072462603163">🎯</tg-emoji>"#;
pub const CATCH_EMOJI_PLAIN: &str = CATCH_FALLBACK;

pub const FLEE_CUSTOM_EMOJI_ID: &str = "5462882007451185227";
pub const FLEE_FALLBACK: &str = "💨";
pub const FLEE_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5462882007451185227">💨</tg-emoji>"#;
pub const FLEE_EMOJI_PLAIN: &str = FLEE_FALLBACK;

pub const NEUTRAL_ARROW_FALLBACK: &str = "➖";
pub const NEUTRAL_ARROW_EMOJI_HTML: &str = NEUTRAL_ARROW_FALLBACK;
pub const NEUTRAL_ARROW_EMOJI_PLAIN: &str = NEUTRAL_ARROW_FALLBACK;

pub fn get_catch_star_emoji(html: bool) -> &'static str {
    if html { CATCH_STAR_EMOJI_HTML } else { CATCH_STAR_EMOJI_PLAIN }
}

pub fn format_catch_animation_stars(stage: i32, html: bool) -> String {
    let star = get_catch_star_emoji(html);
    match stage {
        1 => format!("{}☆☆", star),
        2 => format!("{}{}☆", star, star),
        3 => format!("{}{}{}", star, star, star),
        _ => "☆☆☆".to_string(),
    }
}

pub fn get_caught_star_emoji(html: bool) -> &'static str {
    if html { CAUGHT_POKEMON_EMOJI_HTML } else { CAUGHT_POKEMON_EMOJI_PLAIN }
}

pub fn get_shiny_emoji(html: bool) -> &'static str {
    if html { SHINY_EMOJI_HTML } else { SHINY_EMOJI_PLAIN }
}

pub fn get_chibi_emoji(html: bool) -> &'static str {
    if html { CHIBI_EMOJI_HTML } else { CHIBI_EMOJI_PLAIN }
}

pub fn get_catch_emoji(html: bool) -> &'static str {
    if html { CATCH_EMOJI_HTML } else { CATCH_EMOJI_PLAIN }
}

pub fn get_flee_emoji(html: bool) -> &'static str {
    if html { FLEE_EMOJI_HTML } else { FLEE_EMOJI_PLAIN }
}

pub fn get_neutral_arrow_emoji(_html: bool) -> &'static str {
    NEUTRAL_ARROW_FALLBACK
}

pub fn format_safari_status(catch_mod: f64, flee_mod: f64, html: bool) -> String {
    let up_emoji = get_up_arrow_emoji(html);
    let down_emoji = get_down_arrow_emoji(html);
    let neutral_emoji = get_neutral_arrow_emoji(html);

    let c_arrow = if catch_mod > 1.05 {
        up_emoji
    } else if catch_mod < 0.95 {
        down_emoji
    } else {
        neutral_emoji
    };

    let f_arrow = if flee_mod > 1.05 {
        up_emoji
    } else if flee_mod < 0.95 {
        down_emoji
    } else {
        neutral_emoji
    };

    format!("[ Catch {} | Flee {} ]", c_arrow, f_arrow)
}

// ==============================================================================
// 8. BATTLE COMBAT & STATUS CONDITIONS
// ==============================================================================

pub const BATTLE_RUN_CUSTOM_EMOJI_ID: &str = "";
pub const BATTLE_RUN_FALLBACK: &str = "";
pub const BATTLE_RUN_EMOJI_HTML: &str = "";
pub const BATTLE_RUN_EMOJI_PLAIN: &str = "";

pub fn get_battle_run_emoji(_html: bool) -> &'static str {
    ""
}

pub fn get_status_emoji(_status_key: &str, _html: bool) -> &'static str {
    ""
}

pub fn get_combat_emoji(event_key: &str, html: bool) -> &'static str {
    let clean = event_key.to_lowercase();
    let norm = match clean.trim() {
        "faint" | "fainted" => "fainted",
        "crit" | "critical" | "critical_hit" => "critical_hit",
        "super" | "super_effective" => "super_effective",
        "not_very" | "not_very_effective" => "not_very_effective",
        "immune" | "none" | "no_effect" => "no_effect",
        "evolve" | "ready_to_evolve" => "ready_to_evolve",
        "trainer_level" | "trainer_level_up" => "trainer_level_up",
        "level_up" => "level_up",
        "miss" | "missed" => "missed",
        other => other,
    };

    match norm {
        "super_effective" => {
            if html {
                r#"<tg-emoji emoji-id="5463156928307801722">🤕</tg-emoji>"#
            } else {
                "🤕"
            }
        }
        "not_very_effective" => {
            if html {
                r#"<tg-emoji emoji-id="5465154440287757794">🛡</tg-emoji>"#
            } else {
                "🛡"
            }
        }
        "no_effect" => {
            if html {
                r#"<tg-emoji emoji-id="5462882007451185227">🚫</tg-emoji>"#
            } else {
                "🚫"
            }
        }
        "level_up" | "ready_to_evolve" | "trainer_level_up" => {
            if html {
                r#"<tg-emoji emoji-id="5463122435425448565">⬆️</tg-emoji>"#
            } else {
                "⬆️"
            }
        }
        _ => "",
    }
}

// ==============================================================================
// 9. ITEMS, CANDIES, VITAMINS, BERRIES, STONES & CRYSTALS
// ==============================================================================

pub const ITEM_CUSTOM_EMOJI_IDS: &[(&str, &str)] = &[
    // Candies
    ("rarecandy", "5427163461689833807"),
    ("nullcandy", "5427163461689833807"),
    ("ncandy", "5427163461689833807"),
    ("candy", "5427163461689833807"),

    // Vitamins & Stats
    ("vitamin", "5424826806207280716"),
    ("dose", "5440739140347907722"),
    ("hpup", "5424826806207280716"),
    ("protein", "5424796462263336445"),
    ("iron", "5424768776904145616"),
    ("calcium", "5427337742872772523"),
    ("zinc", "5427074585931576805"),
    ("carbos", "5427116689495981790"),
    ("ppup", "5426846862470574901"),
    ("ppmax", "5424964524333630022"),
    ("boost", "5424810012885153706"),

    // Berries
    ("berry", "5425069540578981119"),
    ("cheriberry", "5425069540578981119"),
    ("chestoberry", "5424963313152849581"),
    ("pechaberry", "5424591643862924640"),
    ("rawstberry", "5427203675468628149"),
    ("aspearberry", "5425125246304817533"),
    ("leppaberry", "5425044784387486665"),
    ("oranberry", "5424974939629318294"),
    ("persimberry", "5427066623062209865"),
    ("lumberry", "5424829391777594136"),
    ("sitrusberry", "5424967900177924144"),
    ("figyberry", "5427306303712164216"),
    ("wikiberry", "5424747259117993766"),
    ("magoberry", "5424748770946482169"),
    ("aguavberry", "5427134474955551541"),
    ("iapapaberry", "5425144066851502617"),
    ("razzberry", "5424826269336369785"),
    ("blukberry", "5427341543918830114"),
    ("nanabberry", "5424711241522246065"),
    ("wepearberry", "5425139527071070479"),
    ("pinapberry", "5425097960377579954"),
    ("pomegberry", "5424628473207487315"),
    ("kelpsyberry", "5426849194637817499"),
    ("qualotberry", "5427397258234591188"),
    ("hondewberry", "5424834296630244837"),
    ("grepaberry", "5424682873263259225"),
    ("tamatoberry", "5426949348980191564"),
    ("cornnberry", "5426877391098114282"),
    ("magostberry", "5424670147275161911"),
    ("rabutaberry", "5427160455212726091"),
    ("nomelberry", "5424733669841466378"),
    ("spelonberry", "5425009453986511337"),
    ("pamtreberry", "5424589187141631205"),
    ("watmelberry", "5424711911537148174"),
    ("durinberry", "5427241299382136762"),
    ("belueberry", "5427222616274402736"),
    ("occaberry", "5427359183349512444"),
    ("passhoberry", "5425045879604146166"),
    ("wacanberry", "5424839845727990215"),
    ("rindoberry", "5424972809325541125"),
    ("yacheberry", "5427398151587786311"),
    ("chopleberry", "5425125568427358372"),
    ("kebiaberry", "5424786931730905894"),
    ("shucaberry", "5427063736844187410"),
    ("cobaberry", "5424942577050745927"),
    ("payapaberry", "5424653113434863235"),
    ("tangaberry", "5425132040943073905"),
    ("chartiberry", "5424733253229639465"),
    ("kasibberry", "5427331708443717854"),
    ("habanberry", "5424619260502637422"),
    ("colburberry", "5424818379481446123"),
    ("babiriberry", "5424803282671400631"),
    ("chilanberry", "5424906688304022286"),
    ("roseliberry", "5424906688304022286"),
    ("liechiberry", "5424836740466637066"),
    ("ganlonberry", "5425114010670366575"),
    ("salacberry", "5427383484274471937"),
    ("petayaberry", "5426899381330671056"),
    ("apicotberry", "5425100326904559129"),
    ("lansatberry", "5425048082922371529"),
    ("starfberry", "5425058081606236674"),
    ("enigmaberry", "5424631556994004946"),
    ("micleberry", "5424693988638619603"),
    ("custapberry", "5427029699228365070"),
    ("jabocaberry", "5424837887222907483"),
    ("rowapberry", "5424837887222907483"),

    // Evolution Stones
    ("firestone", "5427264354766581696"),
    ("thunderstone", "5424613891793517371"),
    ("waterstone", "5425125508297814747"),
    ("icestone", "5425130060963152282"),
    ("leafstone", "5424903187905674190"),
    ("moonstone", "5424616107996642292"),
    ("sunstone", "5424623147448039749"),
    ("shinystone", "5424869249074100636"),
    ("duskstone", "5424869249074100636"),
    ("dawnstone", "5425130060963152282"),
    ("ovalstone", "5424616107996642292"),

    // Mega Stones & Crystals
    ("megastone", "5424869249074100636"),
    ("crystal", "5424896199993882128"),
    ("teracrystal", "5424896199993882128"),
    ("zcrystal", "5424896199993882128"),
    ("terashard", "5424839141353357502"),
    ("shard", "5424839141353357502"),

    // Key Items & Special
    ("omni", OMNI_RING_CUSTOM_EMOJI_ID),
    ("omniring", OMNI_RING_CUSTOM_EMOJI_ID),
    ("safaripass", "5314784791986453963"),
    ("clefairydoll", "5424753164698021505"),
    ("pokedoll", "5424753164698021505"),
    ("egg", "5199761466105818538"),
    ("eggincubator", "6199289128564039060"),
    ("incubator", "6199289128564039060"),
    ("tm", "6192525164958196815"),
    ("tms", "6192525164958196815"),
    ("technicalmachine", "6192525164958196815"),
    ("tutor", TUTORS_CUSTOM_EMOJI_ID),
    ("tutors", TUTORS_CUSTOM_EMOJI_ID),
    ("movetutor", TUTORS_CUSTOM_EMOJI_ID),
];

pub const ITEM_PRIMARY_FALLBACKS: &[(&str, &str)] = &[
    ("rarecandy", "🍬"), ("nullcandy", "🍬"), ("ncandy", "🍬"), ("candy", "🍬"),
    ("vitamin", "💊"), ("dose", "⚡️"), ("hpup", "💊"), ("protein", "💊"),
    ("iron", "💊"), ("calcium", "💊"), ("zinc", "💊"), ("carbos", "💊"),
    ("ppup", "💊"), ("ppmax", "💊"),
    ("boost", "💪"),
    ("berry", "🍒"), ("cheriberry", "🍒"), ("chestoberry", "🍒"), ("pechaberry", "🍒"),
    ("rawstberry", "🍒"), ("aspearberry", "🍒"), ("leppaberry", "🍒"), ("oranberry", "🍒"),
    ("persimberry", "🍒"), ("lumberry", "🍍"), ("sitrusberry", "🍒"), ("figyberry", "🍓"),
    ("wikiberry", "🍒"), ("magoberry", "🍒"), ("aguavberry", "🍒"), ("iapapaberry", "🍒"),
    ("razzberry", "🍒"), ("blukberry", "🍒"), ("nanabberry", "🍒"), ("wepearberry", "🍒"),
    ("pinapberry", "🍒"), ("pomegberry", "🍒"), ("kelpsyberry", "🍒"), ("qualotberry", "🍒"),
    ("hondewberry", "🍒"), ("grepaberry", "🍒"), ("tamatoberry", "🍒"), ("cornnberry", "🍒"),
    ("magostberry", "🍒"), ("rabutaberry", "🍒"), ("nomelberry", "🍒"), ("spelonberry", "🍒"),
    ("pamtreberry", "🍒"), ("watmelberry", "🍒"), ("durinberry", "🍒"), ("belueberry", "🍒"),
    ("occaberry", "🍒"), ("passhoberry", "🍒"), ("wacanberry", "🍒"), ("rindoberry", "🍒"),
    ("yacheberry", "🍒"), ("chopleberry", "🍒"), ("kebiaberry", "🍒"), ("shucaberry", "🍒"),
    ("cobaberry", "🍒"), ("payapaberry", "🍒"), ("tangaberry", "🍒"), ("chartiberry", "🍐"),
    ("kasibberry", "🍒"), ("habanberry", "🍐"), ("colburberry", "🍒"), ("babiriberry", "🍅"),
    ("chilanberry", "🍒"), ("roseliberry", "🍒"), ("liechiberry", "🍒"), ("ganlonberry", "🍒"),
    ("salacberry", "🍒"), ("petayaberry", "🍒"), ("apicotberry", "🍒"), ("lansatberry", "🍒"),
    ("starfberry", "🍒"), ("enigmaberry", "🍒"), ("micleberry", "🧅"), ("custapberry", "🍒"),
    ("jabocaberry", "🍒"), ("rowapberry", "🍒"),
    ("firestone", "🔥"), ("thunderstone", "⚡️"), ("waterstone", "🫧"), ("icestone", "❄️"),
    ("leafstone", "🌱"), ("moonstone", "🪨"), ("sunstone", "⭐️"), ("shinystone", "💎"),
    ("duskstone", "💎"), ("dawnstone", "❄️"), ("ovalstone", "🪨"),
    ("megastone", "💎"), ("crystal", "🧊"), ("teracrystal", "🧊"), ("zcrystal", "🧊"),
    ("terashard", "🧊"), ("shard", "🧊"), ("omni", OMNI_RING_FALLBACK), ("omniring", OMNI_RING_FALLBACK),
    ("safaripass", "🎫"), ("clefairydoll", "😊"), ("pokedoll", "😊"),
    ("egg", "🥚"), ("eggincubator", "🌟"), ("incubator", "🌟"),
    ("tm", "🌟"), ("tms", "🌟"), ("technicalmachine", "🌟"),
    ("tutor", TUTORS_FALLBACK), ("tutors", TUTORS_FALLBACK), ("movetutor", TUTORS_FALLBACK),
];

pub fn get_item_emoji(item_name: &str, html: bool) -> String {
    if item_name.trim().is_empty() {
        return format_custom_emoji(Some("5314784791986453963"), "🎫", html);
    }

    let clean_k = item_name
        .to_lowercase()
        .replace([' ', '_', '-', '(', ')'], "");

    // TMs
    if clean_k == "tm" || clean_k == "tms" || clean_k.starts_with("tm") {
        return get_tms_emoji(html).to_string();
    }

    // Move Tutors
    if clean_k == "tutor" || clean_k == "tutors" || clean_k.starts_with("tutor") || clean_k.contains("movetutor") {
        return get_tutors_emoji(html).to_string();
    }

    // Omni Ring
    if clean_k == "omni" || clean_k == "omniring" {
        return get_omni_ring_emoji(html).to_string();
    }

    // Direct match
    for &(k, eid) in ITEM_CUSTOM_EMOJI_IDS {
        if k == clean_k {
            let mut fb = "🎫";
            for &(fk, fval) in ITEM_PRIMARY_FALLBACKS {
                if fk == clean_k {
                    fb = fval;
                    break;
                }
            }
            return format_custom_emoji(Some(eid), fb, html);
        }
    }

    // Heuristic & Category Matching
    // Mega Stones (e.g. Lucarionite, Charizardite X)
    if clean_k.ends_with("ite")
        || clean_k.ends_with("itex")
        || clean_k.ends_with("itey")
        || clean_k.ends_with("itez")
        || clean_k.contains("megastone")
    {
        return format_custom_emoji(Some("5424869249074100636"), "💎", html);
    }

    // Z-Crystals & Tera Crystals / Orbs / Shards
    if clean_k.contains("iumz")
        || clean_k.contains("zcrystal")
        || clean_k.contains("crystal")
        || clean_k.contains("orb")
        || clean_k.contains("shard")
    {
        return format_custom_emoji(Some("5424896199993882128"), "🧊", html);
    }

    // Swords, Shields, Rusted Artifacts
    if clean_k.contains("sword")
        || clean_k.contains("shield")
        || clean_k.contains("rusted")
    {
        return format_custom_emoji(Some("5424896199993882128"), "⚔️", html);
    }

    // Carrots (Iceroot Carrot, Shaderoot Carrot)
    if clean_k.contains("carrot") {
        return format_custom_emoji(Some("5425069540578981119"), "🥕", html);
    }

    // Berries
    if clean_k.contains("berry") {
        return format_custom_emoji(Some("5425069540578981119"), "🍒", html);
    }

    // Candies
    if clean_k.contains("candy") {
        return format_custom_emoji(Some("5427163461689833807"), "🍬", html);
    }

    // Evolution Stones
    if clean_k.contains("stone") {
        let (eid, fallback) = if clean_k.contains("fire") {
            ("5427264354766581696", "🔥")
        } else if clean_k.contains("thunder") || clean_k.contains("electric") {
            ("5424613891793517371", "⚡️")
        } else if clean_k.contains("water") {
            ("5425125508297814747", "🫧")
        } else if clean_k.contains("ice") || clean_k.contains("dawn") {
            ("5425130060963152282", "❄️")
        } else if clean_k.contains("leaf") || clean_k.contains("grass") {
            ("5424903187905674190", "🌱")
        } else if clean_k.contains("sun") {
            ("5424623147448039749", "⭐️")
        } else if clean_k.contains("shiny") || clean_k.contains("dusk") {
            ("5424869249074100636", "💎")
        } else {
            ("5424616107996642292", "🪨")
        };
        return format_custom_emoji(Some(eid), fallback, html);
    }

    // Vitamins / Stat Boosters
    if clean_k.contains("vitamin")
        || clean_k.contains("protein")
        || clean_k.contains("iron")
        || clean_k.contains("calcium")
        || clean_k.contains("zinc")
        || clean_k.contains("carbos")
        || clean_k.contains("hpup")
        || clean_k.contains("ppup")
        || clean_k.contains("ppmax")
    {
        return format_custom_emoji(Some("5424826806207280716"), "💊", html);
    }

    // Default Key Item
    format_custom_emoji(Some("5314784791986453963"), "🎫", html)
}

// ==============================================================================
// 10. GYM BADGES & DOSE
// ==============================================================================

pub const KANTO_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Boulder Badge", "6212813379283722048"),
    ("Cascade Badge", "6213204006559293292"),
    ("Thunder Badge", "6213102327503527219"),
    ("Rainbow Badge", "6213176918200556197"),
    ("Soul Badge", "6212811657001836714"),
    ("Marsh Badge", "6212936773694135722"),
    ("Volcano Badge", "6213249172435376699"),
    ("Earth Badge", "6212800429957323813"),
];

pub const JOHTO_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Zephyr Badge", "6224448900955118411"),
    ("Hive Badge", "6224361597154892802"),
    ("Plain Badge", "6222191650827935720"),
    ("Fog Badge", "6222162286136531995"),
    ("Storm Badge", "6222261590075383439"),
    ("Mineral Badge", "6221886678085148490"),
    ("Glacier Badge", "6224500363253260486"),
    ("Rising Badge", "6224142244585152552"),
];

pub const HOENN_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Stone Badge", "6233372356937522205"),
    ("Knuckle Badge", "6233543781967209292"),
    ("Dynamo Badge", "6233536368853654054"),
    ("Heat Badge", "6233181943857420831"),
    ("Balance Badge", "6233522951375821687"),
    ("Feather Badge", "6233130859516403656"),
    ("Mind Badge", "6233226143365863801"),
    ("Rain Badge", "6233036434660399461"),
];

pub const SINNOH_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Coal Badge", "6298290882105843328"),
    ("Forest Badge", "6298727667394944238"),
    ("Cobble Badge", "6298369677575856417"),
    ("Fen Badge", "6298490872963015475"),
    ("Relic Badge", "6298760373570904054"),
    ("Mine Badge", "6298669917264682103"),
    ("Icicle Badge", "6298672558669569167"),
    ("Beacon Badge", "6298598019512146938"),
];

pub const ORANGE_ISLAND_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Coral-Eye Badge", "6257784409882108221"),
    ("Sea Ruby Badge", "6260452954667491060"),
    ("Spike Shell Badge", "6260169306437328641"),
    ("Jade Star Badge", "6257859211032534512"),
];

pub const UNOVA_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Trio Badge", ""),
    ("Basic Badge", ""),
    ("Insect Badge", ""),
    ("Bolt Badge", ""),
    ("Quake Badge", ""),
    ("Jet Badge", ""),
    ("Freeze Badge", ""),
    ("Legend Badge", ""),
    ("Toxic Badge", ""),
    ("Wave Badge", ""),
];

pub const KALOS_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Bug Badge", ""),
    ("Cliff Badge", ""),
    ("Rumble Badge", ""),
    ("Plant Badge", ""),
    ("Voltage Badge", ""),
    ("Fairy Badge", ""),
    ("Psychic Badge", ""),
    ("Iceberg Badge", ""),
];

pub const ALOLA_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Melemele Trial Stamp", ""),
    ("Akala Trial Stamp", ""),
    ("Ula'ula Trial Stamp", ""),
    ("Poni Trial Stamp", ""),
    ("Melemele Stamp", ""),
    ("Akala Stamp", ""),
    ("Ulaula Stamp", ""),
    ("Poni Stamp", ""),
];

pub const GALAR_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Grass Badge", ""),
    ("Water Badge", ""),
    ("Fire Badge", ""),
    ("Fighting Badge", ""),
    ("Ghost Badge", ""),
    ("Fairy Badge", ""),
    ("Rock Badge", ""),
    ("Ice Badge", ""),
    ("Dark Badge", ""),
    ("Dragon Badge", ""),
];

pub const PALDEA_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    ("Cortondo Badge", ""),
    ("Artazon Badge", ""),
    ("Levincia Badge", ""),
    ("Cascarrafa Badge", ""),
    ("Medali Badge", ""),
    ("Montenevera Badge", ""),
    ("Alfornada Badge", ""),
    ("Glaseado Badge", ""),
];

pub const ALL_BADGE_EMOJI_IDS: &[(&str, &str)] = &[
    // Kanto
    ("Boulder Badge", "6212813379283722048"),
    ("Cascade Badge", "6213204006559293292"),
    ("Thunder Badge", "6213102327503527219"),
    ("Rainbow Badge", "6213176918200556197"),
    ("Soul Badge", "6212811657001836714"),
    ("Marsh Badge", "6212936773694135722"),
    ("Volcano Badge", "6213249172435376699"),
    ("Earth Badge", "6212800429957323813"),
    // Johto
    ("Zephyr Badge", "6224448900955118411"),
    ("Hive Badge", "6224361597154892802"),
    ("Plain Badge", "6222191650827935720"),
    ("Fog Badge", "6222162286136531995"),
    ("Storm Badge", "6222261590075383439"),
    ("Mineral Badge", "6221886678085148490"),
    ("Glacier Badge", "6224500363253260486"),
    ("Rising Badge", "6224142244585152552"),
    // Hoenn
    ("Stone Badge", "6233372356937522205"),
    ("Knuckle Badge", "6233543781967209292"),
    ("Dynamo Badge", "6233536368853654054"),
    ("Heat Badge", "6233181943857420831"),
    ("Balance Badge", "6233522951375821687"),
    ("Feather Badge", "6233130859516403656"),
    ("Mind Badge", "6233226143365863801"),
    ("Rain Badge", "6233036434660399461"),
    // Sinnoh
    ("Coal Badge", "6298290882105843328"),
    ("Forest Badge", "6298727667394944238"),
    ("Cobble Badge", "6298369677575856417"),
    ("Fen Badge", "6298490872963015475"),
    ("Relic Badge", "6298760373570904054"),
    ("Mine Badge", "6298669917264682103"),
    ("Icicle Badge", "6298672558669569167"),
    ("Beacon Badge", "6298598019512146938"),
    // Orange Island
    ("Coral-Eye Badge", "6257784409882108221"),
    ("Sea Ruby Badge", "6260452954667491060"),
    ("Spike Shell Badge", "6260169306437328641"),
    ("Jade Star Badge", "6257859211032534512"),
    // Unova
    ("Trio Badge", ""),
    ("Basic Badge", ""),
    ("Insect Badge", ""),
    ("Bolt Badge", ""),
    ("Quake Badge", ""),
    ("Jet Badge", ""),
    ("Freeze Badge", ""),
    ("Legend Badge", ""),
    ("Toxic Badge", ""),
    ("Wave Badge", ""),
    // Kalos
    ("Bug Badge", ""),
    ("Cliff Badge", ""),
    ("Rumble Badge", ""),
    ("Plant Badge", ""),
    ("Voltage Badge", ""),
    ("Fairy Badge", ""),
    ("Psychic Badge", ""),
    ("Iceberg Badge", ""),
    // Alola
    ("Melemele Trial Stamp", ""),
    ("Akala Trial Stamp", ""),
    ("Ula'ula Trial Stamp", ""),
    ("Poni Trial Stamp", ""),
    ("Melemele Stamp", ""),
    ("Akala Stamp", ""),
    ("Ulaula Stamp", ""),
    ("Poni Stamp", ""),
    // Galar
    ("Grass Badge", ""),
    ("Water Badge", ""),
    ("Fire Badge", ""),
    ("Fighting Badge", ""),
    ("Ghost Badge", ""),
    ("Fairy Badge", ""),
    ("Rock Badge", ""),
    ("Ice Badge", ""),
    ("Dark Badge", ""),
    ("Dragon Badge", ""),
    // Paldea
    ("Cortondo Badge", ""),
    ("Artazon Badge", ""),
    ("Levincia Badge", ""),
    ("Cascarrafa Badge", ""),
    ("Medali Badge", ""),
    ("Montenevera Badge", ""),
    ("Alfornada Badge", ""),
    ("Glaseado Badge", ""),
];

pub fn get_badge_emoji(badge_name: &str, html: bool) -> String {
    let b_clean = badge_name.trim().to_lowercase();
    let b_no_spaces = b_clean.replace(' ', "");

    for &(k, v) in ALL_BADGE_EMOJI_IDS {
        let k_lower = k.to_lowercase();
        if k_lower == b_clean || k_lower.replace(' ', "") == b_no_spaces {
            if !v.is_empty() {
                return format_custom_emoji(Some(v), "🌟", html);
            } else if html {
                return format_custom_emoji(Some("6212813379283722048"), "🌟", true);
            } else {
                return "🔘".to_string();
            }
        }
    }

    if html {
        format_custom_emoji(Some("6212813379283722048"), "🌟", true)
    } else {
        "🔘".to_string()
    }
}

pub const DOSE_CUSTOM_EMOJI_ID: &str = "5440739140347907722";
pub const DOSE_FALLBACK: &str = "⚡️";
pub const DOSE_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5440739140347907722">⚡️</tg-emoji>"#;
pub const DOSE_EMOJI_PLAIN: &str = DOSE_FALLBACK;

pub fn get_dose_emoji(html: bool) -> &'static str {
    if html { DOSE_EMOJI_HTML } else { DOSE_EMOJI_PLAIN }
}

// ==============================================================================
// 11. LEADERBOARD & PODIUM RANK EMOJIS
// ==============================================================================

pub const RANK_1_CUSTOM_EMOJI_ID: &str = "5440539497383087970";
pub const RANK_1_FALLBACK: &str = "🥇";
pub const RANK_1_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5440539497383087970">🥇</tg-emoji>"#;
pub const RANK_1_EMOJI_PLAIN: &str = RANK_1_FALLBACK;

pub const RANK_2_CUSTOM_EMOJI_ID: &str = "5447203607294265305";
pub const RANK_2_FALLBACK: &str = "🥈";
pub const RANK_2_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5447203607294265305">🥈</tg-emoji>"#;
pub const RANK_2_EMOJI_PLAIN: &str = RANK_2_FALLBACK;

pub const RANK_3_CUSTOM_EMOJI_ID: &str = "5453902265922376865";
pub const RANK_3_FALLBACK: &str = "🥉";
pub const RANK_3_EMOJI_HTML: &str = r#"<tg-emoji emoji-id="5453902265922376865">🥉</tg-emoji>"#;
pub const RANK_3_EMOJI_PLAIN: &str = RANK_3_FALLBACK;

pub fn get_rank_emoji(rank: usize, html: bool) -> &'static str {
    match rank {
        1 => if html { RANK_1_EMOJI_HTML } else { RANK_1_EMOJI_PLAIN },
        2 => if html { RANK_2_EMOJI_HTML } else { RANK_2_EMOJI_PLAIN },
        3 => if html { RANK_3_EMOJI_HTML } else { RANK_3_EMOJI_PLAIN },
        _ => "",
    }
}

pub fn get_rank_1_emoji(html: bool) -> &'static str {
    get_rank_emoji(1, html)
}

pub fn get_rank_2_emoji(html: bool) -> &'static str {
    get_rank_emoji(2, html)
}

pub fn get_rank_3_emoji(html: bool) -> &'static str {
    get_rank_emoji(3, html)
}

// ==============================================================================
// 12. UNIVERSAL GENERIC GETTER
// ==============================================================================

pub fn get_emoji(name: &str, html: bool) -> String {
    let clean_k = name.to_lowercase().trim().replace([' ', '-'], "_");

    match clean_k.as_str() {
        "dose" => get_dose_emoji(html).to_string(),
        "backward" => get_backward_emoji(html).to_string(),
        "forward" => get_forward_emoji(html).to_string(),
        "inventory" => get_inventory_emoji(html).to_string(),
        "events" => get_events_emoji(html).to_string(),
        "ads" | "ad" => get_ad_emoji(html).to_string(),
        "currency" | "pokedollars" => get_currency_emoji(html).to_string(),
        "exp" => get_exp_emoji(html).to_string(),
        "up_arrow" => get_up_arrow_emoji(html).to_string(),
        "down_arrow" => get_down_arrow_emoji(html).to_string(),
        "omni_ring" | "omni" => get_omni_ring_emoji(html).to_string(),
        "key_items" => get_key_items_emoji(html).to_string(),
        "safari_pass" => get_safari_pass_emoji(html).to_string(),
        "incubator" => get_incubator_emoji(html).to_string(),
        "egg" => get_egg_emoji(html).to_string(),
        "hatched_egg" | "hatched" => get_hatched_egg_emoji(html).to_string(),
        "tms" => get_tms_emoji(html).to_string(),
        "pokeballs_header" => get_pokeballs_header_emoji(html).to_string(),
        "shiny" => get_shiny_emoji(html).to_string(),
        "chibi" => get_chibi_emoji(html).to_string(),
        "catch_star" => get_catch_star_emoji(html).to_string(),
        "caught_star" => get_caught_star_emoji(html).to_string(),
        "catch" => get_catch_emoji(html).to_string(),
        "flee" => get_flee_emoji(html).to_string(),
        "neutral_arrow" => get_neutral_arrow_emoji(html).to_string(),
        "conquered" => get_conquered_emoji(html).to_string(),
        "locked_gym" => get_locked_gym_emoji(html).to_string(),
        "battle_run" => get_battle_run_emoji(html).to_string(),
        "shop" | "mart" => get_shop_emoji(html).to_string(),
        "rank_1" => get_rank_emoji(1, html).to_string(),
        "rank_2" => get_rank_emoji(2, html).to_string(),
        "rank_3" => get_rank_emoji(3, html).to_string(),
        _ => {
            // Check Pokéballs
            if clean_k.ends_with("ball") || get_pokeball_custom_emoji_id(&clean_k).is_some() {
                return get_pokeball_emoji(&clean_k, html).to_string();
            }

            // Check Move Types
            let move_t = get_move_type_emoji(&clean_k, html);
            if move_t != "🔘" {
                return move_t.to_string();
            }

            // Fallback to Item
            get_item_emoji(&clean_k, html)
        }
    }
}
