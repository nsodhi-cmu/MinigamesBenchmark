#!/usr/bin/env python3
"""A deliberately minimal computer-use harness, a placeholder for your own. It takes a
screenshot, asks a model behind an OpenAI-compatible endpoint what to do, and runs the Python
it answers with, `pyautogui` in hand. It keeps no history, tells the model nothing about what
its code did, and checks nothing beyond whether the answer compiles.

    harness [--base-url URL] [--model NAME] clock|ticks TASK

`clock` plays a game that runs in real time. `ticks` plays one that stands still until F1 is
pressed, and presses F1 after running each answer. The endpoint is `--base-url` or
`OPENAI_BASE_URL`, the model `--model` or `OPENAI_MODEL`, and the key `OPENAI_API_KEY`."""

import argparse
import base64
import io
import json
import os
import re
import sys
import time
import urllib.request

SETTLE = 0.02
"""Seconds between running an answer and taking the next screenshot."""

ANSWER = """\
Answer with Python code and nothing else, in one ```python block. `pyautogui` is already \
imported and set up; use it to move and click the mouse and to press and hold keys, with \
coordinates in pixels from the top left of the screen. `time` is imported too. You will see \
the screen again after your code has run, but nothing else: not your earlier answers, and not \
whether your code worked."""

CLOCK = f"""\
You are operating a computer by writing code. The game on the screen runs in real time, while \
you think and while your code runs: time.sleep and pyautogui's duration arguments take real \
time, and a key held with pyautogui.keyDown stays held until pyautogui.keyUp.

{ANSWER}"""

TICKS = f"""\
You are operating a computer by writing code. The game on the screen stands still, frame by \
frame. After your code runs, F1 is pressed for you, which lets a few frames pass, and then you \
see the screen again. The game reads only what is held down and where the pointer is at each \
press of F1, so a click, a press, or a keyDown and keyUp in the same answer does nothing but \
move the pointer. To press something, hold it with \
pyautogui.keyDown or pyautogui.mouseDown in one answer and let it go with pyautogui.keyUp or \
pyautogui.mouseUp in a later one. To drag, mouseDown in one answer, moveTo in the next, and \
mouseUp in the one after. Sleeping does not move the game. You may call \
pyautogui.press('f1') yourself to let more frames pass within one answer.

{ANSWER}"""

FENCED = re.compile(r"```(?:python|py)?[^\n]*\n(.*?)```", re.DOTALL)


def ask(base: str, model: str, key: str, prompt: str, task: str, png: bytes) -> str:
    """The model's answer to the task and one screenshot."""
    picture = "data:image/png;base64," + base64.b64encode(png).decode()
    body = {
        "model": model,
        "messages": [
            {"role": "system", "content": prompt},
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": task},
                    {"type": "image_url", "image_url": {"url": picture}},
                ],
            },
        ],
    }
    headers = {"Content-Type": "application/json"}
    if key:
        headers["Authorization"] = f"Bearer {key}"
    request = urllib.request.Request(
        f"{base.rstrip('/')}/chat/completions", json.dumps(body).encode(), headers
    )
    with urllib.request.urlopen(request, timeout=600) as reply:
        return json.load(reply)["choices"][0]["message"]["content"] or ""


def code(answer: str) -> str:
    """The Python in an answer: its fenced blocks, or all of it where there are none."""
    blocks = FENCED.findall(answer)
    return "\n".join(blocks) if blocks else answer


def run(source: str, pyautogui) -> None:
    """Runs `source`, ignoring whatever goes wrong."""
    try:
        compiled = compile(source, "<answer>", "exec")
    except (SyntaxError, ValueError):
        return
    try:
        exec(compiled, {"pyautogui": pyautogui, "time": time})
    except BaseException:
        pass


def main() -> None:
    parser = argparse.ArgumentParser(prog="harness", description=__doc__.split("\n\n")[0])
    parser.add_argument("--base-url", default=os.environ.get("OPENAI_BASE_URL", ""))
    parser.add_argument("--model", default=os.environ.get("OPENAI_MODEL", ""))
    parser.add_argument("pace", choices=("clock", "ticks"))
    parser.add_argument("task")
    arguments = parser.parse_args()
    if not arguments.base_url or not arguments.model:
        parser.error("name the endpoint and the model: --base-url and --model")

    import pyautogui

    pyautogui.PAUSE = 0
    pyautogui.FAILSAFE = False
    prompt = CLOCK if arguments.pace == "clock" else TICKS
    key = os.environ.get("OPENAI_API_KEY", "")
    while True:
        shot = io.BytesIO()
        try:
            pyautogui.screenshot().save(shot, "PNG")
        except Exception:
            time.sleep(SETTLE)
            continue
        try:
            answer = ask(arguments.base_url, arguments.model, key, prompt, arguments.task,
                         shot.getvalue())
        except Exception as failed:
            print(f"the endpoint failed: {failed}", file=sys.stderr, flush=True)
            time.sleep(1)
            continue
        run(code(answer), pyautogui)
        if arguments.pace == "ticks":
            pyautogui.press("f1")
        time.sleep(SETTLE)


if __name__ == "__main__":
    main()
