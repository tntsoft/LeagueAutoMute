"""Composite a red 'muted' badge over the League logo and write icon.ico.

Usage: python assets/make_icon.py
Reads  assets/league.png (256x256 RGBA), writes assets/icon.ico and assets/icon-preview.png.
"""
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).parent
SRC = HERE / "league.png"
ICO = HERE / "icon.ico"
PREVIEW = HERE / "icon-preview.png"

RED = (211, 47, 47, 255)
WHITE = (255, 255, 255, 255)


def draw_badge(base: Image.Image, scale: float) -> Image.Image:
    """Return a copy of `base` with a muted-speaker badge in the bottom-right.

    `scale` grows the badge for small sizes so it stays readable at 16 px.
    Drawn at 4x and downsampled for smooth edges.
    """
    s = 4
    w, h = base.size
    big = base.resize((w * s, h * s), Image.LANCZOS)
    d = ImageDraw.Draw(big)

    r = int(w * 0.30 * scale) * s          # badge radius
    cx, cy = w * s - r - 2 * s, h * s - r - 2 * s
    ring = 3 * s
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=WHITE)
    d.ellipse([cx - r + ring, cy - r + ring, cx + r - ring, cy + r - ring], fill=RED)

    # Speaker glyph inside the badge, sized relative to the badge radius.
    g = r * 0.62                                     # glyph half-size
    bx0, by0 = cx - g * 0.95, cy - g * 0.40          # body rect
    bx1, by1 = cx - g * 0.35, cy + g * 0.40
    d.rectangle([bx0, by0, bx1, by1], fill=WHITE)
    cone = [(bx1, by0), (cx + g * 0.35, cy - g * 0.95),
            (cx + g * 0.35, cy + g * 0.95), (bx1, by1)]
    d.polygon(cone, fill=WHITE)

    # Diagonal slash: red underlay (gap) then white stroke.
    p0 = (cx - g * 0.9, cy - g * 0.9)
    p1 = (cx + g * 0.9, cy + g * 0.9)
    d.line([p0, p1], fill=RED, width=int(g * 0.55))
    d.line([p0, p1], fill=WHITE, width=int(g * 0.28))

    return big.resize((w, h), Image.LANCZOS)


def main() -> None:
    base = Image.open(SRC).convert("RGBA")
    assert base.size == (256, 256), base.size

    frames = []
    for size in (256, 128, 64, 48, 32, 24, 16):
        # Bigger badge for small icons so the red mark survives downscaling.
        scale = 1.0 if size >= 48 else 1.25
        img = draw_badge(base, scale).resize((size, size), Image.LANCZOS)
        frames.append(img)

    # Pillow writes every frame as its own ICO entry when given append_images.
    frames[0].save(ICO, format="ICO", sizes=[f.size for f in frames],
                   append_images=frames[1:])

    # Side-by-side preview of each size, scaled up so it is easy to inspect.
    cell = 72
    sheet = Image.new("RGBA", (cell * len(frames), cell), (40, 40, 40, 255))
    for i, f in enumerate(frames):
        up = f.resize((64, 64), Image.NEAREST) if f.size[0] < 64 else f.resize((64, 64), Image.LANCZOS)
        sheet.alpha_composite(up, (i * cell + 4, 4))
    sheet.save(PREVIEW)
    print(f"wrote {ICO} ({ICO.stat().st_size} bytes) and {PREVIEW}")


if __name__ == "__main__":
    main()
