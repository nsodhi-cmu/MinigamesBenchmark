#!/usr/bin/env python3
"""A chat-completions endpoint on 127.0.0.1:8000 that answers with pyautogui code, following
a script for the pace named on its command line, and prints how many messages and pictures
each request carried."""

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

CLOCK = ["```python\npyautogui.click(320, 240)\npyautogui.press('space')\n```"]
TICKS = [
    "```python\npyautogui.moveTo(320, 240)\npyautogui.mouseDown()\n```",
    "this is not python (((",
    "```python\npyautogui.mouseUp()\npyautogui.click(100, 100)\n```",
    "```python\npass\n```",
]
SCRIPT = TICKS if sys.argv[1:] == ["ticks"] else CLOCK
asked = []


class Model(BaseHTTPRequestHandler):
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        messages = request["messages"]
        pictures = [part for part in messages[-1]["content"] if part["type"] == "image_url"]
        print(f"asked with {len(messages)} messages and {len(pictures)} pictures", flush=True)
        answer = SCRIPT[min(len(asked), len(SCRIPT) - 1)]
        asked.append(answer)
        body = json.dumps({"choices": [{"message": {"content": answer}}]}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


HTTPServer(("127.0.0.1", 8000), Model).serve_forever()
