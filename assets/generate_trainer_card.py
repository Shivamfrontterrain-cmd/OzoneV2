import sys
import os
import argparse
import math
from typing import Dict, Any, Tuple
from PIL import Image, ImageDraw, ImageFont, ImageFilter

CARD_THEMES: Dict[str, Dict[str, Any]] = {
    "Classic Red": {
        "bg_base": (235, 175, 163, 255),
        "border_dark": (152, 36, 26, 255),
        "border_cream": (252, 242, 238, 255),
        "header_color": (152, 36, 26, 255),
        "divider_line": (195, 92, 80, 255),
        "id_box_bg": (212, 148, 136, 255),
        "avatar_box_bg": (250, 238, 232, 255),
        "avatar_box_border": (230, 202, 194, 255),
        "badge_box_bg": (252, 242, 238, 255),
        "flame_dark": (172, 44, 30, 220),
        "flame_mid": (196, 68, 46, 180),
        "arena_rings": [(188, 112, 98, 150), (206, 134, 120, 170), (222, 156, 142, 180), (234, 178, 166, 200)]
    },
    "Ocean Blue": {
        "bg_base": (168, 204, 236, 255),
        "border_dark": (24, 72, 148, 255),
        "border_cream": (238, 246, 254, 255),
        "header_color": (24, 72, 148, 255),
        "divider_line": (90, 138, 196, 255),
        "id_box_bg": (140, 180, 218, 255),
        "avatar_box_bg": (234, 244, 254, 255),
        "avatar_box_border": (190, 215, 240, 255),
        "badge_box_bg": (238, 246, 254, 255),
        "flame_dark": (30, 80, 165, 220),
        "flame_mid": (48, 110, 195, 180),
        "arena_rings": [(102, 148, 196, 150), (125, 170, 215, 170), (150, 192, 230, 180), (175, 210, 242, 200)]
    },
    "Emerald Green": {
        "bg_base": (168, 224, 188, 255),
        "border_dark": (22, 118, 54, 255),
        "border_cream": (238, 252, 242, 255),
        "header_color": (22, 118, 54, 255),
        "divider_line": (88, 178, 118, 255),
        "id_box_bg": (140, 202, 162, 255),
        "avatar_box_bg": (234, 250, 238, 255),
        "avatar_box_border": (190, 230, 205, 255),
        "badge_box_bg": (238, 252, 242, 255),
        "flame_dark": (26, 130, 62, 220),
        "flame_mid": (46, 162, 88, 180),
        "arena_rings": [(105, 180, 128, 150), (128, 198, 148, 170), (152, 212, 170, 180), (176, 226, 192, 200)]
    },
    "Golden Sun": {
        "bg_base": (242, 218, 160, 255),
        "border_dark": (160, 105, 16, 255),
        "border_cream": (254, 250, 236, 255),
        "header_color": (160, 105, 16, 255),
        "divider_line": (205, 160, 80, 255),
        "id_box_bg": (224, 195, 132, 255),
        "avatar_box_bg": (254, 248, 230, 255),
        "avatar_box_border": (235, 220, 185, 255),
        "badge_box_bg": (254, 250, 236, 255),
        "flame_dark": (180, 115, 20, 220),
        "flame_mid": (210, 150, 45, 180),
        "arena_rings": [(198, 165, 100, 150), (215, 184, 120, 170), (228, 200, 140, 180), (240, 215, 162, 200)]
    },
    "Royal Purple": {
        "bg_base": (215, 180, 236, 255),
        "border_dark": (105, 32, 142, 255),
        "border_cream": (250, 240, 254, 255),
        "header_color": (105, 32, 142, 255),
        "divider_line": (168, 108, 198, 255),
        "id_box_bg": (192, 152, 216, 255),
        "avatar_box_bg": (248, 236, 254, 255),
        "avatar_box_border": (224, 198, 238, 255),
        "badge_box_bg": (250, 240, 254, 255),
        "flame_dark": (120, 36, 160, 220),
        "flame_mid": (152, 60, 195, 180),
        "arena_rings": [(168, 125, 196, 150), (188, 148, 212, 170), (205, 168, 225, 180), (220, 188, 238, 200)]
    },
    "Sakura Pink": {
        "bg_base": (244, 192, 212, 255),
        "border_dark": (172, 38, 98, 255),
        "border_cream": (254, 244, 248, 255),
        "header_color": (172, 38, 98, 255),
        "divider_line": (210, 125, 158, 255),
        "id_box_bg": (228, 168, 192, 255),
        "avatar_box_bg": (254, 240, 246, 255),
        "avatar_box_border": (238, 204, 220, 255),
        "badge_box_bg": (254, 244, 248, 255),
        "flame_dark": (185, 45, 110, 220),
        "flame_mid": (215, 80, 145, 180),
        "arena_rings": [(205, 138, 168, 150), (220, 160, 188, 170), (232, 180, 204, 180), (242, 200, 218, 200)]
    },
    "Slate Charcoal": {
        "bg_base": (188, 195, 204, 255),
        "border_dark": (42, 50, 60, 255),
        "border_cream": (244, 247, 250, 255),
        "header_color": (42, 50, 60, 255),
        "divider_line": (120, 132, 146, 255),
        "id_box_bg": (160, 170, 182, 255),
        "avatar_box_bg": (240, 244, 248, 255),
        "avatar_box_border": (205, 214, 224, 255),
        "badge_box_bg": (244, 247, 250, 255),
        "flame_dark": (52, 62, 75, 220),
        "flame_mid": (80, 94, 112, 180),
        "arena_rings": [(135, 146, 158, 150), (155, 166, 178, 170), (175, 185, 196, 180), (195, 204, 212, 200)]
    },
    "Cyan Frost": {
        "bg_base": (172, 228, 236, 255),
        "border_dark": (16, 112, 132, 255),
        "border_cream": (238, 252, 254, 255),
        "header_color": (16, 112, 132, 255),
        "divider_line": (90, 180, 198, 255),
        "id_box_bg": (144, 206, 216, 255),
        "avatar_box_bg": (234, 250, 254, 255),
        "avatar_box_border": (194, 232, 238, 255),
        "badge_box_bg": (238, 252, 254, 255),
        "flame_dark": (22, 126, 148, 220),
        "flame_mid": (48, 168, 192, 180),
        "arena_rings": [(110, 182, 195, 150), (132, 200, 212, 170), (155, 216, 226, 180), (178, 230, 238, 200)]
    },
    "Midnight Indigo": {
        "bg_base": (180, 186, 232, 255),
        "border_dark": (38, 44, 122, 255),
        "border_cream": (242, 244, 254, 255),
        "header_color": (38, 44, 122, 255),
        "divider_line": (110, 120, 192, 255),
        "id_box_bg": (152, 160, 212, 255),
        "avatar_box_bg": (238, 242, 254, 255),
        "avatar_box_border": (200, 208, 238, 255),
        "badge_box_bg": (242, 244, 254, 255),
        "flame_dark": (46, 54, 142, 220),
        "flame_mid": (74, 86, 180, 180),
        "arena_rings": [(125, 134, 195, 150), (148, 156, 212, 170), (168, 176, 226, 180), (188, 195, 238, 200)]
    },
    "Bronze Copper": {
        "bg_base": (238, 196, 162, 255),
        "border_dark": (158, 68, 22, 255),
        "border_cream": (254, 246, 238, 255),
        "header_color": (158, 68, 22, 255),
        "divider_line": (198, 125, 78, 255),
        "id_box_bg": (218, 170, 132, 255),
        "avatar_box_bg": (254, 242, 232, 255),
        "avatar_box_border": (234, 210, 192, 255),
        "badge_box_bg": (254, 246, 238, 255),
        "flame_dark": (174, 76, 26, 220),
        "flame_mid": (204, 108, 52, 180),
        "arena_rings": [(195, 142, 102, 150), (212, 164, 126, 170), (226, 182, 148, 180), (238, 200, 170, 200)]
    }
}

