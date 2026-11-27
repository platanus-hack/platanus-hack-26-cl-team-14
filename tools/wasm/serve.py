#!/usr/bin/env python3
"""Serve the repository root for the browser build: python3 tools/wasm/serve.py [port].

A build replaces the files, so the browser must ask again for each one before it uses its copy.
"""
import http.server
import os
import sys
from urllib.parse import urlsplit


class Handler(http.server.SimpleHTTPRequestHandler):
    def translate_path(self, path):
        route = urlsplit(path).path
        if route in {"/", "/index.html", "/style.css", "/santiago.mp4", "/santiago.jpg", "/replays.html"}:
            path = "/landing/public" + path
        return super().translate_path(path)

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()


os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
port = int(sys.argv[1]) if len(sys.argv) > 1 else 8961
print(f"http://127.0.0.1:{port}/")
http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
