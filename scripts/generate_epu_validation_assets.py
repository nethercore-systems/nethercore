#!/usr/bin/env python3
"""Generate deterministic cubemap-face PNGs for EPU texture validation."""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT_ROOT = ROOT / "examples" / "6-assets" / "epu-textures-demo" / "epu_faces"
SIZE = 256
FACES = ("px", "nx", "py", "ny", "pz", "nz")

FACE_BASE_COLORS = {
    "px": (232, 92, 84),
    "nx": (72, 220, 124),
    "py": (92, 152, 255),
    "ny": (240, 194, 74),
    "pz": (222, 108, 248),
    "nz": (78, 226, 244),
}

# Runtime/import convention from the shader:
#   +X: dir = normalize( 1, -v, -u)
#   -X: dir = normalize(-1, -v,  u)
#   +Y: dir = normalize( u,  1,  v)
#   -Y: dir = normalize( u, -1, -v)
#   +Z: dir = normalize( u, -v,  1)
#   -Z: dir = normalize(-u, -v, -1)
FACE_IMAGE_BASES = {
    "px": ((0.0, 0.0, -1.0), (0.0, -1.0, 0.0)),
    "nx": ((0.0, 0.0, 1.0), (0.0, -1.0, 0.0)),
    "py": ((1.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
    "ny": ((1.0, 0.0, 0.0), (0.0, 0.0, -1.0)),
    "pz": ((1.0, 0.0, 0.0), (0.0, -1.0, 0.0)),
    "nz": ((-1.0, 0.0, 0.0), (0.0, -1.0, 0.0)),
}

AXIS_COLORS = {
    "x": (255, 88, 88),
    "y": (96, 240, 112),
    "z": (92, 196, 255),
}


def png_chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    rows = []
    stride = width * 4
    for y in range(height):
        rows.append(b"\x00" + rgba[y * stride : (y + 1) * stride])
    payload = b"".join(rows)
    png = b"\x89PNG\r\n\x1a\n"
    png += png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    png += png_chunk(b"IDAT", zlib.compress(payload, level=9))
    png += png_chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def clamp(x: float) -> int:
    return max(0, min(255, int(round(x))))


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def mix(c0, c1, t):
    return tuple(clamp(lerp(c0[i], c1[i], t)) for i in range(3))


def over(base, color, alpha):
    alpha = max(0.0, min(1.0, alpha))
    return tuple(clamp(base[i] * (1.0 - alpha) + color[i] * alpha) for i in range(3))


def add_glow(base, center_x, center_y, x, y, color, radius, power=2.0):
    dx = x - center_x
    dy = y - center_y
    dist = math.sqrt(dx * dx + dy * dy)
    if dist >= radius:
        return base
    t = (1.0 - dist / radius) ** power
    return tuple(clamp(base[i] + color[i] * t) for i in range(3))


def rect_alpha(u, v, x0, y0, x1, y1, feather=0.0):
    if feather <= 0.0:
        return 1.0 if x0 <= u <= x1 and y0 <= v <= y1 else 0.0
    ax = min((u - x0) / feather, (x1 - u) / feather)
    ay = min((v - y0) / feather, (y1 - v) / feather)
    return max(0.0, min(1.0, min(ax, ay)))


def distance_to_segment(px, py, ax, ay, bx, by):
    abx = bx - ax
    aby = by - ay
    apx = px - ax
    apy = py - ay
    denom = abx * abx + aby * aby
    if denom <= 1e-8:
        return math.sqrt((px - ax) ** 2 + (py - ay) ** 2)
    t = max(0.0, min(1.0, (apx * abx + apy * aby) / denom))
    qx = ax + abx * t
    qy = ay + aby * t
    return math.sqrt((px - qx) ** 2 + (py - qy) ** 2)


def line_alpha(px, py, ax, ay, bx, by, width):
    dist = distance_to_segment(px, py, ax, ay, bx, by)
    return max(0.0, min(1.0, 1.0 - dist / max(width, 1e-5)))


def point_in_triangle(px, py, ax, ay, bx, by, cx, cy):
    v0x = cx - ax
    v0y = cy - ay
    v1x = bx - ax
    v1y = by - ay
    v2x = px - ax
    v2y = py - ay
    dot00 = v0x * v0x + v0y * v0y
    dot01 = v0x * v1x + v0y * v1y
    dot02 = v0x * v2x + v0y * v2y
    dot11 = v1x * v1x + v1y * v1y
    dot12 = v1x * v2x + v1y * v2y
    inv = dot00 * dot11 - dot01 * dot01
    if abs(inv) <= 1e-8:
        return False
    inv = 1.0 / inv
    uu = (dot11 * dot02 - dot01 * dot12) * inv
    vv = (dot00 * dot12 - dot01 * dot02) * inv
    return uu >= 0.0 and vv >= 0.0 and uu + vv <= 1.0


def arrow_alpha(u, v, start, end, shaft_width=0.018, head_size=0.05):
    ax, ay = start
    bx, by = end
    dx = bx - ax
    dy = by - ay
    length = math.sqrt(dx * dx + dy * dy)
    if length <= 1e-5:
        return 0.0
    nx = dx / length
    ny = dy / length
    shaft_end = (bx - nx * head_size, by - ny * head_size)
    shaft = line_alpha(u, v, ax, ay, shaft_end[0], shaft_end[1], shaft_width)
    px = -ny
    py = nx
    head_half = head_size * 0.45
    head = point_in_triangle(
        u,
        v,
        bx,
        by,
        shaft_end[0] + px * head_half,
        shaft_end[1] + py * head_half,
        shaft_end[0] - px * head_half,
        shaft_end[1] - py * head_half,
    )
    return max(shaft, 1.0 if head else 0.0)


def corner_marker_alpha(u, v, cx, cy, size):
    dx = abs(u - cx)
    dy = abs(v - cy)
    return 1.0 if dx <= size and dy <= size else 0.0


def circle_alpha(u, v, cx, cy, radius, feather=0.0):
    dist = math.sqrt((u - cx) ** 2 + (v - cy) ** 2)
    if feather <= 0.0:
        return 1.0 if dist <= radius else 0.0
    return max(0.0, min(1.0, 1.0 - (dist - radius) / feather))


def ring_alpha(u, v, cx, cy, inner_radius, outer_radius, feather=0.0):
    outer = circle_alpha(u, v, cx, cy, outer_radius, feather)
    inner = circle_alpha(u, v, cx, cy, inner_radius, feather)
    return max(outer - inner, 0.0)


def periodic_line_alpha(coord, center, width):
    dist = abs(coord - center)
    return max(0.0, min(1.0, 1.0 - dist / max(width, 1e-5)))


def normalize3(x: float, y: float, z: float):
    length = math.sqrt(x * x + y * y + z * z)
    if length <= 1e-8:
        return (0.0, 0.0, 0.0)
    return (x / length, y / length, z / length)


def dot3(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def face_uv_to_dir(face: str, u: float, v: float):
    sx = u * 2.0 - 1.0
    sy = v * 2.0 - 1.0
    if face == "px":
        return normalize3(1.0, -sy, -sx)
    if face == "nx":
        return normalize3(-1.0, -sy, sx)
    if face == "py":
        return normalize3(sx, 1.0, sy)
    if face == "ny":
        return normalize3(sx, -1.0, -sy)
    if face == "pz":
        return normalize3(sx, -sy, 1.0)
    return normalize3(-sx, -sy, -1.0)


def project_world_axis_to_face(face: str, world_axis):
    right, down = FACE_IMAGE_BASES[face]
    px = dot3(world_axis, right)
    py = dot3(world_axis, down)
    return (px, py)


def draw_glyph(base, glyph: str, cx: float, cy: float, scale: float, color, u: float, v: float):
    stroke = max(scale * 0.14, 0.006)
    segments = []

    if glyph == "+":
        segments = [
            ((cx - scale * 0.30, cy), (cx + scale * 0.30, cy)),
            ((cx, cy - scale * 0.30), (cx, cy + scale * 0.30)),
        ]
    elif glyph == "-":
        segments = [((cx - scale * 0.30, cy), (cx + scale * 0.30, cy))]
    elif glyph == "X":
        segments = [
            ((cx - scale * 0.35, cy - scale * 0.35), (cx + scale * 0.35, cy + scale * 0.35)),
            ((cx - scale * 0.35, cy + scale * 0.35), (cx + scale * 0.35, cy - scale * 0.35)),
        ]
    elif glyph == "Y":
        segments = [
            ((cx - scale * 0.34, cy - scale * 0.34), (cx, cy)),
            ((cx + scale * 0.34, cy - scale * 0.34), (cx, cy)),
            ((cx, cy), (cx, cy + scale * 0.38)),
        ]
    elif glyph == "Z":
        segments = [
            ((cx - scale * 0.34, cy - scale * 0.34), (cx + scale * 0.34, cy - scale * 0.34)),
            ((cx + scale * 0.34, cy - scale * 0.34), (cx - scale * 0.34, cy + scale * 0.34)),
            ((cx - scale * 0.34, cy + scale * 0.34), (cx + scale * 0.34, cy + scale * 0.34)),
        ]

    alpha = 0.0
    for (ax, ay), (bx, by) in segments:
        alpha = max(alpha, line_alpha(u, v, ax, ay, bx, by, stroke))
    return over(base, color, alpha)


def draw_label(base, text: str, cx: float, cy: float, scale: float, color, u: float, v: float):
    count = len(text)
    spacing = scale * 0.95
    start_x = cx - spacing * (count - 1) * 0.5
    for i, glyph in enumerate(text):
        base = draw_glyph(base, glyph, start_x + i * spacing, cy, scale, color, u, v)
    return base


def apply_simple_overlay(base, face: str, x: int, y: int, label_bg, label_fg):
    u = x / (SIZE - 1)
    v = y / (SIZE - 1)
    label = {
        "px": "+X",
        "nx": "-X",
        "py": "+Y",
        "ny": "-Y",
        "pz": "+Z",
        "nz": "-Z",
    }[face]

    # Clear face label in the middle.
    base = over(base, label_bg, circle_alpha(u, v, 0.5, 0.5, 0.19, 0.02) * 0.94)
    base = over(base, (255, 255, 255), ring_alpha(u, v, 0.5, 0.5, 0.12, 0.19, 0.012) * 0.7)
    base = draw_label(base, label, 0.5, 0.5, 0.09, label_fg, u, v)

    # Single bottom-left orientation gizmo: U right, V up.
    panel = rect_alpha(u, v, 0.05, 0.70, 0.32, 0.95, 0.02)
    base = over(base, (18, 18, 24), panel * 0.30)
    base = over(base, (250, 72, 56), arrow_alpha(u, v, (0.09, 0.90), (0.28, 0.90), 0.012, 0.035))
    base = over(base, (72, 236, 96), arrow_alpha(u, v, (0.09, 0.90), (0.09, 0.74), 0.012, 0.035))
    return base


def axis_room(face: str, x: int, y: int):
    u = x / (SIZE - 1)
    v = y / (SIZE - 1)
    base_color = FACE_BASE_COLORS[face]
    top = tuple(clamp(component * 1.06) for component in base_color)
    bottom = tuple(clamp(component * 0.82) for component in base_color)
    base = mix(top, bottom, v)
    base = apply_simple_overlay(base, face, x, y, (16, 16, 24), (255, 255, 255))
    return (*base, 255)


def studio_warm(face: str, x: int, y: int):
    u = x / (SIZE - 1)
    v = y / (SIZE - 1)
    top = {
        "px": (255, 220, 176),
        "nx": (255, 208, 164),
        "py": (248, 244, 228),
        "ny": (118, 88, 68),
        "pz": (255, 210, 156),
        "nz": (255, 224, 182),
    }[face]
    bottom = {
        "px": (64, 34, 24),
        "nx": (56, 32, 26),
        "py": (204, 200, 190),
        "ny": (86, 54, 40),
        "pz": (64, 32, 26),
        "nz": (52, 28, 24),
    }[face]
    base = mix(top, bottom, v)

    base = apply_simple_overlay(base, face, x, y, (28, 20, 12), (255, 248, 212))
    return (*base, 255)


def neon_night(face: str, x: int, y: int):
    u = x / (SIZE - 1)
    v = y / (SIZE - 1)
    bg0 = {
        "px": (10, 10, 20),
        "nx": (12, 8, 22),
        "py": (8, 10, 20),
        "ny": (4, 4, 8),
        "pz": (12, 8, 20),
        "nz": (8, 10, 22),
    }[face]
    bg1 = {
        "px": (28, 18, 48),
        "nx": (18, 26, 52),
        "py": (22, 20, 54),
        "ny": (10, 8, 18),
        "pz": (20, 20, 50),
        "nz": (28, 18, 52),
    }[face]
    base = mix(bg0, bg1, v)

    base = apply_simple_overlay(base, face, x, y, (228, 236, 255), (24, 24, 36))
    return (*base, 255)


GENERATORS = {
    "axis_room": axis_room,
    "studio_warm": studio_warm,
    "neon_night": neon_night,
}


def build_face(fn, face: str) -> bytes:
    out = bytearray(SIZE * SIZE * 4)
    for y in range(SIZE):
        for x in range(SIZE):
            rgba = fn(face, x, y)
            i = (y * SIZE + x) * 4
            out[i : i + 4] = bytes(rgba)
    return bytes(out)


def main() -> None:
    for env_name, fn in GENERATORS.items():
        for face in FACES:
            rgba = build_face(fn, face)
            write_png(OUT_ROOT / env_name / f"{face}.png", SIZE, SIZE, rgba)
    print(f"Generated validation cubemap faces under {OUT_ROOT}")


if __name__ == "__main__":
    main()