CARD_THEME_NAMES = [
    "Classic Red",
    "Ocean Blue",
    "Emerald Green",
    "Golden Sun",
    "Royal Purple",
    "Sakura Pink",
    "Slate Charcoal",
    "Cyan Frost",
    "Midnight Indigo",
    "Bronze Copper"
]

CARD_TEXT_COLORS: Dict[str, Tuple[int, int, int, int]] = {
    "Classic Dark": (16, 16, 16, 255),
    "Crisp White": (255, 255, 255, 255),
    "Deep Crimson": (152, 36, 26, 255),
    "Navy Indigo": (24, 48, 108, 255),
    "Forest Pine": (18, 72, 36, 255),
    "Royal Plum": (78, 24, 98, 255),
    "Chocolate Brown": (92, 46, 20, 255),
    "Slate Charcoal": (45, 52, 60, 255),
    "Teal Marine": (16, 76, 88, 255),
    "Warm Amber": (132, 68, 12, 255),
    "Ruby Red": (204, 32, 48, 255),
    "Sapphire Blue": (30, 85, 185, 255),
    "Emerald Jade": (34, 139, 74, 255),
    "Amethyst Violet": (128, 48, 160, 255),
    "Sunset Orange": (215, 85, 20, 255),
    "Golden Yellow": (185, 140, 10, 255),
    "Rose Quartz": (195, 60, 120, 255),
    "Sky Cyan": (20, 130, 175, 255),
    "Steel Gray": (70, 80, 95, 255),
    "Bronze Gold": (140, 95, 40, 255)
}

