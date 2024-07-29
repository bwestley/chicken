import PIL
import PIL.ImageDraw as ImageDraw

MAX_PIPS = 12
TEXTURE_SIZE = 120
LINE_WIDTH = TEXTURE_SIZE // 15
MARGIN = TEXTURE_SIZE // 4
PADDING = TEXTURE_SIZE // 15
CELL_2 = (TEXTURE_SIZE - (MARGIN * 2)) // 2
CELL_3 = (TEXTURE_SIZE - (MARGIN * 2)) // 3
PIP_RADIUS = TEXTURE_SIZE // 15
MAP_WIDTH = TEXTURE_SIZE * (MAX_PIPS + 1)
MAP_HEIGHT = TEXTURE_SIZE * (MAX_PIPS + 1) * 2
# BACKGROUND_COLOR = (0, 0, 0, 0)
# LINE_COLOR = (0xFF, 0xFF, 0xFF, 0xFF)
LINE_COLOR = (0, 0, 0, 0xFF)
BACKGROUND_COLOR = (0xFF, 0xFF, 0xFF, 0xFF)
PIP_COLORS = [
    (rgb >> 16 & 0xFF, rgb >> 8 & 0xFF, rgb & 0xFF, 0xFF)
    for rgb in [
        0x000000,
        0xFF6F00,
        0xE454DD,
        0x34E024,
        0xFFC000,
        0x61CBF3,
        0x078F00,
        0x0070C0,
        0x9FA65F,
        0x7030A0,
        0x808080,
        0x9E6900,
        0xD30000,
    ]
]
PIP_POSITIONS = []
with open("pipPositions.txt") as f:
    ROW_Y = [
        MARGIN,
        MARGIN + CELL_3,
        MARGIN + CELL_2,
        MARGIN + CELL_3 * 2,
        MARGIN + CELL_2 * 2,
    ]
    for pips in range(MAX_PIPS + 1):
        PIP_POSITIONS.append([])
        for row in range(5):
            line = f.readline()
            for column in range(3):
                if line[column] == "1":
                    PIP_POSITIONS[pips].append((MARGIN + column * CELL_2, ROW_Y[row]))
        f.readline()  # Empty line

map = PIL.Image.new(mode="RGBA", size=(MAP_WIDTH, MAP_HEIGHT), color=(0, 0, 0, 0))
imageDraw = ImageDraw.Draw(map)


def draw_pips(imageDraw, x, y, pips):
    for pipPosition in PIP_POSITIONS[pips]:
        imageDraw.circle(
            xy=(x + pipPosition[0], y + pipPosition[1]),
            radius=PIP_RADIUS,
            fill=PIP_COLORS[pips],
            width=0,
        )


for max_ in range(0, MAX_PIPS + 1):
    y = max_ * TEXTURE_SIZE * 2
    for min_ in range(0, max_ + 1):
        x = min_ * TEXTURE_SIZE
        imageDraw.rectangle(
            [
                x + PADDING,
                y + PADDING,
                x + TEXTURE_SIZE - PADDING,
                y + TEXTURE_SIZE * 2 - PADDING,
            ],
            fill=BACKGROUND_COLOR,
            outline=LINE_COLOR,
            width=LINE_WIDTH,
        )
        imageDraw.line(
            xy=[
                (x + MARGIN, y + TEXTURE_SIZE),
                (x + TEXTURE_SIZE - MARGIN, y + TEXTURE_SIZE),
            ],
            fill=LINE_COLOR,
            width=LINE_WIDTH,
        )
        draw_pips(imageDraw, x, y, min_)
        draw_pips(imageDraw, x, y + TEXTURE_SIZE, max_)
map.save("set.png")

map = PIL.Image.new(mode="RGBA", size=(MAP_WIDTH, MAP_WIDTH), color=(0, 0, 0, 0))
imageDraw = ImageDraw.Draw(map)
for pips in range(0, MAX_PIPS + 1):
    x = TEXTURE_SIZE * pips
    imageDraw.rectangle(
        [x + PADDING, PADDING, x + TEXTURE_SIZE - PADDING, TEXTURE_SIZE - PADDING],
        fill=BACKGROUND_COLOR,
        outline=LINE_COLOR,
        width=LINE_WIDTH,
    )
    draw_pips(imageDraw, x, 0, pips)
map.save("pips.png")
