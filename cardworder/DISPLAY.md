# ST7789V2 Display Memory Map & Hardware Scroll

## Hardware Configuration

- **Display controller**: ST7789V2
- **Native framebuffer**: 240 columns × 320 rows (RGB565, 16-bit per pixel)
- **Physical LCD panel**: 240 × 135 pixels
- **Display offset**: column +40, row +53 (panel wired to a sub-region of RAM)
- **Rotation**: 90° with BottomToTop refresh
- **SPI clock**: 80 MHz (effective ~15 MHz due to driver overhead)

## Coordinate System

After 90° rotation:
- Native **rows** (0-319) → **horizontal axis** on physical screen (left-right)
- Native **columns** (0-239) → **vertical axis** on physical screen (top-bottom)

Our framebuffer flush uses:
- `SetColumnAddress(40, 279)` → 240 native columns = screen vertical (top to bottom)
- `SetPageAddress(53+y, 53+y)` → native rows = screen horizontal (left to right)

## Full Chip Memory Map

```
240 columns × 320 rows = 153,600 pixels = 307,200 bytes total RAM

Column axis (vertical on screen) — FIXED, never affected by scroll:

  Cols 0-39    [40 cols]  RAM exists, but NO physical pixels wired here
  Cols 40-279  [240 cols] Connected to physical LCD — THIS IS VISIBLE
  Cols 280-319 [40 cols]  RAM exists, but NO physical pixels wired here

Row axis (horizontal on screen) — SCROLLABLE via SetScrollStart:

  Rows 0-52    [53 rows]  Free RAM (writable at cols 40-279, scrollable)
  Rows 53-187  [135 rows] OUR CONTENT (framebuffer lives here)
  Rows 188-319 [132 rows] Free RAM (writable at cols 40-279, scrollable)
```

### Visual Map (cols = vertical, rows = horizontal)

```
         Row 0        Row 52 Row 53                  Row 187 Row 188                 Row 319
          |             |     |                        |      |                        |
Col   0:  ·  ·  ·  ·  · ·  · ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
...       (cols 0-39: no physical pixels — invisible regardless of scroll)
Col  39:  ·  ·  ·  ·  · ·  · ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
          ┌──────────────┬────────────────────────────┬────────────────────────────────┐
Col  40:  │ A  A  A  A  A│ O  O  O  O  O  O  O  O  O │ B  B  B  B  B  B  B  B  B  B  │
Col  41:  │ A  A  A  A  A│ O  O  O  O  O  O  O  O  O │ B  B  B  B  B  B  B  B  B  B  │
...       │              │                            │                                │
Col 150:  │ A  A  A  A  A│ O  O  O  O  O  O  O  O  O │ B  B  B  B  B  B  B  B  B  B  │
...       │              │                            │                                │
Col 279:  │ A  A  A  A  A│ O  O  O  O  O  O  O  O  O │ B  B  B  B  B  B  B  B  B  B  │
          └──────────────┴────────────────────────────┴────────────────────────────────┘
Col 280:  ·  ·  ·  ·  · ·  · ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·
...       (cols 280-319: no physical pixels — invisible regardless of scroll)
Col 319:  ·  ·  ·  ·  · ·  · ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·

Legend:
  ·  = RAM exists but no physical pixels (cols 0-39, 280-319) — always invisible
  O  = Our framebuffer content (cols 40-279, rows 53-187) — 240 × 135 = main screen
  A  = Free scrollable RAM (cols 40-279, rows 0-52) — 240 × 53 pixels
  B  = Free scrollable RAM (cols 40-279, rows 188-319) — 240 × 132 pixels
```

## Hardware Scroll

### How It Works

The ST7789V2 supports vertical scrolling via two commands:

