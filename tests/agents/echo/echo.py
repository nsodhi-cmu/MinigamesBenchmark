#!/usr/bin/env python3
"""Prints the task it was handed, then sends a fixed run of input through XTEST."""

import sys
import time

from Xlib import X, XK, display
from Xlib.ext import xtest

sys.stdout.write(sys.argv[1])
sys.stdout.flush()
screen = display.Display()


def key(name, down):
    code = screen.keysym_to_keycode(XK.string_to_keysym(name))
    xtest.fake_input(screen, X.KeyPress if down else X.KeyRelease, code)
    screen.sync()


def button(number, down):
    xtest.fake_input(screen, X.ButtonPress if down else X.ButtonRelease, number)
    screen.sync()


def move(x, y):
    xtest.fake_input(screen, X.MotionNotify, x=x, y=y)
    screen.sync()


move(100, 100)
button(1, True)
button(1, False)
key("Left", True)
time.sleep(0.3)
key("Left", False)
key("Shift_L", True)
key("b", True)
key("b", False)
key("Shift_L", False)
move(200, 200)
button(3, True)
button(3, False)
