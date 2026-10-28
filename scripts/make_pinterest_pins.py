#!/usr/bin/env python3
"""Generate vertical 1000x1500 Pinterest pin images for PhotonCAM and rewrite the CSV Media URLs."""
import csv, os, textwrap
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "assets", "gumroad")
OUT = os.path.join(ROOT, "assets", "pinterest")
CSV_PATH = os.path.join(ROOT, "pinterest_photoncam_v1.2.5.csv")
RAW = "https://raw.githubusercontent.com/pietini69/photoncam/master/assets/pinterest/"
os.makedirs(OUT, exist_ok=True)

FONT = "/System/Library/Fonts/HelveticaNeue.ttc"
def font(size, bold=False):
    return ImageFont.truetype(FONT, size, index=1 if bold else 0)

SHOTS = ["screenshot_dark.png", "screenshot_dark_and_light.png", "screenshot_light.png", "laserburn_app_screenshot.png"]
W, H = 1000, 1500

def make(i, headline, sub):
    bg = Image.new("RGB", (W, H), (18, 20, 28))
    d = ImageDraw.Draw(bg)
    for y in range(H):  # vertical gradient
        t = y / H
        d.line([(0, y), (W, y)], fill=(int(18 + 40 * t), int(20 + 10 * t), int(28 + 8 * t)))
    d.rectangle([0, 0, W, 14], fill=(255, 90, 40))
    # headline
    y = 70
    for line in textwrap.wrap(headline, 20):
        d.text((60, y), line, font=font(78, True), fill=(255, 255, 255))
        y += 92
    y += 10
    for line in textwrap.wrap(sub, 38):
        d.text((60, y), line, font=font(38), fill=(215, 220, 232))
        y += 50
    # screenshot, cropped to portrait-friendly 4:3 block
    shot = Image.open(os.path.join(SRC, SHOTS[i % len(SHOTS)])).convert("RGB")
    sw = W - 120
    sh = int(shot.height * sw / shot.width)
    shot = shot.resize((sw, sh), Image.LANCZOS)
    sy = max(y + 40, 640)
    shadow = Image.new("RGBA", (sw + 40, sh + 40), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle([20, 20, sw + 20, sh + 20], 24, fill=(0, 0, 0, 170))
    bg.paste(shadow.filter(ImageFilter.GaussianBlur(14)), (40, sy - 10), shadow.filter(ImageFilter.GaussianBlur(14)))
    mask = Image.new("L", (sw, sh), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, sw, sh], 20, fill=255)
    bg.paste(shot, (60, sy), mask)
    # footer
    d = ImageDraw.Draw(bg)
    d.rounded_rectangle([60, H - 190, W - 60, H - 70], 28, fill=(255, 90, 40))
    d.text((W // 2, H - 150), "PhotonCAM — Free Download", font=font(54, True), fill=(255, 255, 255), anchor="mm")
    d.text((W // 2, H - 100), "macOS · Windows · Linux · Raspberry Pi", font=font(32), fill=(255, 235, 225), anchor="mm")
    name = f"pin_{i + 1:02d}.jpg"
    bg.save(os.path.join(OUT, name), quality=90)
    return name

rows = list(csv.reader(open(CSV_PATH, encoding="utf-8")))
hdr, body = rows[0], rows[1:]
mi = hdr.index("Media URL")
for i, r in enumerate(body):
    title = r[hdr.index("Title")].split("|")[0].strip()
    desc = r[hdr.index("Description")].split("!")[0].split("?")[0][:110]
    r[mi] = RAW + make(i, title, "Modern laser CAM built in Rust. LightBurn .lbrn2 compatible.")
csv.writer(open(CSV_PATH, "w", newline="", encoding="utf-8")).writerows([hdr] + body)
print("Generated", len(body), "pins in", OUT)
