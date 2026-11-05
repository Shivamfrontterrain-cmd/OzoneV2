use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn format_evolution_ready_message(pokemon_name: &str) -> String {
    format!("<b>{}</b> is ready to evolve", crate::escape_html(pokemon_name))
}

pub fn build_evolution_keyboard(pokemon_db_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Evolve!", format!("poke_evolve:{}", pokemon_db_id)),
    ]])
}

pub fn is_chibi_id(pokemon_id: u32) -> bool {
    (20001..=40000).contains(&pokemon_id)
}

pub fn get_base_species_id_for_chibi(pokemon_id: u32) -> u32 {
    if !is_chibi_id(pokemon_id) {
        return pokemon_id;
    }
    match pokemon_id {
        20010 => 7,
        20011 => 8,
        20012 => 9,
        31020 => 652, // chesnaught
        31022 => 654, // braixen
        31023 => 653, // fennekin
        31024 => 658, // greninja
        31025 => 656, // froakie
        31027 => 658, // greninja
        31030 => 662, // fletchinder
        31032 => 701, // hawlucha
        31035 => 671, // florges
        31037 => 669, // flabebe
        31043 => 696, // tyrunt
        31045 => 688, // binacle
        31046 => 689, // barbaracle
        31047 => 695, // heliolisk
        31049 => 690, // skrelp
        31050 => 691, // dragalge
        31052 => 699, // aurorus
        31053 => 44, // gloom
        31054 => 705, // sliggoo
        31055 => 706, // goodra
        31059 => 713, // avalugg
        31063 => 723, // dartrix
        31064 => 724, // decidueye
        31066 => 726, // torracat
        31067 => 727, // incineroar
        31068 => 728, // popplio
        31071 => 731, // pikipek
        31072 => 732, // trumbeak
        31073 => 733, // toucannon
        31074 => 10112, // grimer-alola
        31075 => 10113, // muk-alola
        31076 => 739, // crabrawler
        31077 => 740, // crabominable
        31078 => 744, // rockruff
        31084 => 747, // mareanie
        31085 => 748, // toxapex
        31086 => 749, // mudbray
        31088 => 757, // salandit
        31089 => 758, // salazzle
        31091 => 759, // stufful
        31092 => 760, // bewear
        31093 => 765, // oranguru
        31096 => 768, // golisopod
        31097 => 769, // sandygast
        31098 => 770, // palossand
        31101 => 10102, // sandslash-alola
        31124 => 10157, // necrozma-ultra
        31129 => 890, // eternatus
        _ => pokemon_id - 20000,
    }
}

