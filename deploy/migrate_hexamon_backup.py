#!/usr/bin/env python3
"""
Hexamon Backup to Ozone Pokémon SQLite Database Migration Script
Safely migrates:
- users (658 users)
- user_inventory (balls, items, key items, stones, orbs, TMs, tutors)
- user_pokemon (89,511 pokemon with stats, IVs, EVs, moves, boosted IVs, relearn moves)
- user_seen_pokemon (pokedex seen/caught entries)
- user_teams (active teams and team slots)
- active_incubations (user_eggs)
- referrals (completed and pending referral tracking)
- unique_creation_gods (claimed Giratina, Dialga, Palkia, Arceus)
- gym_configs (custom leader teams, fees, rules, and rewards)
"""

import sys
import os
import json
import sqlite3
import zipfile
import re
from datetime import datetime

ZIP_PATH = "hexamon_backup_20261105_150149.zip"
DB_PATH = "pokemon_bot.db"

def safe_int(v, default=0):
    if v is None:
        return default
    if isinstance(v, list):
        return len(v)
    try:
        return int(v)
    except:
        return default

def safe_float(v, default=0.0):
    if v is None:
        return default
    try:
        return float(v)
    except:
        return default

def parse_iso_date(d_val):
    if not d_val:
        return datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S")
    if isinstance(d_val, dict) and "$date" in d_val:
        return str(d_val["$date"]).replace("T", " ").replace("Z", "").split(".")[0]
    s = str(d_val)
    return s.replace("T", " ").replace("Z", "").split(".")[0]

def map_sort_option(val):
    if not val: return 4
    v = str(val).lower()
    if 'time' in v or 'caught' in v or 'order' in v: return 1
    if 'dex' in v or 'pokedex' in v or 'number' in v: return 2
    if 'level' in v or 'lvl' in v: return 3
    if 'iv' in v: return 4
    if 'ev' in v: return 5
    if 'name' in v: return 6
    if 'nature' in v: return 7
    if 'type' in v: return 8
    if 'catch' in v: return 9
    if 'hp' in v: return 10
    return 4

def map_display_option(val):
    if not val: return 5
    v = str(val).lower()
    if 'none' in v: return 1
    if 'level' in v or 'lvl' in v: return 2
    if 'iv' in v: return 3
    if 'ev' in v: return 4
    if 'nature' in v: return 5
    if 'symbol' in v: return 7
    if 'type' in v: return 6
    if 'catch' in v: return 8
    if 'hp' in v: return 9
    if 'attack' in v: return 10
    return 5

def get_unlocked_regions_for_level(lvl):
    all_regs = ["Kanto", "Johto", "Hoenn", "Sinnoh", "Unova", "Kalos", "Alola", "Galar", "Paldea"]
    count = min(lvl + 1, len(all_regs))
    return ",".join(all_regs[:count])

def extract_catalogs():
    with open('src/main.rs', 'r', encoding='utf-8') as f:
        content = f.read()

    tms = {}
    for m in re.finditer(r'ShopTm\s*\{\s*number:\s*(\d+),\s*name:\s*"([^"]+)"', content):
        tms[int(m.group(1))] = m.group(2)

    tutors = {}
    for m in re.finditer(r'TutorMove\s*\{\s*number:\s*(\d+),\s*name:\s*"([^"]+)"', content):
        tutors[int(m.group(1))] = m.group(2)

    return tms, tutors

