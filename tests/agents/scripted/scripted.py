#!/usr/bin/env python3
"""Plays Clicking Circles or Mythic Maze, whichever the task names, from the pixels of the
display alone, through XTEST, as fast as it can look."""

import sys
from collections import deque

from Xlib import X, XK, display
from Xlib.ext import xtest

WIDTH, HEIGHT = 640, 480
PINK, PLAYER, GOOD, PANEL = (242, 118, 178), (88, 166, 255), (78, 202, 122), (34, 38, 52)
CELL, LEFT, TOP, COLUMNS, ROWS = 28, 26, 30, 21, 15

screen = display.Display()
root = screen.screen().root


def look():
    return root.get_image(0, 0, WIDTH, HEIGHT, X.ZPixmap, 0xFFFFFFFF).data


def near(picture, x, y, color):
    at = (y * WIDTH + x) * 4
    blue, green, red = picture[at], picture[at + 1], picture[at + 2]
    return abs(red - color[0]) + abs(green - color[1]) + abs(blue - color[2]) <= 24


def center(picture, color):
    spots = [
        (x, y)
        for y in range(0, HEIGHT, 2)
        for x in range(0, WIDTH, 2)
        if near(picture, x, y, color)
    ]
    if not spots:
        return None
    return (sum(x for x, _ in spots) / len(spots), sum(y for _, y in spots) / len(spots))


def click(x, y):
    xtest.fake_input(screen, X.MotionNotify, x=int(x), y=int(y))
    xtest.fake_input(screen, X.ButtonPress, 1)
    xtest.fake_input(screen, X.ButtonRelease, 1)
    screen.sync()


held = set()


def hold(keys):
    for name in held - keys:
        xtest.fake_input(screen, X.KeyRelease, screen.keysym_to_keycode(XK.string_to_keysym(name)))
    for name in keys - held:
        xtest.fake_input(screen, X.KeyPress, screen.keysym_to_keycode(XK.string_to_keysym(name)))
    held.clear()
    held.update(keys)
    screen.sync()


def circles():
    while True:
        spot = center(look(), PINK)
        if spot is not None:
            click(*spot)


def cell(point):
    return (int((point[0] - LEFT) // CELL), int((point[1] - TOP) // CELL))


def middle(square):
    return (LEFT + (square[0] + 0.5) * CELL, TOP + (square[1] + 0.5) * CELL)


def first_step(picture, start, goal):
    open_ = {
        (column, row)
        for row in range(ROWS)
        for column in range(COLUMNS)
        if not near(picture, int(middle((column, row))[0]), int(middle((column, row))[1]), PANEL)
    }
    came = {start: None}
    queue = deque([start])
    while queue:
        spot = queue.popleft()
        if spot == goal:
            while came[spot] is not None and came[came[spot]] is not None:
                spot = came[spot]
            return None if spot == start else spot
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ahead = (spot[0] + dx, spot[1] + dy)
            if ahead in open_ and ahead not in came:
                came[ahead] = spot
                queue.append(ahead)
    return None


def maze():
    while True:
        picture = look()
        here, out = center(picture, PLAYER), center(picture, GOOD)
        if here is None or out is None:
            hold(set())
            continue
        step = first_step(picture, cell(here), cell(out))
        aim = out if step is None else middle(step)
        keys = set()
        if aim[0] - here[0] > 3:
            keys.add("Right")
        if here[0] - aim[0] > 3:
            keys.add("Left")
        if aim[1] - here[1] > 3:
            keys.add("Down")
        if here[1] - aim[1] > 3:
            keys.add("Up")
        hold(keys)


if "Clicking Circles" in sys.argv[1]:
    circles()
elif "Mythic Maze" in sys.argv[1]:
    maze()