1. **SetScrollArea(TFA, VSA, BFA)** — defines scroll region
   - TFA = Top Fixed Area (rows that don't scroll)
   - VSA = Vertical Scroll Area (rows that scroll)
   - BFA = Bottom Fixed Area (rows that don't scroll)
   - Constraint: TFA + VSA + BFA = 320
   - Current config: `SetScrollArea(0, 320, 0)` — all 320 rows scroll

2. **SetScrollStart(offset)** — which row appears at the physical top (left edge after rotation)
   - The display reads 135 consecutive rows starting from `offset`
   - Rows wrap at the 320 boundary (row 319 wraps to row 0)
   - Only sends 4 bytes via SPI — instant, no pixel data transfer

### After 90° rotation, "vertical scroll" = horizontal shift on physical screen.

### Scroll Offset Behavior

The physical LCD panel always shows exactly 135 consecutive rows from RAM.
The `offset` determines which row appears at the left edge of the screen.

```
offset = 53 (NORMAL — shows our content):

  Physical screen left edge                    right edge
  |                                                     |
  [row 53] [row 54] ... [row 186] [row 187]
  [  O  O    O  O    O    O  O      O  O  ]  ← full content visible


offset = 188 (HIDDEN — shows empty area B):

  [row 188] [row 189] ... [row 319] [row 0] [row 1] [row 2]
  [  B   B    B   B    B    B   B     A  A    A  A    A  A ]  ← content NOT visible


offset = 0 (PARTIAL — cuts off content):

  [row 0] [row 1] ... [row 52] [row 53] ... [row 134]
  [ A  A    A  A   A    A  A     O  O    O    O  O   ]  ← only rows 53-134 visible!
                                                         rows 135-187 cut off!
```

### Safe Scroll Ranges

For our content at rows 53-187 (135 rows):

| Offset | Visible rows | Content visible? | Use |
|--------|-------------|------------------|-----|
| 53 | 53-187 | YES — full content | Normal display |
| 188 | 188-319,0-2 | NO — content hidden | Start of slide-in animation |
| 0 | 0-134 | PARTIAL (53-134 only) | Avoid — cuts off bottom |
| 280 | 280-319,0-94 | PARTIAL (53-94 overlaps!) | Avoid — wraps into content |

**Safe range to hide content**: offsets 188 through 185 (wrapping via 319→0).
Any offset in this range keeps rows 53-187 completely outside the visible window.

**Slide-in animation**: scroll from 188 → 53

### Writing to Free RAM Areas

To use zones A or B for a second page or animation:

```rust
// Write to zone B (rows 188-319) using SetPageAddress
unsafe {
    let screen = &mut self.display.screen;
    screen.dcs().write_command(SetColumnAddress::new(40, 279)).unwrap();
    screen.dcs().write_command(SetPageAddress::new(188, 319)).unwrap(); // zone B
    screen.dcs().write_command(WriteMemoryStart).unwrap();
    screen.dcs().di.send_data(DataFormat::U8(&pixel_data)).unwrap();
}
```

Zone B is 132 rows — 3 rows short of a full 135-row screen. For a complete second page,
use zone B (132 rows) + part of zone A (3 rows from rows 0-2) = 135 rows.
Scroll offset 188 would show this combined area perfectly.

### Animation Examples

**Slide-in from right:**
1. Render content to rows 53-187 (normal flush)
2. Set scroll offset to 188 (content off-screen)
3. Animate: 188 → 53 (content slides in from right)

**Page transition (current → new):**
1. Current page at rows 53-187 (already displayed, offset=53)
2. Render new page to rows 188-319 + 0-2 (zone B + partial A)
3. Animate: offset 53 → 188 (slide current page out, new page in)
4. Copy new page from zone B to rows 53-187 for normal operation

**Smooth horizontal text scroll:**
1. Render wide text across rows 53-319 (wider than screen)
2. Animate scroll offset: 53 → 53+N to pan across text

## Memory Budget

| Area | Cols | Rows | Pixels | Bytes (RGB565) |
|------|------|------|--------|----------------|
| Full RAM | 240 | 320 | 76,800 | 153,600 |
| Physical LCD | 240 | 135 | 32,400 | 64,800 |
| Zone O (our content) | 240 | 135 | 32,400 | 64,800 |
| Zone A (free top) | 240 | 53 | 12,720 | 25,440 |
| Zone B (free bottom) | 240 | 132 | 31,680 | 63,360 |
| A+B (total free) | 240 | 185 | 44,400 | 88,800 |
| Invisible cols | 80 | 320 | 25,600 | 51,200 |

## Full Pixel Map (1 char = 1 row, columns grouped)

320 rows shown vertically, column ranges shown as zones.
Each line = one native row. Read top-to-bottom = left-to-right on physical screen.

```
ROW  | cols 0-39 | cols 40-52 | cols 53-187 ... wait, columns are 0-239 not 0-319
```

**Note:** Columns range 0-239 (240 total), Rows range 0-319 (320 total).
The display_offset is (col_offset=40, row_offset=53). But in mipidsi with 90° rotation,
`display_offset(52, 40)` maps to: SetColumnAddress adds 40, SetPageAddress adds 53.

Below: each line = 1 native row. Column groups shown as character blocks.

```
     cols: |0         39|40        279|280      319|
     size: |  40 cols   | 240 cols    | 40 cols    |
     vis:  | invisible  | VISIBLE LCD | invisible  |
           |            |             |            |
row   0:   ··············AAAAAAAAAAAAA··············  ← zone A (free, scrollable)
row   1:   ··············AAAAAAAAAAAAA··············
row   2:   ··············AAAAAAAAAAAAA··············
row   3:   ··············AAAAAAAAAAAAA··············
row   4:   ··············AAAAAAAAAAAAA··············
row   5:   ··············AAAAAAAAAAAAA··············
row   6:   ··············AAAAAAAAAAAAA··············
row   7:   ··············AAAAAAAAAAAAA··············
row   8:   ··············AAAAAAAAAAAAA··············
row   9:   ··············AAAAAAAAAAAAA··············
row  10:   ··············AAAAAAAAAAAAA··············
row  11:   ··············AAAAAAAAAAAAA··············
row  12:   ··············AAAAAAAAAAAAA··············
row  13:   ··············AAAAAAAAAAAAA··············
row  14:   ··············AAAAAAAAAAAAA··············
row  15:   ··············AAAAAAAAAAAAA··············
row  16:   ··············AAAAAAAAAAAAA··············
row  17:   ··············AAAAAAAAAAAAA··············
row  18:   ··············AAAAAAAAAAAAA··············
row  19:   ··············AAAAAAAAAAAAA··············
row  20:   ··············AAAAAAAAAAAAA··············
row  21:   ··············AAAAAAAAAAAAA··············
row  22:   ··············AAAAAAAAAAAAA··············
row  23:   ··············AAAAAAAAAAAAA··············
row  24:   ··············AAAAAAAAAAAAA··············
row  25:   ··············AAAAAAAAAAAAA··············
row  26:   ··············AAAAAAAAAAAAA··············
row  27:   ··············AAAAAAAAAAAAA··············
row  28:   ··············AAAAAAAAAAAAA··············
row  29:   ··············AAAAAAAAAAAAA··············
row  30:   ··············AAAAAAAAAAAAA··············
row  31:   ··············AAAAAAAAAAAAA··············
row  32:   ··············AAAAAAAAAAAAA··············
row  33:   ··············AAAAAAAAAAAAA··············
row  34:   ··············AAAAAAAAAAAAA··············
row  35:   ··············AAAAAAAAAAAAA··············
row  36:   ··············AAAAAAAAAAAAA··············
row  37:   ··············AAAAAAAAAAAAA··············
row  38:   ··············AAAAAAAAAAAAA··············
row  39:   ··············AAAAAAAAAAAAA··············
row  40:   ··············AAAAAAAAAAAAA··············
row  41:   ··············AAAAAAAAAAAAA··············
row  42:   ··············AAAAAAAAAAAAA··············
row  43:   ··············AAAAAAAAAAAAA··············
row  44:   ··············AAAAAAAAAAAAA··············
row  45:   ··············AAAAAAAAAAAAA··············
row  46:   ··············AAAAAAAAAAAAA··············
row  47:   ··············AAAAAAAAAAAAA··············
row  48:   ··············AAAAAAAAAAAAA··············
row  49:   ··············AAAAAAAAAAAAA··············
row  50:   ··············AAAAAAAAAAAAA··············
row  51:   ··············AAAAAAAAAAAAA··············
row  52:   ··············AAAAAAAAAAAAA··············  ← last row of zone A (53 rows total)
---------- ──────────────────────────────────────── boundary
row  53:   ··············OOOOOOOOOOOOO··············  ← zone O starts (our framebuffer)
row  54:   ··············OOOOOOOOOOOOO··············
row  55:   ··············OOOOOOOOOOOOO··············
row  56:   ··············OOOOOOOOOOOOO··············
row  57:   ··············OOOOOOOOOOOOO··············
row  58:   ··············OOOOOOOOOOOOO··············
row  59:   ··············OOOOOOOOOOOOO··············
row  60:   ··············OOOOOOOOOOOOO··············
row  61:   ··············OOOOOOOOOOOOO··············
row  62:   ··············OOOOOOOOOOOOO··············
row  63:   ··············OOOOOOOOOOOOO··············
row  64:   ··············OOOOOOOOOOOOO··············
row  65:   ··············OOOOOOOOOOOOO··············
row  66:   ··············OOOOOOOOOOOOO··············
row  67:   ··············OOOOOOOOOOOOO··············
row  68:   ··············OOOOOOOOOOOOO··············
row  69:   ··············OOOOOOOOOOOOO··············
row  70:   ··············OOOOOOOOOOOOO··············
row  71:   ··············OOOOOOOOOOOOO··············
row  72:   ··············OOOOOOOOOOOOO··············
row  73:   ··············OOOOOOOOOOOOO··············
row  74:   ··············OOOOOOOOOOOOO··············
row  75:   ··············OOOOOOOOOOOOO··············
row  76:   ··············OOOOOOOOOOOOO··············
row  77:   ··············OOOOOOOOOOOOO··············
row  78:   ··············OOOOOOOOOOOOO··············
row  79:   ··············OOOOOOOOOOOOO··············
row  80:   ··············OOOOOOOOOOOOO··············
row  81:   ··············OOOOOOOOOOOOO··············
row  82:   ··············OOOOOOOOOOOOO··············
row  83:   ··············OOOOOOOOOOOOO··············
row  84:   ··············OOOOOOOOOOOOO··············
row  85:   ··············OOOOOOOOOOOOO··············
row  86:   ··············OOOOOOOOOOOOO··············
row  87:   ··············OOOOOOOOOOOOO··············
row  88:   ··············OOOOOOOOOOOOO··············
row  89:   ··············OOOOOOOOOOOOO··············
row  90:   ··············OOOOOOOOOOOOO··············
row  91:   ··············OOOOOOOOOOOOO··············
row  92:   ··············OOOOOOOOOOOOO··············
row  93:   ··············OOOOOOOOOOOOO··············
row  94:   ··············OOOOOOOOOOOOO··············
row  95:   ··············OOOOOOOOOOOOO··············
row  96:   ··············OOOOOOOOOOOOO··············
row  97:   ··············OOOOOOOOOOOOO··············
row  98:   ··············OOOOOOOOOOOOO··············
row  99:   ··············OOOOOOOOOOOOO··············
row 100:   ··············OOOOOOOOOOOOO··············
row 101:   ··············OOOOOOOOOOOOO··············
row 102:   ··············OOOOOOOOOOOOO··············
row 103:   ··············OOOOOOOOOOOOO··············
row 104:   ··············OOOOOOOOOOOOO··············
row 105:   ··············OOOOOOOOOOOOO··············
row 106:   ··············OOOOOOOOOOOOO··············
row 107:   ··············OOOOOOOOOOOOO··············
row 108:   ··············OOOOOOOOOOOOO··············
row 109:   ··············OOOOOOOOOOOOO··············
row 110:   ··············OOOOOOOOOOOOO··············
row 111:   ··············OOOOOOOOOOOOO··············
row 112:   ··············OOOOOOOOOOOOO··············
row 113:   ··············OOOOOOOOOOOOO··············
row 114:   ··············OOOOOOOOOOOOO··············
row 115:   ··············OOOOOOOOOOOOO··············
row 116:   ··············OOOOOOOOOOOOO··············
row 117:   ··············OOOOOOOOOOOOO··············
row 118:   ··············OOOOOOOOOOOOO··············
row 119:   ··············OOOOOOOOOOOOO··············
row 120:   ··············OOOOOOOOOOOOO··············
row 121:   ··············OOOOOOOOOOOOO··············
row 122:   ··············OOOOOOOOOOOOO··············
row 123:   ··············OOOOOOOOOOOOO··············
row 124:   ··············OOOOOOOOOOOOO··············
row 125:   ··············OOOOOOOOOOOOO··············
row 126:   ··············OOOOOOOOOOOOO··············
row 127:   ··············OOOOOOOOOOOOO··············
row 128:   ··············OOOOOOOOOOOOO··············
row 129:   ··············OOOOOOOOOOOOO··············
row 130:   ··············OOOOOOOOOOOOO··············
row 131:   ··············OOOOOOOOOOOOO··············
row 132:   ··············OOOOOOOOOOOOO··············
row 133:   ··············OOOOOOOOOOOOO··············
row 134:   ··············OOOOOOOOOOOOO··············
row 135:   ··············OOOOOOOOOOOOO··············
row 136:   ··············OOOOOOOOOOOOO··············
row 137:   ··············OOOOOOOOOOOOO··············
row 138:   ··············OOOOOOOOOOOOO··············
row 139:   ··············OOOOOOOOOOOOO··············
row 140:   ··············OOOOOOOOOOOOO··············
row 141:   ··············OOOOOOOOOOOOO··············
row 142:   ··············OOOOOOOOOOOOO··············
row 143:   ··············OOOOOOOOOOOOO··············
row 144:   ··············OOOOOOOOOOOOO··············
row 145:   ··············OOOOOOOOOOOOO··············
row 146:   ··············OOOOOOOOOOOOO··············
row 147:   ··············OOOOOOOOOOOOO··············
row 148:   ··············OOOOOOOOOOOOO··············
row 149:   ··············OOOOOOOOOOOOO··············
row 150:   ··············OOOOOOOOOOOOO··············
row 151:   ··············OOOOOOOOOOOOO··············
row 152:   ··············OOOOOOOOOOOOO··············
row 153:   ··············OOOOOOOOOOOOO··············
row 154:   ··············OOOOOOOOOOOOO··············
row 155:   ··············OOOOOOOOOOOOO··············
row 156:   ··············OOOOOOOOOOOOO··············
row 157:   ··············OOOOOOOOOOOOO··············
row 158:   ··············OOOOOOOOOOOOO··············
row 159:   ··············OOOOOOOOOOOOO··············
row 160:   ··············OOOOOOOOOOOOO··············
row 161:   ··············OOOOOOOOOOOOO··············
row 162:   ··············OOOOOOOOOOOOO··············
row 163:   ··············OOOOOOOOOOOOO··············
row 164:   ··············OOOOOOOOOOOOO··············
row 165:   ··············OOOOOOOOOOOOO··············
row 166:   ··············OOOOOOOOOOOOO··············
row 167:   ··············OOOOOOOOOOOOO··············
row 168:   ··············OOOOOOOOOOOOO··············
row 169:   ··············OOOOOOOOOOOOO··············
row 170:   ··············OOOOOOOOOOOOO··············
row 171:   ··············OOOOOOOOOOOOO··············
row 172:   ··············OOOOOOOOOOOOO··············
row 173:   ··············OOOOOOOOOOOOO··············
row 174:   ··············OOOOOOOOOOOOO··············
row 175:   ··············OOOOOOOOOOOOO··············
row 176:   ··············OOOOOOOOOOOOO··············
row 177:   ··············OOOOOOOOOOOOO··············
row 178:   ··············OOOOOOOOOOOOO··············
row 179:   ··············OOOOOOOOOOOOO··············
row 180:   ··············OOOOOOOOOOOOO··············
row 181:   ··············OOOOOOOOOOOOO··············
row 182:   ··············OOOOOOOOOOOOO··············
row 183:   ··············OOOOOOOOOOOOO··············
row 184:   ··············OOOOOOOOOOOOO··············
row 185:   ··············OOOOOOOOOOOOO··············
row 186:   ··············OOOOOOOOOOOOO··············
row 187:   ··············OOOOOOOOOOOOO··············  ← last row of zone O (135 rows total)
---------- ──────────────────────────────────────── boundary
row 188:   ··············BBBBBBBBBBBBB··············  ← zone B starts (free, scrollable)
row 189:   ··············BBBBBBBBBBBBB··············
row 190:   ··············BBBBBBBBBBBBB··············
row 191:   ··············BBBBBBBBBBBBB··············
row 192:   ··············BBBBBBBBBBBBB··············
row 193:   ··············BBBBBBBBBBBBB··············
row 194:   ··············BBBBBBBBBBBBB··············
row 195:   ··············BBBBBBBBBBBBB··············
row 196:   ··············BBBBBBBBBBBBB··············
row 197:   ··············BBBBBBBBBBBBB··············
row 198:   ··············BBBBBBBBBBBBB··············
row 199:   ··············BBBBBBBBBBBBB··············
row 200:   ··············BBBBBBBBBBBBB··············
row 201:   ··············BBBBBBBBBBBBB··············
row 202:   ··············BBBBBBBBBBBBB··············
row 203:   ··············BBBBBBBBBBBBB··············
row 204:   ··············BBBBBBBBBBBBB··············
row 205:   ··············BBBBBBBBBBBBB··············
row 206:   ··············BBBBBBBBBBBBB··············
row 207:   ··············BBBBBBBBBBBBB··············
row 208:   ··············BBBBBBBBBBBBB··············
row 209:   ··············BBBBBBBBBBBBB··············
row 210:   ··············BBBBBBBBBBBBB··············
row 211:   ··············BBBBBBBBBBBBB··············
row 212:   ··············BBBBBBBBBBBBB··············
row 213:   ··············BBBBBBBBBBBBB··············
row 214:   ··············BBBBBBBBBBBBB··············
row 215:   ··············BBBBBBBBBBBBB··············
row 216:   ··············BBBBBBBBBBBBB··············
row 217:   ··············BBBBBBBBBBBBB··············
row 218:   ··············BBBBBBBBBBBBB··············
row 219:   ··············BBBBBBBBBBBBB··············
row 220:   ··············BBBBBBBBBBBBB··············
row 221:   ··············BBBBBBBBBBBBB··············
row 222:   ··············BBBBBBBBBBBBB··············
row 223:   ··············BBBBBBBBBBBBB··············
row 224:   ··············BBBBBBBBBBBBB··············
row 225:   ··············BBBBBBBBBBBBB··············
row 226:   ··············BBBBBBBBBBBBB··············
row 227:   ··············BBBBBBBBBBBBB··············
row 228:   ··············BBBBBBBBBBBBB··············
row 229:   ··············BBBBBBBBBBBBB··············
row 230:   ··············BBBBBBBBBBBBB··············
row 231:   ··············BBBBBBBBBBBBB··············
row 232:   ··············BBBBBBBBBBBBB··············
row 233:   ··············BBBBBBBBBBBBB··············
row 234:   ··············BBBBBBBBBBBBB··············
row 235:   ··············BBBBBBBBBBBBB··············
row 236:   ··············BBBBBBBBBBBBB··············
row 237:   ··············BBBBBBBBBBBBB··············
row 238:   ··············BBBBBBBBBBBBB··············
row 239:   ··············BBBBBBBBBBBBB··············
row 240:   ··············BBBBBBBBBBBBB··············
row 241:   ··············BBBBBBBBBBBBB··············
row 242:   ··············BBBBBBBBBBBBB··············
row 243:   ··············BBBBBBBBBBBBB··············
row 244:   ··············BBBBBBBBBBBBB··············
row 245:   ··············BBBBBBBBBBBBB··············
row 246:   ··············BBBBBBBBBBBBB··············
row 247:   ··············BBBBBBBBBBBBB··············
row 248:   ··············BBBBBBBBBBBBB··············
row 249:   ··············BBBBBBBBBBBBB··············
row 250:   ··············BBBBBBBBBBBBB··············
row 251:   ··············BBBBBBBBBBBBB··············
row 252:   ··············BBBBBBBBBBBBB··············
row 253:   ··············BBBBBBBBBBBBB··············
row 254:   ··············BBBBBBBBBBBBB··············
row 255:   ··············BBBBBBBBBBBBB··············
row 256:   ··············BBBBBBBBBBBBB··············
row 257:   ··············BBBBBBBBBBBBB··············
row 258:   ··············BBBBBBBBBBBBB··············
row 259:   ··············BBBBBBBBBBBBB··············
row 260:   ··············BBBBBBBBBBBBB··············
row 261:   ··············BBBBBBBBBBBBB··············
row 262:   ··············BBBBBBBBBBBBB··············
row 263:   ··············BBBBBBBBBBBBB··············
row 264:   ··············BBBBBBBBBBBBB··············
row 265:   ··············BBBBBBBBBBBBB··············
row 266:   ··············BBBBBBBBBBBBB··············
row 267:   ··············BBBBBBBBBBBBB··············
row 268:   ··············BBBBBBBBBBBBB··············
row 269:   ··············BBBBBBBBBBBBB··············
row 270:   ··············BBBBBBBBBBBBB··············
row 271:   ··············BBBBBBBBBBBBB··············
row 272:   ··············BBBBBBBBBBBBB··············
row 273:   ··············BBBBBBBBBBBBB··············
row 274:   ··············BBBBBBBBBBBBB··············
row 275:   ··············BBBBBBBBBBBBB··············
row 276:   ··············BBBBBBBBBBBBB··············
row 277:   ··············BBBBBBBBBBBBB··············
row 278:   ··············BBBBBBBBBBBBB··············
row 279:   ··············BBBBBBBBBBBBB··············
row 280:   ··············BBBBBBBBBBBBB··············
row 281:   ··············BBBBBBBBBBBBB··············
row 282:   ··············BBBBBBBBBBBBB··············
row 283:   ··············BBBBBBBBBBBBB··············
row 284:   ··············BBBBBBBBBBBBB··············
row 285:   ··············BBBBBBBBBBBBB··············
row 286:   ··············BBBBBBBBBBBBB··············
row 287:   ··············BBBBBBBBBBBBB··············
row 288:   ··············BBBBBBBBBBBBB··············
row 289:   ··············BBBBBBBBBBBBB··············
row 290:   ··············BBBBBBBBBBBBB··············
row 291:   ··············BBBBBBBBBBBBB··············
row 292:   ··············BBBBBBBBBBBBB··············
row 293:   ··············BBBBBBBBBBBBB··············
row 294:   ··············BBBBBBBBBBBBB··············
row 295:   ··············BBBBBBBBBBBBB··············
row 296:   ··············BBBBBBBBBBBBB··············
row 297:   ··············BBBBBBBBBBBBB··············
row 298:   ··············BBBBBBBBBBBBB··············
row 299:   ··············BBBBBBBBBBBBB··············
row 300:   ··············BBBBBBBBBBBBB··············
row 301:   ··············BBBBBBBBBBBBB··············
row 302:   ··············BBBBBBBBBBBBB··············
row 303:   ··············BBBBBBBBBBBBB··············
row 304:   ··············BBBBBBBBBBBBB··············
row 305:   ··············BBBBBBBBBBBBB··············
row 306:   ··············BBBBBBBBBBBBB··············
row 307:   ··············BBBBBBBBBBBBB··············
row 308:   ··············BBBBBBBBBBBBB··············
row 309:   ··············BBBBBBBBBBBBB··············
row 310:   ··············BBBBBBBBBBBBB··············
row 311:   ··············BBBBBBBBBBBBB··············
row 312:   ··············BBBBBBBBBBBBB··············
row 313:   ··············BBBBBBBBBBBBB··············
row 314:   ··············BBBBBBBBBBBBB··············
row 315:   ··············BBBBBBBBBBBBB··············
row 316:   ··············BBBBBBBBBBBBB··············
row 317:   ··············BBBBBBBBBBBBB··············
row 318:   ··············BBBBBBBBBBBBB··············
row 319:   ··············BBBBBBBBBBBBB··············  ← last row of zone B (132 rows total)
---------- ──────────────────────────────────────── wraps to row 0

Each line: 14 · chars (cols 0-39, scaled ~3:1) + 13 zone chars (cols 40-279, scaled ~18:1) + 14 · chars (cols 280-319)
True pixel counts per zone per row: 40 invisible + 240 visible + 40 invisible = 320 cols total

Summary:
  · (dot)  = RAM exists, no physical pixel (cols 0-39 and 280-319) — 80 cols × 320 rows
  A        = free scrollable RAM, visible cols (cols 40-279, rows 0-52) — 240 × 53 = 12,720 px
  O        = our framebuffer content (cols 40-279, rows 53-187) — 240 × 135 = 32,400 px
  B        = free scrollable RAM, visible cols (cols 40-279, rows 188-319) — 240 × 132 = 31,680 px

Scroll reads 135 consecutive rows (wrapping at 319→0).
Normal display: scroll_offset=53 shows rows 53-187 (all O).
Hidden:         scroll_offset=188 shows rows 188-319+0-2 (all B+A, no O).
```

## Code References

- Scroll area init: `cardworder/src/cardputer_hal/screen/st7789v2.rs:52`
- Scroll offset method: `cardworder/src/ui/cardworder_ui.rs` — `set_scroll_offset()`
- Framebuffer flush: `cardworder/src/ui/cardworder_ui.rs` — `flip_buffer()`
- Display build: `cardworder/src/cardputer_hal/screen/display.rs`
- Display offset: `display_offset(52, 40)` in `display.rs`