CARD_TEXT_COLOR_NAMES = list(CARD_TEXT_COLORS.keys())

AVATAR_LIST = [
    "red.png",
    "leaf.png",
    "blue.png",
    "brock.png",
    "misty.png",
    "sabrina.png",
    "ethan.png",
    "lyra.png",
    "lance.png",
    "brendan.png",
    "may.png",
    "steven.png",
    "lucas.png",
    "dawn.png",
    "cynthia.png",
    "volkner.png",
    "hilbert.png",
    "hilda.png",
    "nate.png",
    "rosa.png",
    "calem.png",
    "serena.png",
    "leon.png",
    "fisherman.png",
    "trainer_boy.png",
    "trainer_girl.png",
    "drake_gen3.png",
    "captain.png",
    "ash.png",
    "champion.png"
]

def _resolve_theme(theme_input: str) -> str:
    s = str(theme_input).strip()
    if s.isdigit():
        idx = int(s) - 1
        if 0 <= idx < len(CARD_THEME_NAMES):
            return CARD_THEME_NAMES[idx]
    if s in CARD_THEMES:
        return s
    for name in CARD_THEME_NAMES:
        if name.lower() == s.lower():
            return name
    return "Classic Red"

def _resolve_text_color(color_input: str) -> Tuple[int, int, int, int]:
    s = str(color_input).strip()
    if s.isdigit():
        idx = int(s) - 1
        if 0 <= idx < len(CARD_TEXT_COLOR_NAMES):
            return CARD_TEXT_COLORS[CARD_TEXT_COLOR_NAMES[idx]]
    if s in CARD_TEXT_COLORS:
        return CARD_TEXT_COLORS[s]
    for name, col in CARD_TEXT_COLORS.items():
        if name.lower() == s.lower():
            return col
    return CARD_TEXT_COLORS["Classic Dark"]

