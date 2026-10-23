"""Draws the app icon: a white wifi symbol whose dot is a golf ball, on a blue rounded square.

Writes assets/icon.ico (16-256 px, for the exe and installer) and assets/icon.png (256 px,
for the window). Run from the repository root:  python assets/make_icon.py
"""

import math
from pathlib import Path

from PIL import Image, ImageDraw

BLUE_TOP = (0x09, 0x9C, 0xF0)
BLUE_BOTTOM = (0x06, 0x6C, 0xA6)
WHITE = (255, 255, 255, 255)
DIMPLE = (0xC9, 0xD6, 0xE2, 255)
SIZES = [16, 24, 32, 48, 64, 128, 256]
SUPERSAMPLE = 4


def draw(size: int) -> Image.Image:
    s = size * SUPERSAMPLE
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))

    # Blue rounded square with a top-to-bottom gradient.
    gradient = Image.new("RGBA", (s, s))
    gd = ImageDraw.Draw(gradient)
    for y in range(s):
        t = y / (s - 1)
        color = tuple(round(a + (b - a) * t) for a, b in zip(BLUE_TOP, BLUE_BOTTOM))
        gd.line([(0, y), (s, y)], fill=color + (255,))
    mask = Image.new("L", (s, s), 0)
    margin = round(s * 0.03)
    ImageDraw.Draw(mask).rounded_rectangle(
        [margin, margin, s - margin, s - margin], radius=round(s * 0.22), fill=255
    )
    img.paste(gradient, (0, 0), mask)

    d = ImageDraw.Draw(img)
    cx, cy = s * 0.5, s * 0.70
    # Thicker strokes and fewer arcs at small sizes so the icon stays legible.
    small = size <= 24
    stroke = s * (0.10 if small else 0.075)
    radii = [0.30, 0.47] if small else [0.24, 0.36, 0.48]
    span = 45  # degrees either side of straight up
    for r in radii:
        r *= s
        box = [cx - r, cy - r, cx + r, cy + r]
        d.arc(box, 270 - span, 270 + span, fill=WHITE, width=round(stroke))
        # Round caps.
        for angle in (270 - span, 270 + span):
            a = math.radians(angle)
            ex = cx + (r - stroke / 2) * math.cos(a)
            ey = cy + (r - stroke / 2) * math.sin(a)
            d.ellipse([ex - stroke / 2, ey - stroke / 2, ex + stroke / 2, ey + stroke / 2], fill=WHITE)

    # Golf ball, with dimples where there are enough pixels to show them.
    br = s * (0.13 if small else 0.11)
    d.ellipse([cx - br, cy - br, cx + br, cy + br], fill=WHITE)
    if size >= 48:
        ball = Image.new("L", (s, s), 0)
        ImageDraw.Draw(ball).ellipse([cx - br * 0.92, cy - br * 0.92, cx + br * 0.92, cy + br * 0.92], fill=255)
        dimples = Image.new("RGBA", (s, s), (0, 0, 0, 0))
        dd = ImageDraw.Draw(dimples)
        step = br * 0.42
        dr = br * 0.11
        row = 0
        y = cy - br
        while y <= cy + br:
            offset = step / 2 if row % 2 else 0
            x = cx - br + offset
            while x <= cx + br:
                dd.ellipse([x - dr, y - dr, x + dr, y + dr], fill=DIMPLE)
                x += step
            y += step * 0.87
            row += 1
        img.paste(dimples, (0, 0), Image.composite(dimples, Image.new("RGBA", (s, s)), ball).getchannel("A"))

    return img.resize((size, size), Image.LANCZOS)


def main() -> None:
    out = Path(__file__).parent
    images = [draw(size) for size in SIZES]
    images[-1].save(out / "icon.png")
    images[-1].save(out / "icon.ico", sizes=[(n, n) for n in SIZES], append_images=images[:-1])
    print("wrote", out / "icon.ico", "and", out / "icon.png")


if __name__ == "__main__":
    main()
