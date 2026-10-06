#!/usr/bin/env python3
"""Holds Right across two presses of F1, taps Space between two presses, and exits."""

import time

from Xlib import X, XK, display
from Xlib.ext import xtest

screen = display.Display()


def key(name, down):
    code = screen.keysym_to_keycode(XK.string_to_keysym(name))
    xtest.fake_input(screen, X.KeyPress if down else X.KeyRelease, code)
    screen.sync()


def advance():
    key("F1", True)
    key("F1", False)
    time.sleep(0.05)


key("Right", True)
advance()
advance()
key("Right", False)
advance()
key("space", True)
key("space", False)
advance()