def _resolve_avatar(avatar_input: str) -> str:
    s = str(avatar_input).strip()
    if s.isdigit():
        idx = int(s) - 1
        if 0 <= idx < len(AVATAR_LIST):
            p = os.path.join("assets", "avatars", AVATAR_LIST[idx])
            if os.path.exists(p):
                return p
    candidates = [
        avatar_input,
        os.path.join("assets", "avatars", avatar_input),
        os.path.join("assets", "avatars", f"{avatar_input}.png"),
        os.path.join("assets", "avatars", avatar_input.lower().replace(" ", "_") + ".png"),
        os.path.join("assets", "avatars", "captain.png"),
        os.path.join("assets", "avatars", "brendan.png"),
        os.path.join("assets", "avatars", "trainer_boy.png"),
        os.path.join("assets", "avatars", "drake_gen3.png"),
    ]
    for c in candidates:
        if c and os.path.exists(c):
            return c
    return ""

def _draw_card_flames(draw: ImageDraw.ImageDraw, flame_dark: tuple, flame_mid: tuple, width: int = 256, height: int = 168):
    """Draw authentic fiery stadium flames across the upper half at 256x168."""
    draw.rectangle([6, 25, width - 6, 68], fill=(flame_dark[0], flame_dark[1], flame_dark[2], 90))

    p1 = [(6, 68)]
    for x in range(8, width - 6, 6):
        h = math.sin(x * 0.25) * 6 + math.cos(x * 0.1) * 5
        py = 28 + int(h) + (x % 3) * 2
        p1.extend([(x - 2, py + 4), (x, py), (x + 2, py + 5)])
    p1.append((width - 6, 68))
    draw.polygon(p1, fill=flame_dark)

    p2 = [(6, 68)]
    for x in range(9, width - 6, 7):
        h = math.cos(x * 0.28) * 7 + math.sin(x * 0.12) * 4
        py = 32 + int(h)
        p2.extend([(x - 2, py + 3), (x, py), (x + 2, py + 4)])
    p2.append((width - 6, 68))
    draw.polygon(p2, fill=flame_mid)

def _draw_card_arena(draw: ImageDraw.ImageDraw, arena_rings: list, width: int = 256, height: int = 168):
    """Draw the concentric stadium arena ring in the center at 256x168."""
    cx, cy = 115, 96
    draw.ellipse([cx - 90, cy - 38, cx + 90, cy + 38], fill=arena_rings[0])
    draw.ellipse([cx - 72, cy - 28, cx + 72, cy + 28], fill=arena_rings[1])
    draw.ellipse([cx - 52, cy - 19, cx + 52, cy + 19], fill=arena_rings[2])
    draw.ellipse([cx - 30, cy - 10, cx + 30, cy + 10], fill=arena_rings[3])