pub fn get_chibi_id_for_base(base_id: u32) -> u32 {
    match base_id {
        7 => 20010,
        8 => 20011,
        9 => 20012,
        652 => 31020,
        654 => 31022,
        653 => 31023,
        656 => 31025,
        658 => 31027,
        662 => 31030,
        701 => 31032,
        671 => 31035,
        669 => 31037,
        696 => 31043,
        688 => 31045,
        689 => 31046,
        695 => 31047,
        690 => 31049,
        691 => 31050,
        699 => 31052,
        705 => 31054,
        706 => 31055,
        713 => 31059,
        723 => 31063,
        724 => 31064,
        726 => 31066,
        727 => 31067,
        728 => 31068,
        731 => 31071,
        732 => 31072,
        733 => 31073,
        10112 => 31074,
        10113 => 31075,
        739 => 31076,
        740 => 31077,
        744 => 31078,
        747 => 31084,
        748 => 31085,
        749 => 31086,
        757 => 31088,
        758 => 31089,
        759 => 31091,
        760 => 31092,
        765 => 31093,
        768 => 31096,
        769 => 31097,
        770 => 31098,
        10102 => 31101,
        10157 => 31124,
        890 => 31129,
        _ => 20000 + base_id,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionalEvolutionRule {
    pub base_id: u32,
    pub required_level: u32,
    pub region: &'static str,
    pub target_id: u32,
    pub target_name: &'static str,
}

pub const REGIONAL_EVOLUTION_RULES: &[RegionalEvolutionRule] = &[
    // Hisui (Goomy -> Hisuian Sliggoo at 40, Sliggoo -> Hisuian Goodra at 50, Bergmite -> Hisuian Avalugg at 37)
    RegionalEvolutionRule { base_id: 704, required_level: 40, region: "Hisui", target_id: 10241, target_name: "Sliggoo-Hisui" },
    RegionalEvolutionRule { base_id: 10241, required_level: 50, region: "Hisui", target_id: 10242, target_name: "Goodra-Hisui" },
    RegionalEvolutionRule { base_id: 705, required_level: 50, region: "Hisui", target_id: 10242, target_name: "Goodra-Hisui" },
    RegionalEvolutionRule { base_id: 712, required_level: 37, region: "Hisui", target_id: 10243, target_name: "Avalugg-Hisui" },
    RegionalEvolutionRule { base_id: 156, required_level: 36, region: "Hisui", target_id: 10233, target_name: "Typhlosion-Hisui" },
    RegionalEvolutionRule { base_id: 502, required_level: 36, region: "Hisui", target_id: 10236, target_name: "Samurott-Hisui" },
    RegionalEvolutionRule { base_id: 723, required_level: 34, region: "Hisui", target_id: 10244, target_name: "Decidueye-Hisui" },
    RegionalEvolutionRule { base_id: 548, required_level: 20, region: "Hisui", target_id: 10237, target_name: "Lilligant-Hisui" },
    RegionalEvolutionRule { base_id: 627, required_level: 54, region: "Hisui", target_id: 10240, target_name: "Braviary-Hisui" },
    RegionalEvolutionRule { base_id: 58, required_level: 20, region: "Hisui", target_id: 10230, target_name: "Arcanine-Hisui" },
    RegionalEvolutionRule { base_id: 100, required_level: 30, region: "Hisui", target_id: 10232, target_name: "Electrode-Hisui" },
    RegionalEvolutionRule { base_id: 570, required_level: 30, region: "Hisui", target_id: 10239, target_name: "Zoroark-Hisui" },
    RegionalEvolutionRule { base_id: 123, required_level: 36, region: "Hisui", target_id: 900, target_name: "Kleavor" },
    RegionalEvolutionRule { base_id: 215, required_level: 28, region: "Hisui", target_id: 903, target_name: "Sneasler" },
    RegionalEvolutionRule { base_id: 211, required_level: 28, region: "Hisui", target_id: 904, target_name: "Overqwil" },
    RegionalEvolutionRule { base_id: 217, required_level: 36, region: "Hisui", target_id: 901, target_name: "Ursaluna" },
    RegionalEvolutionRule { base_id: 234, required_level: 20, region: "Hisui", target_id: 899, target_name: "Wyrdeer" },
    RegionalEvolutionRule { base_id: 550, required_level: 20, region: "Hisui", target_id: 902, target_name: "Basculegion" },

    // Alola
    RegionalEvolutionRule { base_id: 25, required_level: 22, region: "Alola", target_id: 10100, target_name: "Raichu-Alola" },
    RegionalEvolutionRule { base_id: 102, required_level: 20, region: "Alola", target_id: 10114, target_name: "Exeggutor-Alola" },
    RegionalEvolutionRule { base_id: 104, required_level: 28, region: "Alola", target_id: 10115, target_name: "Marowak-Alola" },

    // Galar
    RegionalEvolutionRule { base_id: 109, required_level: 35, region: "Galar", target_id: 10167, target_name: "Weezing-Galar" },
    RegionalEvolutionRule { base_id: 439, required_level: 20, region: "Galar", target_id: 10168, target_name: "Mr-Mime-Galar" },
    RegionalEvolutionRule { base_id: 83, required_level: 20, region: "Galar", target_id: 865, target_name: "Sirfetchd" },
    RegionalEvolutionRule { base_id: 222, required_level: 38, region: "Galar", target_id: 864, target_name: "Cursola" },
    RegionalEvolutionRule { base_id: 264, required_level: 35, region: "Galar", target_id: 862, target_name: "Obstagoon" },
    RegionalEvolutionRule { base_id: 562, required_level: 34, region: "Galar", target_id: 867, target_name: "Runerigus" },

    // Paldea
    RegionalEvolutionRule { base_id: 194, required_level: 20, region: "Paldea", target_id: 980, target_name: "Clodsire" },
    RegionalEvolutionRule { base_id: 57, required_level: 36, region: "Paldea", target_id: 979, target_name: "Annihilape" },
    RegionalEvolutionRule { base_id: 203, required_level: 20, region: "Paldea", target_id: 981, target_name: "Farigiraf" },
    RegionalEvolutionRule { base_id: 206, required_level: 20, region: "Paldea", target_id: 982, target_name: "Dudunsparce" },
    RegionalEvolutionRule { base_id: 625, required_level: 36, region: "Paldea", target_id: 983, target_name: "Kingambit" },
];

pub fn matches_region(required_region: &str, current_region: &str) -> bool {
    let req = required_region.trim();
    let cur = current_region.trim();
    if req.eq_ignore_ascii_case(cur) {
        return true;
    }
    // Hisui is ancient Sinnoh: players in Sinnoh or Hisui both match Hisui
    if req.eq_ignore_ascii_case("Hisui") && cur.eq_ignore_ascii_case("Sinnoh") {
        return true;
    }
    if req.eq_ignore_ascii_case("Sinnoh") && cur.eq_ignore_ascii_case("Hisui") {
        return true;
    }
    false
}

pub fn get_evolution_for_region(pokemon_id: u32, region: Option<&str>) -> Option<(u32, &'static str, u32)> {
    if let Some(r) = region {
        for rule in REGIONAL_EVOLUTION_RULES {
            if rule.base_id == pokemon_id && matches_region(rule.region, r) {
                return Some((rule.required_level, rule.target_name, rule.target_id));
            }
        }
    }
    get_static_evolution(pokemon_id)
}

pub fn check_evolution_ready_in_region(
    pokemon_id: u32,
    level: i32,
    region: Option<&str>,
) -> Option<(u32, &'static str, u32)> {
    let (is_chibi, lookup_id) = if is_chibi_id(pokemon_id) {
        (true, get_base_species_id_for_chibi(pokemon_id))
    } else {
        (false, pokemon_id)
    };

    if let Some((req_lvl, target_name, target_base_id)) = get_evolution_for_region(lookup_id, region) {
        if level as u32 >= req_lvl {
            let final_target_id = if is_chibi {
                get_chibi_id_for_base(target_base_id)
            } else {
                target_base_id
            };
            return Some((req_lvl, target_name, final_target_id));
        }
    }
    None
}

pub fn check_evolution_ready(pokemon_id: u32, level: i32) -> Option<(u32, &'static str, u32)> {
    check_evolution_ready_in_region(pokemon_id, level, None)
}

pub fn get_static_evolution(pokemon_id: u32) -> Option<(u32, &'static str, u32)> {
    match pokemon_id {
        1 => Some((16, "Ivysaur", 2)),
        2 => Some((32, "Venusaur", 3)),
        4 => Some((16, "Charmeleon", 5)),
        5 => Some((36, "Charizard", 6)),
        7 => Some((16, "Wartortle", 8)),
        8 => Some((36, "Blastoise", 9)),
        10 => Some((7, "Metapod", 11)),
        11 => Some((10, "Butterfree", 12)),
        13 => Some((7, "Kakuna", 14)),
        14 => Some((10, "Beedrill", 15)),
        16 => Some((18, "Pidgeotto", 17)),
        17 => Some((36, "Pidgeot", 18)),
        19 => Some((20, "Raticate", 20)),
        21 => Some((20, "Fearow", 22)),
        23 => Some((22, "Arbok", 24)),
        25 => Some((22, "Raichu", 26)),
        27 => Some((22, "Sandslash", 28)),
        29 => Some((16, "Nidorina", 30)),
        30 => Some((36, "Nidoqueen", 31)),
        32 => Some((16, "Nidorino", 33)),
        33 => Some((36, "Nidoking", 34)),
        35 => Some((22, "Clefable", 36)),
        37 => Some((22, "Ninetales", 38)),
        39 => Some((22, "Wigglytuff", 40)),
        41 => Some((22, "Golbat", 42)),
        42 => Some((36, "Crobat", 169)),
        43 => Some((21, "Gloom", 44)),
        44 => Some((36, "Vileplume", 45)),
        46 => Some((24, "Parasect", 47)),
        48 => Some((31, "Venomoth", 49)),
        50 => Some((26, "Dugtrio", 51)),
        52 => Some((28, "Persian", 53)),
        54 => Some((33, "Golduck", 55)),
        56 => Some((28, "Primeape", 57)),
        57 => Some((36, "Annihilape", 979)),
        58 => Some((20, "Arcanine", 59)),
        60 => Some((25, "Poliwhirl", 61)),
        61 => Some((36, "Poliwrath", 62)),
        63 => Some((16, "Kadabra", 64)),
        64 => Some((36, "Alakazam", 65)),
        66 => Some((28, "Machoke", 67)),
        67 => Some((36, "Machamp", 68)),
        69 => Some((21, "Weepinbell", 70)),
        70 => Some((36, "Victreebel", 71)),
        72 => Some((30, "Tentacruel", 73)),
        74 => Some((25, "Graveler", 75)),
        75 => Some((36, "Golem", 76)),
        77 => Some((40, "Rapidash", 78)),
        79 => Some((37, "Slowbro", 80)),
        81 => Some((30, "Magneton", 82)),
        82 => Some((36, "Magnezone", 462)),
        83 => Some((20, "Sirfetchd", 865)),
        84 => Some((31, "Dodrio", 85)),
        86 => Some((34, "Dewgong", 87)),
        88 => Some((38, "Muk", 89)),
        90 => Some((20, "Cloyster", 91)),
        92 => Some((25, "Haunter", 93)),
        93 => Some((36, "Gengar", 94)),
        95 => Some((20, "Steelix", 208)),
        96 => Some((26, "Hypno", 97)),
        98 => Some((28, "Kingler", 99)),
        100 => Some((30, "Electrode", 101)),
        102 => Some((20, "Exeggutor", 103)),
        104 => Some((28, "Marowak", 105)),
        108 => Some((20, "Lickilicky", 463)),
        109 => Some((35, "Weezing", 110)),
        111 => Some((42, "Rhydon", 112)),
        112 => Some((36, "Rhyperior", 464)),
        113 => Some((36, "Blissey", 242)),
        114 => Some((20, "Tangrowth", 465)),
        116 => Some((32, "Seadra", 117)),
        117 => Some((36, "Kingdra", 230)),
        118 => Some((33, "Seaking", 119)),
        120 => Some((20, "Starmie", 121)),
        122 => Some((42, "Mr-Rime", 866)),
        123 => Some((20, "Scizor", 212)),
        125 => Some((36, "Electivire", 466)),
        126 => Some((36, "Magmortar", 467)),
        129 => Some((20, "Gyarados", 130)),
        133 => Some((22, "Vaporeon", 134)),
        137 => Some((20, "Porygon2", 233)),
        138 => Some((40, "Omastar", 139)),
        140 => Some((40, "Kabutops", 141)),
        147 => Some((30, "Dragonair", 148)),
        148 => Some((55, "Dragonite", 149)),
        152 => Some((16, "Bayleef", 153)),
        153 => Some((32, "Meganium", 154)),
        155 => Some((14, "Quilava", 156)),
        156 => Some((36, "Typhlosion", 157)),
        158 => Some((18, "Croconaw", 159)),
        159 => Some((30, "Feraligatr", 160)),
        161 => Some((15, "Furret", 162)),
        163 => Some((20, "Noctowl", 164)),
        165 => Some((18, "Ledian", 166)),
        167 => Some((22, "Ariados", 168)),
        170 => Some((27, "Lanturn", 171)),
        172 => Some((20, "Pikachu", 25)),
        173 => Some((20, "Clefairy", 35)),
        174 => Some((20, "Jigglypuff", 39)),
        175 => Some((20, "Togetic", 176)),
        176 => Some((36, "Togekiss", 468)),
        177 => Some((25, "Xatu", 178)),
        179 => Some((15, "Flaaffy", 180)),
        180 => Some((30, "Ampharos", 181)),
        183 => Some((18, "Azumarill", 184)),
        187 => Some((18, "Skiploom", 188)),
        188 => Some((27, "Jumpluff", 189)),
        190 => Some((20, "Ambipom", 424)),
        191 => Some((20, "Sunflora", 192)),
        193 => Some((20, "Yanmega", 469)),
        194 => Some((20, "Quagsire", 195)),
        198 => Some((20, "Honchkrow", 430)),
        200 => Some((20, "Mismagius", 429)),
        203 => Some((20, "Farigiraf", 981)),
        204 => Some((31, "Forretress", 205)),
        206 => Some((20, "Dudunsparce", 982)),
        207 => Some((20, "Gliscor", 472)),
        209 => Some((23, "Granbull", 210)),
        211 => Some((20, "Overqwil", 904)),
        215 => Some((20, "Weavile", 461)),
        216 => Some((30, "Ursaring", 217)),
        217 => Some((36, "Ursaluna", 901)),
        218 => Some((38, "Magcargo", 219)),
        220 => Some((33, "Piloswine", 221)),
        221 => Some((36, "Mamoswine", 473)),
        222 => Some((38, "Cursola", 864)),
        223 => Some((25, "Octillery", 224)),
        228 => Some((24, "Houndoom", 229)),
        231 => Some((25, "Donphan", 232)),
        233 => Some((36, "Porygon-Z", 474)),
        234 => Some((20, "Wyrdeer", 899)),
        236 => Some((20, "Hitmonlee", 106)),
        238 => Some((30, "Jynx", 124)),
        239 => Some((30, "Electabuzz", 125)),
        240 => Some((30, "Magmar", 126)),
        246 => Some((30, "Pupitar", 247)),
        247 => Some((55, "Tyranitar", 248)),
        252 => Some((16, "Grovyle", 253)),
        253 => Some((36, "Sceptile", 254)),
        255 => Some((16, "Combusken", 256)),
        256 => Some((36, "Blaziken", 257)),
        258 => Some((16, "Marshtomp", 259)),
        259 => Some((36, "Swampert", 260)),
        261 => Some((18, "Mightyena", 262)),
        263 => Some((20, "Linoone", 264)),
        264 => Some((35, "Obstagoon", 862)),
        265 => Some((7, "Silcoon", 266)),
        266 => Some((10, "Beautifly", 267)),
        268 => Some((10, "Dustox", 269)),
        270 => Some((14, "Lombre", 271)),
        271 => Some((36, "Ludicolo", 272)),
        273 => Some((14, "Nuzleaf", 274)),
        274 => Some((36, "Shiftry", 275)),
        276 => Some((22, "Swellow", 277)),
        278 => Some((25, "Pelipper", 279)),
        280 => Some((20, "Kirlia", 281)),
        281 => Some((30, "Gardevoir", 282)),
        283 => Some((22, "Masquerain", 284)),
        285 => Some((23, "Breloom", 286)),
        287 => Some((18, "Vigoroth", 288)),
        288 => Some((36, "Slaking", 289)),
        290 => Some((20, "Ninjask", 291)),
        293 => Some((20, "Loudred", 294)),
        294 => Some((40, "Exploud", 295)),
        296 => Some((24, "Hariyama", 297)),
        298 => Some((20, "Marill", 183)),
        299 => Some((20, "Probopass", 476)),
        300 => Some((20, "Delcatty", 301)),
        304 => Some((32, "Lairon", 305)),
        305 => Some((42, "Aggron", 306)),
        307 => Some((37, "Medicham", 308)),
        309 => Some((26, "Manectric", 310)),
        315 => Some((36, "Roserade", 407)),
        316 => Some((26, "Swalot", 317)),
        318 => Some((30, "Sharpedo", 319)),
        320 => Some((40, "Wailord", 321)),
        322 => Some((33, "Camerupt", 323)),
        325 => Some((32, "Grumpig", 326)),
        328 => Some((35, "Vibrava", 329)),
        329 => Some((45, "Flygon", 330)),
        331 => Some((32, "Cacturne", 332)),
        333 => Some((35, "Altaria", 334)),
        339 => Some((30, "Whiscash", 340)),
        341 => Some((30, "Crawdaunt", 342)),
        343 => Some((36, "Claydol", 344)),
        345 => Some((40, "Cradily", 346)),
        347 => Some((40, "Armaldo", 348)),
        349 => Some((20, "Milotic", 350)),
        353 => Some((37, "Banette", 354)),
        355 => Some((37, "Dusclops", 356)),
        356 => Some((36, "Dusknoir", 477)),
        360 => Some((15, "Wobbuffet", 202)),
        361 => Some((42, "Glalie", 362)),
        363 => Some((32, "Sealeo", 364)),
        364 => Some((44, "Walrein", 365)),
        366 => Some((20, "Huntail", 367)),
        371 => Some((30, "Shelgon", 372)),
        372 => Some((50, "Salamence", 373)),
        374 => Some((20, "Metang", 375)),
        375 => Some((45, "Metagross", 376)),
        387 => Some((18, "Grotle", 388)),
        388 => Some((32, "Torterra", 389)),
        390 => Some((14, "Monferno", 391)),
        391 => Some((36, "Infernape", 392)),
        393 => Some((16, "Prinplup", 394)),
        394 => Some((36, "Empoleon", 395)),
        396 => Some((14, "Staravia", 397)),
        397 => Some((34, "Staraptor", 398)),
        399 => Some((15, "Bibarel", 400)),
        401 => Some((10, "Kricketune", 402)),
        403 => Some((15, "Luxio", 404)),
        404 => Some((30, "Luxray", 405)),
        406 => Some((20, "Roselia", 315)),
        408 => Some((30, "Rampardos", 409)),
        410 => Some((30, "Bastiodon", 411)),
        412 => Some((20, "Wormadam", 413)),
        415 => Some((21, "Vespiquen", 416)),
        418 => Some((26, "Floatzel", 419)),
        420 => Some((25, "Cherrim", 421)),
        422 => Some((30, "Gastrodon", 423)),
        425 => Some((28, "Drifblim", 426)),
        427 => Some((20, "Lopunny", 428)),
        431 => Some((38, "Purugly", 432)),
        433 => Some((20, "Chimecho", 358)),
        434 => Some((34, "Skuntank", 435)),
        436 => Some((33, "Bronzong", 437)),
        438 => Some((20, "Sudowoodo", 185)),
        439 => Some((20, "Mr-Mime", 122)),
        440 => Some((20, "Chansey", 113)),
        443 => Some((24, "Gabite", 444)),
        444 => Some((48, "Garchomp", 445)),
        446 => Some((20, "Snorlax", 143)),
        447 => Some((20, "Lucario", 448)),
        449 => Some((34, "Hippowdon", 450)),
        451 => Some((40, "Drapion", 452)),
        453 => Some((37, "Toxicroak", 454)),
        456 => Some((31, "Lumineon", 457)),
        458 => Some((20, "Mantine", 226)),
        459 => Some((40, "Abomasnow", 460)),
        495 => Some((17, "Servine", 496)),
        496 => Some((36, "Serperior", 497)),
        498 => Some((17, "Pignite", 499)),
        499 => Some((36, "Emboar", 500)),
        501 => Some((17, "Dewott", 502)),
        502 => Some((36, "Samurott", 503)),
        504 => Some((20, "Watchog", 505)),
        506 => Some((16, "Herdier", 507)),
        507 => Some((32, "Stoutland", 508)),
        509 => Some((20, "Liepard", 510)),
        511 => Some((20, "Simisage", 512)),
        513 => Some((20, "Simisear", 514)),
        515 => Some((20, "Simipour", 516)),
        517 => Some((20, "Musharna", 518)),
        519 => Some((21, "Tranquill", 520)),
        520 => Some((32, "Unfezant", 521)),
        522 => Some((27, "Zebstrika", 523)),
        524 => Some((25, "Boldore", 525)),
        525 => Some((36, "Gigalith", 526)),
        527 => Some((20, "Swoobat", 528)),
        529 => Some((31, "Excadrill", 530)),
        532 => Some((25, "Gurdurr", 533)),
        533 => Some((36, "Conkeldurr", 534)),
        535 => Some((25, "Palpitoad", 536)),
        536 => Some((36, "Seismitoad", 537)),
        540 => Some((20, "Swadloon", 541)),
        541 => Some((36, "Leavanny", 542)),
        543 => Some((22, "Whirlipede", 544)),
        544 => Some((30, "Scolipede", 545)),
        546 => Some((20, "Whimsicott", 547)),
        548 => Some((20, "Lilligant", 549)),
        550 => Some((20, "Basculegion", 902)),
        551 => Some((29, "Krokorok", 552)),
        552 => Some((40, "Krookodile", 553)),
        554 => Some((35, "Darmanitan", 555)),
        557 => Some((34, "Crustle", 558)),
        559 => Some((39, "Scrafty", 560)),
        562 => Some((34, "Cofagrigus", 563)),
        564 => Some((37, "Carracosta", 565)),
        566 => Some((37, "Archeops", 567)),
        568 => Some((36, "Garbodor", 569)),
        570 => Some((30, "Zoroark", 571)),
        572 => Some((20, "Cinccino", 573)),
        574 => Some((32, "Gothorita", 575)),
        575 => Some((41, "Gothitelle", 576)),
        577 => Some((32, "Duosion", 578)),
        578 => Some((41, "Reuniclus", 579)),
        580 => Some((35, "Swanna", 581)),
        582 => Some((35, "Vanillish", 583)),
        583 => Some((47, "Vanilluxe", 584)),
        585 => Some((34, "Sawsbuck", 586)),
        588 => Some((20, "Escavalier", 589)),
        590 => Some((39, "Amoonguss", 591)),
        592 => Some((40, "Jellicent", 593)),
        595 => Some((36, "Galvantula", 596)),
        597 => Some((40, "Ferrothorn", 598)),
        599 => Some((38, "Klang", 600)),
        600 => Some((49, "Klinklang", 601)),
        602 => Some((39, "Eelektrik", 603)),
        603 => Some((36, "Eelektross", 604)),
        605 => Some((42, "Beheeyem", 606)),
        607 => Some((41, "Lampent", 608)),
        608 => Some((36, "Chandelure", 609)),
        610 => Some((38, "Fraxure", 611)),
        611 => Some((48, "Haxorus", 612)),
        613 => Some((37, "Beartic", 614)),
        616 => Some((20, "Accelgor", 617)),
        619 => Some((50, "Mienshao", 620)),
        622 => Some((43, "Golurk", 623)),
        624 => Some((52, "Bisharp", 625)),
        625 => Some((36, "Kingambit", 983)),
        627 => Some((54, "Braviary", 628)),
        629 => Some((54, "Mandibuzz", 630)),
        633 => Some((50, "Zweilous", 634)),
        634 => Some((64, "Hydreigon", 635)),
        636 => Some((59, "Volcarona", 637)),
        650 => Some((16, "Quilladin", 651)),
        651 => Some((36, "Chesnaught", 652)),
        653 => Some((16, "Braixen", 654)),
        654 => Some((36, "Delphox", 655)),
        656 => Some((16, "Frogadier", 657)),
        657 => Some((36, "Greninja", 658)),
        659 => Some((20, "Diggersby", 660)),
        661 => Some((17, "Fletchinder", 662)),
        662 => Some((35, "Talonflame", 663)),
        664 => Some((9, "Spewpa", 665)),
        665 => Some((12, "Vivillon", 666)),
        667 => Some((35, "Pyroar", 668)),
        669 => Some((19, "Floette", 670)),
        670 => Some((36, "Florges", 671)),
        672 => Some((32, "Gogoat", 673)),
        674 => Some((32, "Pangoro", 675)),
        677 => Some((25, "Meowstic", 678)),
        679 => Some((35, "Doublade", 680)),
        680 => Some((36, "Aegislash", 681)),
        682 => Some((20, "Aromatisse", 683)),
        684 => Some((20, "Slurpuff", 685)),
        686 => Some((30, "Malamar", 687)),
        688 => Some((39, "Barbaracle", 689)),
        690 => Some((48, "Dragalge", 691)),
        692 => Some((37, "Clawitzer", 693)),
        694 => Some((20, "Heliolisk", 695)),
        696 => Some((39, "Tyrantrum", 697)),
        698 => Some((39, "Aurorus", 699)),
        704 => Some((40, "Sliggoo", 705)),
        705 => Some((50, "Goodra", 706)),
        708 => Some((20, "Trevenant", 709)),
        710 => Some((20, "Gourgeist", 711)),
        712 => Some((37, "Avalugg", 713)),
        714 => Some((48, "Noivern", 715)),
        722 => Some((17, "Dartrix", 723)),
        723 => Some((34, "Decidueye", 724)),
        725 => Some((17, "Torracat", 726)),
        726 => Some((34, "Incineroar", 727)),
        728 => Some((17, "Brionne", 729)),
        729 => Some((34, "Primarina", 730)),
        731 => Some((14, "Trumbeak", 732)),
        732 => Some((28, "Toucannon", 733)),
        734 => Some((20, "Gumshoos", 735)),
        736 => Some((20, "Charjabug", 737)),
        737 => Some((36, "Vikavolt", 738)),
        739 => Some((20, "Crabominable", 740)),
        742 => Some((25, "Ribombee", 743)),
        744 => Some((25, "Lycanroc", 745)),
        747 => Some((38, "Toxapex", 748)),
        749 => Some((30, "Mudsdale", 750)),
        751 => Some((22, "Araquanid", 752)),
        753 => Some((34, "Lurantis", 754)),
        755 => Some((24, "Shiinotic", 756)),
        757 => Some((33, "Salazzle", 758)),
        759 => Some((27, "Bewear", 760)),
        761 => Some((18, "Steenee", 762)),
        762 => Some((36, "Tsareena", 763)),
        767 => Some((30, "Golisopod", 768)),
        769 => Some((42, "Palossand", 770)),
        772 => Some((20, "Silvally", 773)),
        782 => Some((35, "Hakamo-O", 783)),
        783 => Some((45, "Kommo-O", 784)),
        789 => Some((43, "Cosmoem", 790)),
        790 => Some((53, "Solgaleo", 791)),
        803 => Some((20, "Naganadel", 804)),
        808 => Some((20, "Melmetal", 809)),
        810 => Some((16, "Thwackey", 811)),
        811 => Some((35, "Rillaboom", 812)),
        813 => Some((16, "Raboot", 814)),
        814 => Some((35, "Cinderace", 815)),
        816 => Some((16, "Drizzile", 817)),
        817 => Some((35, "Inteleon", 818)),
        819 => Some((24, "Greedent", 820)),
        821 => Some((18, "Corvisquire", 822)),
        822 => Some((38, "Corviknight", 823)),
        824 => Some((10, "Dottler", 825)),
        825 => Some((30, "Orbeetle", 826)),
        827 => Some((18, "Thievul", 828)),
        829 => Some((20, "Eldegoss", 830)),
        831 => Some((24, "Dubwool", 832)),
        833 => Some((22, "Drednaw", 834)),
        835 => Some((25, "Boltund", 836)),
        837 => Some((18, "Carkol", 838)),
        838 => Some((34, "Coalossal", 839)),
        840 => Some((20, "Flapple", 841)),
        843 => Some((36, "Sandaconda", 844)),
        846 => Some((26, "Barraskewda", 847)),
        848 => Some((30, "Toxtricity", 849)),
        850 => Some((28, "Centiskorch", 851)),
        852 => Some((20, "Grapploct", 853)),
        854 => Some((20, "Polteageist", 855)),
        856 => Some((32, "Hattrem", 857)),
        857 => Some((42, "Hatterene", 858)),
        859 => Some((32, "Morgrem", 860)),
        860 => Some((42, "Grimmsnarl", 861)),
        868 => Some((20, "Alcremie", 869)),
        872 => Some((20, "Frosmoth", 873)),
        878 => Some((34, "Copperajah", 879)),
        884 => Some((20, "Archaludon", 1018)),
        885 => Some((50, "Drakloak", 886)),
        886 => Some((60, "Dragapult", 887)),
        891 => Some((20, "Urshifu", 892)),
        906 => Some((16, "Floragato", 907)),
        907 => Some((36, "Meowscarada", 908)),
        909 => Some((16, "Crocalor", 910)),
        910 => Some((36, "Skeledirge", 911)),
        912 => Some((16, "Quaxwell", 913)),
        913 => Some((36, "Quaquaval", 914)),
        915 => Some((18, "Oinkologne", 916)),
        917 => Some((15, "Spidops", 918)),
        919 => Some((24, "Lokix", 920)),
        921 => Some((18, "Pawmo", 922)),
        922 => Some((36, "Pawmot", 923)),
        924 => Some((25, "Maushold", 925)),
        926 => Some((26, "Dachsbun", 927)),
        928 => Some((25, "Dolliv", 929)),
        929 => Some((35, "Arboliva", 930)),
        932 => Some((24, "Naclstack", 933)),
        933 => Some((38, "Garganacl", 934)),
        935 => Some((20, "Armarouge", 936)),
        938 => Some((20, "Bellibolt", 939)),
        940 => Some((25, "Kilowattrel", 941)),
        942 => Some((30, "Mabosstiff", 943)),
        944 => Some((28, "Grafaiai", 945)),
        946 => Some((20, "Brambleghast", 947)),
        948 => Some((30, "Toedscruel", 949)),
        951 => Some((20, "Scovillain", 952)),
        953 => Some((20, "Rabsca", 954)),
        955 => Some((35, "Espathra", 956)),
        957 => Some((24, "Tinkatuff", 958)),
        958 => Some((38, "Tinkaton", 959)),
        960 => Some((26, "Wugtrio", 961)),
        963 => Some((38, "Palafin", 964)),
        964 => Some((40, "Palafin-Hero", 10256)),
        965 => Some((40, "Revavroom", 966)),
        969 => Some((35, "Glimmora", 970)),
        971 => Some((30, "Houndstone", 972)),
        974 => Some((20, "Cetitan", 975)),
        996 => Some((35, "Arctibax", 997)),
        997 => Some((54, "Baxcalibur", 998)),
        999 => Some((20, "Gholdengo", 1000)),
        1011 => Some((36, "Hydrapple", 1019)),
        1012 => Some((20, "Sinistcha", 1013)),
        10091 => Some((20, "Raticate-Alola", 10092)),
        10101 => Some((22, "Sandslash-Alola", 10102)),
        10103 => Some((22, "Ninetales-Alola", 10104)),
        10105 => Some((26, "Dugtrio-Alola", 10106)),
        10107 => Some((28, "Persian-Alola", 10108)),
        10109 => Some((25, "Graveler-Alola", 10110)),
        10110 => Some((36, "Golem-Alola", 10111)),
        10112 => Some((38, "Muk-Alola", 10113)),
        10161 => Some((28, "Perrserker", 863)),
        10162 => Some((40, "Rapidash-Galar", 10163)),
        10164 => Some((37, "Slowbro-Galar", 10165)),
        10166 => Some((28, "Sirfetchd", 865)),
        10168 => Some((42, "Mr-Rime", 866)),
        10173 => Some((38, "Cursola", 864)),
        10174 => Some((20, "Linoone-Galar", 10175)),
        10175 => Some((35, "Obstagoon", 862)),
        10176 => Some((35, "Darmanitan-Galar", 10177)),
        10179 => Some((34, "Runerigus", 867)),
        10229 => Some((20, "Arcanine-Hisui", 10230)),
        10231 => Some((20, "Electrode-Hisui", 10232)),
        10234 => Some((28, "Overqwil", 904)),
        10235 => Some((28, "Sneasler", 903)),
        10238 => Some((30, "Zoroark-Hisui", 10239)),
        10241 => Some((50, "Goodra-Hisui", 10242)),
        10247 => Some((36, "Basculegion", 902)),
        10253 => Some((20, "Clodsire", 980)),
        _ => None,
    }
}

pub fn get_pre_evolution(target_id: u32) -> Option<u32> {
    let (is_chibi, lookup_target) = if is_chibi_id(target_id) {
        (true, get_base_species_id_for_chibi(target_id))
    } else {
        (false, target_id)
    };

    for rule in REGIONAL_EVOLUTION_RULES {
        if rule.target_id == lookup_target {
            return if is_chibi {
                Some(get_chibi_id_for_base(rule.base_id))
            } else {
                Some(rule.base_id)
            };
        }
    }

    for id in (1..=1025).chain(10001..=10260) {
        if let Some((_, _, tid)) = get_static_evolution(id) {
            if tid == lookup_target {
                return if is_chibi {
                    Some(get_chibi_id_for_base(id))
                } else {
                    Some(id)
                };
            }
        }
    }
    None
}

