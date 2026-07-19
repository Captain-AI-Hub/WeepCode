#!/usr/bin/env python3
"""Minimal OpenAI-compatible chat-completions mock for gate testing.

Serves POST /v1/chat/completions (SSE streaming) on 127.0.0.1:18321 and logs
every request line + headers to <logfile> so the test can assert
  - the agent's request actually arrived (auth header present), and
  - no x-grok-* tracking headers leaked to the third-party endpoint.

Usage: provider_mock.py <logfile>
"""
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

HOST, PORT = "127.0.0.1", 18321
LOG = sys.argv[1]


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # silence default stderr log
        pass

    def _record(self, note):
        with open(LOG, "a") as f:
            f.write(note + "\n")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8", errors="replace")
        self._record(f"POST {self.path}")
        for k, v in self.headers.items():
            self._record(f"HDR {k}: {v}")
        self._record(f"BODY {body[:2000]}")
        self._record("---")

        if self.path.rstrip("/").endswith("chat/completions"):
            self._send_sse()
        else:
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()

    def _send_sse(self):
        chunks = [
            {
                "id": "chatcmpl-mock",
                "object": "chat.completion.chunk",
                "created": 1,
                "model": "smoke-model-1",
                "choices": [
                    {
                        "index": 0,
                        "delta": {"role": "assistant", "content": "mock-"},
                        "finish_reason": None,
                    }
                ],
            },
            {
                "id": "chatcmpl-mock",
                "object": "chat.completion.chunk",
                "created": 1,
                "model": "smoke-model-1",
                "choices": [
                    {"index": 0, "delta": {"content": "reply"}, "finish_reason": None}
                ],
            },
            {
                "id": "chatcmpl-mock",
                "object": "chat.completion.chunk",
                "created": 1,
                "model": "smoke-model-1",
                "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
            },
        ]
        payload = "".join(f"data: {json.dumps(c)}\n\n" for c in chunks)
        payload += "data: [DONE]\n\n"
        body = payload.encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


if __name__ == "__main__":
    HTTPServer((HOST, PORT), Handler).serve_forever()