def generate_card(
    user_id: int,
    rank: int = 0,
    exp_points: int = 0,
    to_next_rank: int = 100000,
    wins: int = 0,
    losses: int = 0,
    adventure_started: str = "Oct 06, 2026",
    avatar_path: str = "assets/avatars/brendan.png",
    badges_count: int = 0,
    theme_name: str = "Classic Red",
    text_color_name: str = "Classic Dark",
    output_path: str = "assets/trainer_cards/test_card.png"
):
    # Authentic GBA 256x168 resolution
    width, height = 256, 168
    
    canonical_theme = _resolve_theme(theme_name)
    t_cfg = CARD_THEMES.get(canonical_theme, CARD_THEMES["Classic Red"])
    bg_base = t_cfg["bg_base"]
    border_dark = t_cfg["border_dark"]
    border_cream = t_cfg["border_cream"]
    header_color = t_cfg["header_color"]
    divider_line = t_cfg["divider_line"]
    id_box_bg = t_cfg["id_box_bg"]
    avatar_box_bg = t_cfg["avatar_box_bg"]
    avatar_box_border = t_cfg["avatar_box_border"]
    badge_box_bg = t_cfg["badge_box_bg"]
    flame_dark = t_cfg["flame_dark"]
    flame_mid = t_cfg["flame_mid"]
    arena_rings = t_cfg["arena_rings"]

    text_color = _resolve_text_color(text_color_name)

    base_img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(base_img)

    margin = 3
    # Outer frame
    draw.rounded_rectangle([margin, margin, width - margin, height - margin], radius=8, fill=bg_base, outline=border_dark, width=2)
    draw.rounded_rectangle([margin + 2, margin + 2, width - margin - 2, height - margin - 2], radius=6, fill=None, outline=border_cream, width=1)

    # Arena flames & stadium floor watermark
    flame_layer = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    flame_draw = ImageDraw.Draw(flame_layer)
    _draw_card_flames(flame_draw, flame_dark, flame_mid, width, height)
    _draw_card_arena(flame_draw, arena_rings, width, height)
    flame_layer = flame_layer.filter(ImageFilter.GaussianBlur(0.6))
    base_img.paste(flame_layer, (0, 0), flame_layer)

    # Redraw frame cleanly over background texture
    draw.rounded_rectangle([margin, margin, width - margin, height - margin], radius=8, fill=None, outline=border_dark, width=2)
    draw.rounded_rectangle([margin + 2, margin + 2, width - margin - 2, height - margin - 2], radius=6, fill=None, outline=border_cream, width=1)

    # Fonts: Standard pixel bitmap font matching the authentic GBA retro look with 100% crisp, clean rendering
    font_header_title = font_header_rank = font_id = font_body = font_badge = ImageFont.load_default()

    # 1. Top Header Line (Y: 9 to 24)
    draw.text((12, 10), "TRAINER CARD", fill=header_color, font=font_header_title)
    rank_str = f"Rank  {rank}"
    draw.text((172, 8), rank_str, fill=text_color, font=font_header_rank)
    draw.line([(8, 25), (width - 8, 25)], fill=divider_line, width=1)

    # 2. ID Number Container Box (Top-Right: X: 132 to 245, Y: 28 to 43)
    draw.rounded_rectangle([132, 28, 245, 43], radius=3, fill=id_box_bg)
    draw.text((137, 29), "IDNo.", fill=text_color, font=font_id)
    draw.text((176, 29), f"{user_id}", fill=text_color, font=font_id)

    # 3. Stats Rows (Left side rows)
    exp_next_str = "MAX RANK" if rank >= 10000000 else f"{to_next_rank:,}"
    stats_left = [
        ("Exp. Points", f"{exp_points:,}"),
        ("To Next Rank", exp_next_str),
        ("Wins", f"{wins:,}"),
        ("Losses", f"{losses:,}")
    ]

    sy = 46
    gap = 18
    for i, (lbl, val) in enumerate(stats_left):
        y = sy + (i * gap)
        draw.text((12, y), lbl, fill=text_color, font=font_body)
        vw = font_body.getbbox(val)[2] - font_body.getbbox(val)[0]
        draw.text((166 - vw, y), val, fill=text_color, font=font_body)

    # 4. Avatar Container Box (Right side: X: 174 to 244, Y: 46 to 114)
    av_rect = [174, 46, 244, 114]
    draw.rounded_rectangle(av_rect, radius=6, fill=avatar_box_bg, outline=avatar_box_border, width=1)

    actual_avatar = _resolve_avatar(avatar_path)
    if actual_avatar:
        try:
            with Image.open(actual_avatar) as av_img:
                av_img = av_img.convert("RGBA")
                bbox = av_img.getbbox()
                if bbox:
                    av_img = av_img.crop(bbox)
                max_w, max_h = 58, 62
                scale = min(max_w / av_img.width, max_h / av_img.height)
                nw, nh = max(1, int(av_img.width * scale)), max(1, int(av_img.height * scale))
                av_scaled = av_img.resize((nw, nh), Image.Resampling.NEAREST)
                px = av_rect[0] + (av_rect[2] - av_rect[0] - nw) // 2
                py = av_rect[1] + (av_rect[3] - av_rect[1] - nh) // 2
                base_img.paste(av_scaled, (px, py), av_scaled)
        except Exception as e:
            print(f"Error loading avatar: {e}", file=sys.stderr)

    # 5. Adventure started date row (Below avatar & stats, aligned with stats column)
    adv_y = 118
    draw.text((12, adv_y), "Adventure started", fill=text_color, font=font_body)
    vw_date = font_body.getbbox(adventure_started)[2] - font_body.getbbox(adventure_started)[0]
    vw_adv = font_body.getbbox("Adventure started")[2] - font_body.getbbox("Adventure started")[0]
    date_x = 166 - vw_date
    if date_x < 12 + vw_adv + 4:
        date_x = 12 + vw_adv + 6
    draw.text((date_x, adv_y), adventure_started, fill=text_color, font=font_body)

    # 6. Badges Bar at Bottom (Y: 134 to 158)
    badge_rect = [8, 134, width - 8, 158]
    draw.rounded_rectangle(badge_rect, radius=3, fill=badge_box_bg, outline=avatar_box_border, width=1)
    draw.text((12, 138), "Badges", fill=header_color, font=font_badge)

    badge_names = ['boulder', 'cascade', 'thunder', 'rainbow', 'soul', 'marsh', 'volcano', 'earth']
    bx_start = 74
    b_spacing = 21
    by_center = 146
    badge_size = 14

    effective_badges = max(0, min(8, int(badges_count if badges_count is not None else 0)))

    for bi, bname in enumerate(badge_names):
        bx = bx_start + (bi * b_spacing)
        is_unlocked = bi < effective_badges
        b_path = os.path.join("assets", "badges", f"{bname}.png")
        if os.path.exists(b_path):
            try:
                with Image.open(b_path) as b_raw:
                    b_raw = b_raw.convert("RGBA")
                    scale = min(badge_size / b_raw.width, badge_size / b_raw.height)
                    bw, bh = max(1, int(b_raw.width * scale)), max(1, int(b_raw.height * scale))
                    b_img = b_raw.resize((bw, bh), Image.Resampling.LANCZOS)
                    pos_x = bx - bw // 2
                    pos_y = by_center - bh // 2
                    if is_unlocked:
                        base_img.paste(b_img, (pos_x, pos_y), b_img)
                    else:
                        r_ch, g_ch, b_ch, a_ch = b_img.split()
                        gray_img = Image.merge("RGBA", (
                            Image.new("L", b_img.size, 175),
                            Image.new("L", b_img.size, 175),
                            Image.new("L", b_img.size, 175),
                            a_ch.point(lambda p: int(p * 0.22))
                        ))
                        base_img.paste(gray_img, (pos_x, pos_y), gray_img)
            except Exception:
                pass

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    # Save as RGB to eliminate any transparency or white borders in Telegram
    base_img.convert("RGB").save(output_path, "PNG", optimize=True)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--user-id", type=int, default=8404111906)
    parser.add_argument("--rank", type=int, default=0)
    parser.add_argument("--exp", type=int, default=0)
    parser.add_argument("--to-next", type=int, default=100000)
    parser.add_argument("--wins", type=int, default=0)
    parser.add_argument("--losses", type=int, default=0)
    parser.add_argument("--date", type=str, default="Oct 07, 2026")
    parser.add_argument("--avatar", type=str, default="assets/avatars/brendan.png")
    parser.add_argument("--badges", type=int, default=0)
    parser.add_argument("--theme", type=str, default="Classic Red")
    parser.add_argument("--text-color", type=str, default="Classic Dark")
    parser.add_argument("--output", type=str, default="assets/trainer_cards/test_card.png")
    args = parser.parse_args()

    generate_card(
        user_id=args.user_id,
        rank=args.rank,
        exp_points=args.exp,
        to_next_rank=args.to_next,
        wins=args.wins,
        losses=args.losses,
        adventure_started=args.date,
        avatar_path=args.avatar,
        badges_count=args.badges,
        theme_name=args.theme,
        text_color_name=args.text_color,
        output_path=args.output
    )