def run_migration():
    print(f"[INFO] Opening {ZIP_PATH}...")
    if not os.path.exists(ZIP_PATH):
        print(f"[ERROR] {ZIP_PATH} not found!")
        sys.exit(1)

    tm_catalog, tutor_catalog = extract_catalogs()
    print(f"[INFO] Loaded catalogs: {len(tm_catalog)} TMs, {len(tutor_catalog)} Tutors")

    con = sqlite3.connect(DB_PATH)
    cur = con.cursor()

    seen_set = set()

    with zipfile.ZipFile(ZIP_PATH, 'r') as z:
        namelist = set(z.namelist())

        # -------------------------------------------------------------
        # 1. Load user_inventory into memory map
        # -------------------------------------------------------------
        inv_map = {}
        if "user_inventory.json" in namelist:
            inv_data = json.loads(z.read("user_inventory.json").decode("utf-8"))
            for inv in inv_data:
                u_id = inv.get("user_id")
                if u_id:
                    inv_map[int(u_id)] = inv
            print(f"[INFO] Loaded inventory for {len(inv_map)} users")

        # -------------------------------------------------------------
        # 2. Migrate users
        # -------------------------------------------------------------
        if "users.json" in namelist:
            users_data = json.loads(z.read("users.json").decode("utf-8"))
            print(f"[INFO] Migrating {len(users_data)} users...")
            user_rows = []
            user_gym_rows = []

            for u in users_data:
                uid = int(u["user_id"])
                username = u.get("username")
                first_name = u.get("first_name") or ""
                starter_id = safe_int(u.get("starter_chosen"), None)
                current_region = u.get("current_region") or "Kanto"
                trainer_lvl = safe_int(u.get("trainer_level") or u.get("level"), 0)
                trainer_exp = safe_int(u.get("trainer_exp"), 0)
                unlocked_regions = get_unlocked_regions_for_level(trainer_lvl)
                wins = safe_int(u.get("wins"), 0)
                losses = safe_int(u.get("losses"), 0)
                pokedollars = safe_int(u.get("pokedollars"), 0)

                # Gyms beaten / Badges
                gym_prog = u.get("gym_progress") or {}
                badges_count = 0
                if isinstance(gym_prog, dict):
                    for r_name, b_count in gym_prog.items():
                        if isinstance(b_count, (int, float)):
                            b_int = int(b_count)
                            badges_count += b_int
                            user_gym_rows.append((uid, str(r_name).lower(), b_int))

                page_size = safe_int(u.get("poke_page_size"), 20)
                sort_opt = map_sort_option(u.get("poke_sort_by"))
                sort_dir = str(u.get("poke_sort_order") or "Descending")
                disp_opt = map_display_option(u.get("poke_display_by"))
                show_num = safe_int(u.get("poke_show_numbering"), 1)
                active_team = safe_int(u.get("active_team_slot"), 1)

                card_theme = str(u.get("card_theme") or "Bronze Copper")
                card_text_color = str(u.get("card_text_color") or "Bronze Gold")
                card_avatar = str(u.get("card_avatar") or "assets/avatars/captain.png")
                if not card_avatar.startswith("assets/avatars/"):
                    card_avatar = f"assets/avatars/{card_avatar.lower().replace(' ', '_')}.png"

                # Halloween fields
                h_candies = safe_int(u.get("halloween_candies"), 0)
                h_shards = safe_int(u.get("halloween_void_shards"), 0)
                h_tot_candies = safe_int(u.get("halloween_total_candies_earned"), 0)
                h_tot_shards = safe_int(u.get("halloween_total_shards_earned"), 0)
                h_spins = safe_int(u.get("halloween_cauldron_spins"), 0)
                h_mega = str(u.get("halloween_megastone_bought") or "")
                h_voucher = safe_float(u.get("halloween_catch_rate_voucher_expires"), 0.0)

                # Pull items and balls from user_inventory
                inv = inv_map.get(uid, {})
                ball_reg = safe_int(inv.get("pokeball"), 100)
                ball_great = safe_int(inv.get("greatball"), 50)
                ball_ultra = safe_int(inv.get("ultraball"), 30)
                ball_repeat = safe_int(inv.get("repeatball"), 20)
                ball_master = safe_int(inv.get("masterball"), 5)
                ball_dusk = safe_int(inv.get("duskball"), 20)
                ball_nest = safe_int(inv.get("nestball"), 0)

                vitamins = safe_int(inv.get("vitamin"), 100)
                berries = safe_int(inv.get("berry"), 100)
                candies = safe_int(inv.get("rare_candy") or inv.get("rarecandy"), 100)
                null_candies = safe_int(inv.get("null_candy"), 0)
                boosts = safe_int(inv.get("boost"), 10)
                omni_ring = safe_int(inv.get("omni_ring"), 0)
                egg_incubator = safe_int(inv.get("egg_incubator"), 0)

                # Seen & caught lists
                pc_list = u.get("pokedex_caught") or []
                ps_list = u.get("pokedex_seen") or []
                p_caught = len(pc_list) if isinstance(pc_list, list) else safe_int(pc_list, 0)
                p_seen = len(ps_list) if isinstance(ps_list, list) else safe_int(ps_list, 0)

                now_str = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S")
                if isinstance(ps_list, list):
                    for pid in ps_list:
                        if isinstance(pid, int):
                            seen_set.add((uid, pid, now_str))
                if isinstance(pc_list, list):
                    for pid in pc_list:
                        if isinstance(pid, int):
                            seen_set.add((uid, pid, now_str))

                user_rows.append((
                    uid, username, first_name, starter_id, current_region, unlocked_regions,
                    trainer_lvl, trainer_exp, wins, losses, badges_count, pokedollars,
                    card_avatar, 0, p_caught, p_seen, p_caught,
                    ball_reg, ball_great, ball_ultra, ball_repeat, ball_master, ball_dusk,
                    page_size, sort_opt, sort_dir, disp_opt, show_num,
                    vitamins, berries, candies, boosts, ball_nest, omni_ring, egg_incubator, null_candies,
                    active_team, card_theme, card_text_color, card_avatar,
                    h_candies, h_shards, h_tot_candies, h_tot_shards, h_spins, h_mega, h_voucher
                ))

            cur.executemany("""
                INSERT OR REPLACE INTO users (
                    id, username, first_name, starter_id, current_region, unlocked_regions,
                    trainer_level, trainer_exp, wins, losses, badges_count, pokedollars,
                    avatar, pokeballs_used, pokemons_caught, pokemon_seen, pokemon_obtained,
                    ball_regular, ball_great, ball_ultra, ball_repeat, ball_master, ball_dusk,
                    page_size, sort_option, sort_direction, display_option, show_numbering,
                    vitamins, berries, candies, boosts, ball_nest, omni_ring, egg_incubator, null_candies,
                    active_team, card_theme, card_text_color, card_avatar,
                    halloween_candies, halloween_void_shards, halloween_total_candies_earned, halloween_total_shards_earned,
                    halloween_cauldron_spins, halloween_megastone_bought, halloween_catch_rate_voucher_expires
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, user_rows)

            if user_gym_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO user_gyms (user_id, region, gyms_beaten) VALUES (?, ?, ?)
                """, user_gym_rows)

            con.commit()
            print(f"[OK] Migrated {len(user_rows)} users into SQLite")

        # -------------------------------------------------------------
        # 3. Migrate user_inventory (Key items, TMs, Tutors)
        # -------------------------------------------------------------
        key_items_rows = []
        tms_rows = []
        tutors_rows = []

        for uid, inv in inv_map.items():
            # Mega Stones
            stones = inv.get("stones") or {}
            if isinstance(stones, str):
                try: stones = json.loads(stones)
                except: stones = {}
            if isinstance(stones, dict):
                for st_name, qty in stones.items():
                    if qty and safe_int(qty) > 0:
                        key_items_rows.append((uid, st_name.strip(), safe_int(qty)))

            # Orbs / Key Items
            orbs = inv.get("orbs") or {}
            if isinstance(orbs, str):
                try: orbs = json.loads(orbs)
                except: orbs = {}
            if isinstance(orbs, dict):
                for orb_name, qty in orbs.items():
                    if qty and safe_int(qty) > 0:
                        key_items_rows.append((uid, orb_name.strip(), safe_int(qty)))

            # TMs
            tms_dict = inv.get("tms") or {}
            if isinstance(tms_dict, str):
                try: tms_dict = json.loads(tms_dict)
                except: tms_dict = {}
            if isinstance(tms_dict, dict):
                for tm_k, qty in tms_dict.items():
                    if qty and safe_int(qty) > 0:
                        m = re.search(r'\d+', str(tm_k))
                        if m:
                            tm_num = int(m.group(0))
                            move_name = tm_catalog.get(tm_num, f"TM{tm_num}")
                            tms_rows.append((uid, tm_num, move_name, safe_int(qty)))

            # Tutors
            tutors_dict = inv.get("tutors") or {}
            if isinstance(tutors_dict, str):
                try: tutors_dict = json.loads(tutors_dict)
                except: tutors_dict = {}
            if isinstance(tutors_dict, dict):
                for tut_k, qty in tutors_dict.items():
                    if qty and safe_int(qty) > 0:
                        m = re.search(r'\d+', str(tut_k))
                        if m:
                            tut_num = int(m.group(0))
                            move_name = tutor_catalog.get(tut_num, f"Tutor{tut_num}")
                            tutors_rows.append((uid, tut_num, move_name, safe_int(qty)))

        if key_items_rows:
            cur.executemany("INSERT OR REPLACE INTO user_key_items (user_id, item_name, quantity) VALUES (?, ?, ?)", key_items_rows)
        if tms_rows:
            cur.executemany("INSERT OR REPLACE INTO user_tms (user_id, tm_number, move_name, quantity) VALUES (?, ?, ?, ?)", tms_rows)
        if tutors_rows:
            cur.executemany("INSERT OR REPLACE INTO user_tutors (user_id, tutor_number, move_name, quantity) VALUES (?, ?, ?, ?)", tutors_rows)
        con.commit()
        print(f"[OK] Migrated {len(key_items_rows)} key items, {len(tms_rows)} TMs, {len(tutors_rows)} Tutors")

        # -------------------------------------------------------------
        # 4. Migrate user_pokemon (89,511 Pokémon in batches)
        # -------------------------------------------------------------
        if "user_pokemon.json" in namelist:
            pokes_data = json.loads(z.read("user_pokemon.json").decode("utf-8"))
            print(f"[INFO] Migrating {len(pokes_data)} Pokémon...")
            poke_rows = []
            batch_size = 5000

            for p in pokes_data:
                pid = safe_int(p["id"])
                uid = safe_int(p["user_id"])
                dex_id = safe_int(p["pokemon_id"])
                nickname = p.get("nickname") or p.get("name") or "Pokemon"
                lvl = safe_int(p.get("level"), 1)
                nature = str(p.get("nature") or "Hardy")
                iv_hp = safe_int(p.get("iv_hp"), 0)
                iv_atk = safe_int(p.get("iv_attack"), 0)
                iv_def = safe_int(p.get("iv_defense"), 0)
                iv_spa = safe_int(p.get("iv_sp_attack"), 0)
                iv_spd = safe_int(p.get("iv_sp_defense"), 0)
                iv_spe = safe_int(p.get("iv_speed"), 0)

                ev_hp = safe_int(p.get("ev_hp"), 0)
                ev_atk = safe_int(p.get("ev_attack"), 0)
                ev_def = safe_int(p.get("ev_defense"), 0)
                ev_spa = safe_int(p.get("ev_sp_attack"), 0)
                ev_spd = safe_int(p.get("ev_sp_defense"), 0)
                ev_spe = safe_int(p.get("ev_speed"), 0)

                created_at = parse_iso_date(p.get("created_at"))
                is_starter = safe_int(p.get("is_starter"), 0)
                is_shiny = safe_int(p.get("is_shiny"), 0)
                is_hatched = safe_int(p.get("is_hatched") or p.get("egg_hatched"), 0)
                exp = safe_int(p.get("exp"), 0)

                # Moves
                moves = p.get("moves")
                if not moves and p.get("moves_json"):
                    try: moves = json.loads(p["moves_json"])
                    except: moves = []
                if not isinstance(moves, list):
                    moves = []
                m1 = moves[0] if len(moves) > 0 else None
                m2 = moves[1] if len(moves) > 1 else None
                m3 = moves[2] if len(moves) > 2 else None
                m4 = moves[3] if len(moves) > 3 else None

                # Relearn moves
                relearn = p.get("skipped_moves_json") or p.get("forgotten_moves_json") or "[]"
                if isinstance(relearn, list):
                    relearn = json.dumps(relearn)

                # Boosted IVs string: "atk,hp"
                boosted_stats = []
                if safe_int(p.get("iv_boost_hp"), 0) > 0: boosted_stats.append("hp")
                if safe_int(p.get("iv_boost_attack"), 0) > 0: boosted_stats.append("atk")
                if safe_int(p.get("iv_boost_defense"), 0) > 0: boosted_stats.append("def")
                if safe_int(p.get("iv_boost_sp_attack"), 0) > 0: boosted_stats.append("spa")
                if safe_int(p.get("iv_boost_sp_defense"), 0) > 0: boosted_stats.append("spd")
                if safe_int(p.get("iv_boost_speed"), 0) > 0: boosted_stats.append("spe")
                boosted_ivs = ",".join(boosted_stats)

                poke_rows.append((
                    pid, uid, dex_id, nickname, lvl, nature,
                    iv_hp, iv_atk, iv_def, iv_spa, iv_spd, iv_spe,
                    ev_hp, ev_atk, ev_def, ev_spa, ev_spd, ev_spe,
                    m1, m2, m3, m4,
                    is_starter, is_shiny, is_hatched,
                    boosted_ivs, relearn, exp, created_at
                ))
                seen_set.add((uid, dex_id, created_at))

                if len(poke_rows) >= batch_size:
                    cur.executemany("""
                        INSERT OR REPLACE INTO pokemons (
                            id, user_id, pokemon_id, nickname, level, nature,
                            iv_hp, iv_attack, iv_defense, iv_sp_attack, iv_sp_defense, iv_speed,
                            ev_hp, ev_attack, ev_defense, ev_sp_attack, ev_sp_defense, ev_speed,
                            move_1, move_2, move_3, move_4,
                            is_starter, is_shiny, is_hatched,
                            boosted_ivs, relearn_moves, experience, created_at
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, poke_rows)
                    con.commit()
                    print(f"  ...inserted up to pokemon id {pid}...")
                    poke_rows.clear()

            if poke_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO pokemons (
                        id, user_id, pokemon_id, nickname, level, nature,
                        iv_hp, iv_attack, iv_defense, iv_sp_attack, iv_sp_defense, iv_speed,
                        ev_hp, ev_attack, ev_defense, ev_sp_attack, ev_sp_defense, ev_speed,
                        move_1, move_2, move_3, move_4,
                        is_starter, is_shiny, is_hatched,
                        boosted_ivs, relearn_moves, experience, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, poke_rows)
                con.commit()
                poke_rows.clear()

            print(f"[OK] Migrated all {len(pokes_data)} Pokémon into SQLite")

            # Insert seen pokemon into user_seen_pokemon
            print(f"[INFO] Populating user_seen_pokemon ({len(seen_set)} entries)...")
            seen_rows = list(seen_set)
            for i in range(0, len(seen_rows), batch_size):
                cur.executemany("""
                    INSERT OR IGNORE INTO user_seen_pokemon (user_id, pokemon_id, created_at)
                    VALUES (?, ?, ?)
                """, seen_rows[i:i+batch_size])
            con.commit()
            print("[OK] user_seen_pokemon populated")

        # -------------------------------------------------------------
        # 5. Migrate user_teams
        # -------------------------------------------------------------
        if "user_teams.json" in namelist:
            teams_data = json.loads(z.read("user_teams.json").decode("utf-8"))
            team_rows = []
            for t in teams_data:
                uid = safe_int(t["user_id"])
                t_slot = safe_int(t.get("team_slot"), 1)
                p_ids = t.get("pokemon_ids_json") or "[]"
                if isinstance(p_ids, str):
                    try: p_ids = json.loads(p_ids)
                    except: p_ids = []
                if isinstance(p_ids, list):
                    for idx, poke_db_id in enumerate(p_ids[:6]):
                        if poke_db_id:
                            team_rows.append((uid, t_slot, idx + 1, safe_int(poke_db_id)))

            if team_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO user_teams (user_id, team_num, slot, pokemon_db_id)
                    VALUES (?, ?, ?, ?)
                """, team_rows)
                con.commit()
                print(f"[OK] Migrated {len(team_rows)} team slots across {len(teams_data)} user teams")

        # -------------------------------------------------------------
        # 6. Migrate referrals
        # -------------------------------------------------------------
        if "referrals.json" in namelist:
            refs_data = json.loads(z.read("referrals.json").decode("utf-8"))
            ref_rows = []
            for r in refs_data:
                ref_rows.append((
                    safe_int(r["referrer_id"]),
                    safe_int(r["referred_id"]),
                    r.get("referred_username"),
                    r.get("referred_first_name") or "",
                    r.get("status") or "pending",
                    parse_iso_date(r.get("created_at"))
                ))
            if ref_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO referrals (referrer_id, referred_id, referred_username, referred_first_name, status, created_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                """, ref_rows)
                con.commit()
                print(f"[OK] Migrated {len(ref_rows)} referral records")

        # -------------------------------------------------------------
        # 7. Migrate unique_creation_gods
        # -------------------------------------------------------------
        if "unique_creation_gods.json" in namelist:
            gods_data = json.loads(z.read("unique_creation_gods.json").decode("utf-8"))
            god_rows = []
            for g in gods_data:
                c_at = parse_iso_date(g.get("claimed_at")) if g.get("claimed") else None
                owner_uid = safe_int(g.get("owner_user_id"), None) if g.get("owner_user_id") else None
                pdb_id = safe_int(g.get("pokemon_db_id"), None) if g.get("pokemon_db_id") else None
                god_rows.append((
                    g["god_key"].lower(),
                    g["name"],
                    g.get("title") or "",
                    safe_int(g["species_id"]),
                    safe_int(g.get("origin_species_id") or g["species_id"]),
                    1 if g.get("claimed") else 0,
                    owner_uid,
                    g.get("owner_name"),
                    c_at,
                    pdb_id
                ))
            if god_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO halloween_unique_creation_gods (
                        god_key, name, title, species_id, origin_species_id, claimed, owner_user_id, owner_name, claimed_at, pokemon_db_id
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, god_rows)
                con.commit()
                print(f"[OK] Migrated {len(god_rows)} unique creation gods")

        # -------------------------------------------------------------
        # 8. Migrate active_incubations -> user_eggs
        # -------------------------------------------------------------
        if "active_incubations.json" in namelist:
            eggs_data = json.loads(z.read("active_incubations.json").decode("utf-8"))
            egg_rows = []
            for e in eggs_data:
                uid = safe_int(e["user_id"])
                ename = str(e.get("egg_name") or "")
                pkmn_id = 490 if "manaphy" in ename.lower() else 1
                dur = safe_int(e.get("duration_seconds"), 3600)
                rem = max(1, min(50, dur // 60))
                egg_rows.append((uid, pkmn_id, rem, 1, parse_iso_date(e.get("created_at"))))
            if egg_rows:
                cur.executemany("""
                    INSERT INTO user_eggs (user_id, pokemon_id, steps_remaining, is_incubating, created_at)
                    VALUES (?, ?, ?, ?, ?)
                """, egg_rows)
                con.commit()
                print(f"[OK] Migrated {len(egg_rows)} incubating eggs into user_eggs")

        # -------------------------------------------------------------
        # 9. Migrate gym_configs (custom leader teams, fees, rules, rewards)
        # -------------------------------------------------------------
        if "gym_configs.json" in namelist:
            gyms_data = json.loads(z.read("gym_configs.json").decode("utf-8"))
            gym_rows = []
            for g in gyms_data:
                reg = str(g.get("region") or "").capitalize()
                num = safe_int(g.get("gym_num"), 0)
                is_open = 1 if g.get("is_open") else 0
                leader = str(g.get("leader") or "")
                city = str(g.get("city") or "")
                title = str(g.get("title") or "")
                ptype = str(g.get("type") or "Normal")
                fee = safe_int(g.get("fee") or g.get("challenge_fee"), 200)
                badge = str(g.get("badge") or "")
                quote = str(g.get("dialogue", {}).get("intro") if isinstance(g.get("dialogue"), dict) else "")
                rules_json = json.dumps(g.get("rules") or {})
                rewards_json = json.dumps(g.get("rewards") or {})
                team_json = json.dumps(g.get("team") or [])
                gym_rows.append((
                    reg, num, is_open, leader, city, f"{city} Gym", ptype, fee, quote, badge,
                    team_json, rules_json, rewards_json
                ))
            if gym_rows:
                cur.executemany("""
                    INSERT OR REPLACE INTO gym_configs (
                        region, gym_num, is_open, leader_name, city, gym_name, poke_type, fee, quote, badge,
                        team_json, rules_json, rewards_json
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, gym_rows)
                con.commit()
                print(f"[OK] Migrated {len(gym_rows)} custom gym configs")

    con.close()
    print("\n[SUCCESS] All necessary data migrated successfully into pokemon_bot.db!")

if __name__ == "__main__":
    run_migration()
