#!/usr/bin/env python3
"""Generate GitX architecture diagram"""
from PIL import Image, ImageDraw, ImageFont
import math, os

W, H = 1400, 800
img = Image.new('RGB', (W, H), '#ffffff')
draw = ImageDraw.Draw(img)

# Colors
indigo = '#6366F1'
teal = '#14B8A6'
purple = '#A855F7'
amber = '#F59E0B'
dark = '#111827'
gray = '#6B7280'
light_gray = '#E5E7EB'
card_bg = '#FAFAFA'

# Fonts
try:
    title_f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 36)
    h2_f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 22)
    h3_f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 18)
    body_f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 15)
    small_f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 13)
    mono_f = ImageFont.truetype("/System/Library/Fonts/SF Mono.ttc", 11)
except:
    title_f = h2_f = h3_f = body_f = small_f = mono_f = ImageFont.load_default()

def shadow_card(x, y, w, h, radius=20):
    for i in range(12, 0, -1):
        alpha = int(10 * (12-i)/12)
        c = f'#{alpha:02x}{alpha:02x}{alpha:02x}'
        draw.rounded_rectangle([x+4+i//2, y+4+i//2, x+w+4+i//2, y+h+4+i//2], 
                              radius=radius, fill=c)
    draw.rounded_rectangle([x, y, x+w, y+h], radius=radius, fill=card_bg, outline=light_gray)

# === Title ===
draw.text((W//2, 45), "GitX Architecture", font=title_f, fill=dark, anchor="mm")
draw.line([(W//2-150, 72), (W//2+150, 72)], fill=indigo, width=3)

# === Block 1: Git Repository ===
b1x, b1y = 50, 110
bw, bh = 340, 530
shadow_card(b1x, b1y, bw, bh)
draw.rounded_rectangle([b1x, b1y, bw, b1y+56], 
                        corners=[(20,0),(20,0),(0,0),(0,20)], fill='#F5F3FF')
draw.rectangle([b1x+4, b1y+52, b1x+bw-4, b1y+56], fill=indigo)
draw.text((b1x + bw//2, b1y+30), "Git Repository", font=h2_f, fill=dark, anchor="mm")

# Git tree
tree_cx = b1x + bw//2 - 10
tree_top = b1y + 100

main_x = tree_cx
for i in range(5):
    y = tree_top + 45 + i*60
    if i > 0:
        draw.line([(main_x, tree_top+45+(i-1)*60), (main_x, y)], fill=indigo, width=3)
    draw.ellipse([main_x-8, y-8, main_x+8, y+8], fill=indigo)

dev_x = main_x - 55
for i in range(3):
    y = tree_top + 75 + i*55
    if i > 0:
        draw.line([(dev_x, tree_top+75+(i-1)*55), (dev_x, y)], fill=teal, width=2)
    draw.ellipse([dev_x-7, y-7, dev_x+7, y+7], fill=teal)
draw.line([(main_x, tree_top+105), (dev_x, tree_top+75)], fill=teal, width=2)

feat_x = main_x + 55
for i in range(2):
    y = tree_top + 135 + i*50
    if i > 0:
        draw.line([(feat_x, tree_top+135+(i-1)*50), (feat_x, y)], fill=purple, width=2)
    draw.ellipse([feat_x-7, y-7, feat_x+7, y+7], fill=purple)
draw.line([(main_x, tree_top+165), (feat_x, tree_top+135)], fill=purple, width=2)

# Branch labels
label_y = tree_top + 310
pill_data = [(b1x+25, label_y, "main", indigo),
             (b1x+105, label_y+38, "develop", teal),
             (b1x+205, label_y+76, "feature/auth", purple)]
for px, py, txt, col in pill_data:
    tw = len(txt)*9 + 24
    draw.rounded_rectangle([px, py, px+tw, py+28], radius=14, fill=col)
    draw.text((px+tw//2, py+14), txt, font=body_f, fill='white', anchor="mm")

# Stats
stats_y = label_y + 120
draw.rounded_rectangle([b1x+20, stats_y, b1x+bw-20, stats_y+58], radius=10, fill='#F0FDF4')
draw.text((b1x+40, stats_y+16), "4 branches", font=body_f, fill='#166534')
draw.text((b1x+40, stats_y+38), "9 commits", font=body_f, fill='#166534')

# === Arrow 1 ===
arr_x = b1x + bw + 25
arr_mid = arr_x + 40
arr_y = H//2 + 35
for i in range(36):
    r = int(99*(1-i/36) + 168*i/36)
    g = int(102*(1-i/36) + 85*i/36)
    b_val = int(241*(1-i/36) + 247*i/36)
    draw.line([(arr_x+i, arr_y), (arr_x+i, arr_y+2)], fill=f'#{r:02x}{g:02x}{b_val:02x}')
draw.polygon([(arr_x+38, arr_y-12), (arr_x+56, arr_y), (arr_x+38, arr_y+12)], fill=purple)

# === Block 2: GitX App Window ===
b2x = b1x + bw + 90
b2w, b2h = 430, 530
shadow_card(b2x, b1y, b2w, b2h)

# Title bar
draw.rounded_rectangle([b2x, b1y, b2x+b2w, b1y+48],
                        corners=[(20,0),(20,0),(0,0),(0,20)], fill='#1e1e2e')
draw.ellipse([b2x+18, b1y+16, b2x+32, b1y+30], fill='#FF5F57')
draw.ellipse([b2x+40, b1y+16, b2x+54, b1y+30], fill='#FEBC2E')
draw.ellipse([b2x+62, b1y+16, b2x+76, b1y+30], fill='#28C840')
draw.text((b2x+b2w//2, b1y+26), "GitX — AI Diff Analyzer", font=h3_f, fill='#cdd6f4', anchor="mm")

app_y = b1y + 56
app_h = b2h - 68
gap, pad = 6, 10

# Sidebar
sw = 95
draw.rounded_rectangle([b2x+pad, app_y, b2x+pad+sw, app_y+app_h], radius=10, fill='#181825')
draw.text((b2x+pad+sw//2, app_y+18), "Repos", font=small_f, fill=gray, anchor="mm")
items = ['myproject', 'frontend', 'backend']
iy = app_y+42
for item in items:
    bg = '#313244' if item=='myproject' else '#1e1e2e'
    fg = '#cdd6f4' if item=='myproject' else gray
    draw.rounded_rectangle([b2x+pad+6, iy, b2x+pad+sw-6, iy+26], radius=5, fill=bg)
    draw.text((b2x+pad+14, iy+13), item, font=small_f, fill=fg, anchor="lm")
    iy += 32
iy += 6
draw.text((b2x+pad+sw//2, iy), "Branches", font=small_f, fill=gray, anchor="mm")
iy += 24
for txt, col in [('main', indigo), ('develop', gray), ('feature', gray)]:
    draw.text((b2x+pad+14, iy), f"  {txt}", font=mono_f, fill=col)
    iy += 22

# Diff panel
dx = b2x+pad+sw+gap
dw = b2w-sw-pad*2-gap-96-gap
draw.rounded_rectangle([dx, app_y, dx+dw, app_y+app_h], radius=10, fill='#1e1e2e')
tab_w = 85
draw.rounded_rectangle([dx+4, app_y+4, dx+4+tab_w, app_y+30], radius=6, fill='#313244')
draw.text((dx+4+tab_w//2, app_y+18), "ai_client.go", font=mono_f, fill='#cdd6f4', anchor="mm")

dy = app_y+42
diffs = [
    ("+", "#23D18B", "+func NewClient(c Config) (*Client, error) {"),
    (" ", "", "     client := &Client{config: c}"),
    ("+", "#23D18B", "+    client.api = openai.New()"),
    ("-", "#F38BA8", "-    return nil, nil"),
    (" ", "", "     return client, nil"),
    (" ", "", "}"),
]
line_h = 23
max_chars = int(dw / 7.2)
for sign, color, code in diffs:
    bg = '#1a1a28' if not sign else ('#1a3a2a' if sign == '+' else '#3a1a1a')
    draw.rectangle([dx+5, dy, dx+dw-5, dy+line_h], fill=bg)
    draw.text((dx+14, dy+4), f"{sign:>2}", font=mono_f, fill=color if color else gray)
    display_code = code[:max_chars] if len(code) > max_chars else code
    fg_color = '#cdd6f4' if not sign or sign == '+' else '#f5c2e7'
    draw.text((dx+34, dy+4), display_code, font=mono_f, fill=fg_color)
    dy += line_h

status_y = app_y+app_h-28
draw.rectangle([dx, status_y, dx+dw, app_y+app_h], fill='#11111b')
draw.text((dx+12, status_y+8), "3 files changed · +89 -42", font=small_f, fill=gray)

# AI Panel
ax = dx+dw+gap
aw = 94
draw.rounded_rectangle([ax, app_y, ax+aw, app_y+app_h], radius=10, fill='#181825')
draw.text((ax+aw//2, app_y+18), "AI Chat", font=small_f, fill=teal, anchor="mm")

msg_y = app_y+40
draw.rounded_rectangle([ax+6, msg_y, ax+aw-6, msg_y+62], radius=8, fill='#313244')
draw.text((ax+14, msg_y+14), "Analyze this", font=small_f, fill='#cdd6f4')
draw.text((ax+14, msg_y+32), "diff please", font=small_f, fill='#cdd6f4')

rsp_y = msg_y+72
draw.rounded_rectangle([ax+6, rsp_y, ax+aw-6, rsp_y+120], radius=8, fill=indigo)
lines_rsp = ["This adds", "OpenAI client", "with error", "handling."]
for i, ln in enumerate(lines_rsp):
    draw.text((ax+14, rsp_y+14+i*22), ln, font=small_f, fill='white')

# Typing dots
type_y = rsp_y+132
for i in range(3):
    dot_off = math.sin(type_y/200 + i*0.8) * 2
    draw.ellipse([ax+20+i*17, type_y+6+dot_off, ax+27+i*17, type_y+13+dot_off], fill=gray)

# === Arrow 2 ===
arr2x = b2x + b2w + 25
for i in range(36):
    r = int(168*(1-i/36) + 20*i/36)
    g = int(85*(1-i/36) + 184*i/36)
    b_val = int(247*(1-i/36) + 166*i/36)
    draw.line([(arr2x+i, arr_y), (arr2x+i, arr_y+2)], fill=f'#{r:02x}{g:02x}{b_val:02x}')
draw.polygon([(arr2x+38, arr_y-12), (arr2x+56, arr_y), (arr2x+38, arr_y+12)], fill=teal)

# === Block 3: AI Insights ===
b3x = b2x + b2w + 90
b3w, b3h = 340, 530
shadow_card(b3x, b1y, b3w, b3h)

draw.rounded_rectangle([b3x, b1y, b3x+b3w, b1y+56],
                        corners=[(20,0),(20,0),(0,0),(0,20)], fill='#ECFEFF')
draw.rectangle([b3x+4, b1y+52, b3x+b3w-4, b1y+56], fill=teal)
draw.text((b3x+b3w//2, b1y+30), "AI Insights", font=h2_f, fill=dark, anchor="mm")

brain_cx = b3x + b3w//2
brain_y = b1y + 115
r_big = 44
draw.ellipse([brain_cx-r_big, brain_y-r_big, brain_cx+r_big, brain_y+r_big],
            fill='#F0FDFA', outline=teal, width=2)
nodes = [(0,-22), (-20,7), (20,7), (-11,25), (11,25)]
node_pos = [(brain_cx+n[0], brain_y+n[1]) for n in nodes]
for i, (nx, ny) in enumerate(node_pos):
    draw.ellipse([nx-9, ny-9, nx+9, ny+9], fill=teal if i%2==0 else purple)
for i in range(len(nodes)):
    for j in range(i+1, len(nodes)):
        draw.line([node_pos[i], node_pos[j]], fill='#99F6E4', width=1)

sparkles = [(brain_cx-63,brain_y-42),(brain_cx+66,brain_y-32),
            (brain_cx-53,brain_y+48),(brain_cx+58,brain_y+46)]
for sx, sy in sparkles:
    for angle in range(0,360,60):
        rad = math.radians(angle)
        draw.line([(sx,sy),(sx+int(7*math.cos(rad)),sy+int(7*math.sin(rad)))], fill=amber, width=2)

insights = [
    ("Security Review", "No sensitive data exposure", '#DCFCE7', '#166534'),
    ("Code Quality", "Good error handling pattern", '#EFF6FF', '#1D4ED8'),
    ("Impact", "Medium — auth module only", '#FEF3C7', '#92400E'),
    ("Suggestion", "Add unit tests for coverage", '#F3E8FF', '#7C3AED'),
]
card_y = brain_y + 78
for title, desc, bg_col, fg_col in insights:
    ch = 60
    draw.rounded_rectangle([b3x+20, card_y, b3x+b3w-20, card_y+ch], radius=10, fill=bg_col)
    draw.text((b3x+32, card_y+12), title, font=body_f, fill=fg_col)
    draw.text((b3x+32, card_y+36), desc, font=small_f, fill=gray)
    card_y += ch + 8

# Bottom labels
labels_y = b1y + bh + 25
for lx, ltxt in [(b1x+bw//2,"Branch Explorer"), (b2x+b2w//2,"Desktop App"), (b3x+b3w//2,"AI Analysis")]:
    draw.text((lx, labels_y), ltxt, font=body_f, fill=gray, anchor="mm")
    draw.line([(lx-48, labels_y+18), (lx+48, labels_y+18)], fill=light_gray, width=2)

# Tech badges
badge_y = labels_y + 42
badges = [("Rust",indigo),("Tauri v2",purple),("Vue 3",teal),("TypeScript",amber)]
bx_start = W//2 - sum(len(t)*9+50 for t,c in badges)//2
for txt, col in badges:
    tw = len(txt)*9 + 26
    draw.rounded_rectangle([bx_start, badge_y, bx_start+tw, badge_y+30], radius=15, fill=col)
    draw.text((bx_start+tw//2, badge_y+16), txt, font=small_f, fill='white', anchor="mm")
    bx_start += tw + 8

out = "/Users/wei.lli/projects/gitx/docs/gitx-arch.png"
img.save(out, 'PNG', quality=95)
print(f"Done! Saved: {out} ({os.path.getsize(out)//1024}KB)")
