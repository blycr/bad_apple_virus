# Compare captured screenshots against boxes.bin frames (best match)
from PIL import Image
import numpy as np

with open('assets/boxes.bin', 'rb') as f:
    data = f.read()

# parse frames
frames = []
cur = []
i = 0
while i + 4 <= len(data):
    x, y, w, h = data[i], data[i+1], data[i+2], data[i+3]
    if (x, y, w, h) == (0, 0, 0, 0):
        frames.append(cur)
        cur = []
    else:
        cur.append((x, y, w, h))
    i += 4

def frame_grid(idx):
    g = np.zeros((48, 64), dtype=bool)
    for (x, y, w, h) in frames[idx]:
        g[y:y+h, x:x+w] = True
    return g

def shot_grid(name):
    im = Image.open(name).convert('RGB')
    a = np.asarray(im).astype(int)
    lum = a.mean(axis=2)
    g = np.zeros((48, 64), dtype=bool)
    for gy in range(48):
        for gx in range(64):
            x0 = 256 + gx * 22
            y0 = 12 + gy * 22
            region = lum[y0:y0+22, x0:x0+22]
            g[gy, gx] = (region > 200).mean() > 0.5
    return g

for name, t in [('final_6.png', 6), ('final_12.png', 12), ('final_20.png', 20)]:
    shot = shot_grid(name)
    # search frames near t*30 (minus ~6 startup frames)
    best = None
    for fidx in range(max(0, t*30 - 20), min(len(frames), t*30 + 20)):
        fg = frame_grid(fidx)
        # only compare cells that differ from "expected white density"? use simple mismatch
        diff = (shot != fg).mean()
        if best is None or diff < best[0]:
            best = (diff, fidx)
    diff, fidx = best
    print('=' * 72)
    print(f'{name}: best match frame {fidx} (t={fidx/30:.1f}s), mismatch {diff:.3f}')
    fg = frame_grid(fidx)
    # render both side by side
    lines = []
    for gy in range(48):
        a = ''.join('#' if shot[gy, gx] else '.' for gx in range(64))
        b = ''.join('#' if fg[gy, gx] else '.' for gx in range(64))
        lines.append(f'{a}   |   {b}')
    print('\n'.join(lines[:20]))
    print('  ...')
    print('\n'.join(lines[28:48]))
