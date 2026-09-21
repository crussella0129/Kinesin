"""Loopback-only live provenance relay; never generates or rewrites a request."""

import hashlib
import http.client
import json
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

root = Path(__file__).resolve().parent
settings = json.loads((root / "runtime.json").read_text(encoding="utf-8"))


class Relay(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def do_POST(self):
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return
        control = json.loads((root / "active.json").read_text(encoding="utf-8"))
        out = Path(control["evidence"])
        length = int(self.headers.get("Content-Length", "0"))
        if not 0 < length <= 2 * 1024 * 1024:
            self.send_error(413)
            return
        body = self.rfile.read(length)
        index = len(list(out.glob("wire-*-request.json")))
        request_path = out / f"wire-{index:02d}-request.json"
        request_path.write_bytes(body)
        expected = control.get("expected_first_sha256")
        digest = hashlib.sha256(body).hexdigest()
        if index == 0 and expected and digest != expected:
            (out / "wire-mismatch.json").write_text(
                json.dumps({"expected": expected, "actual": digest}), encoding="utf-8"
            )
            print(f"REJECTED first wire mismatch: {out.name}", flush=True)
            self.send_error(409, "first wire identity mismatch; inference not forwarded")
            return
        started = time.time()
        connection = http.client.HTTPConnection("127.0.0.1", settings["model_port"], timeout=190)
        try:
            connection.request("POST", self.path, body=body, headers={"Content-Type": "application/json"})
            response = connection.getresponse()
            data = response.read(4 * 1024 * 1024 + 1)
            if len(data) > 4 * 1024 * 1024:
                raise ValueError("response exceeds relay bound")
            (out / f"wire-{index:02d}-response.json").write_bytes(data)
            (out / f"wire-{index:02d}-timing.json").write_text(
                json.dumps({"started_unix": started, "elapsed_s": time.time() - started, "status": response.status, "request_sha256": digest}),
                encoding="utf-8",
            )
            self.send_response(response.status)
            self.send_header("Content-Type", response.getheader("Content-Type", "application/json"))
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            print(f"{out.parent.name} model exchange {index}: HTTP {response.status}, {time.time()-started:.2f}s", flush=True)
        except Exception as error:
            (out / f"wire-{index:02d}-error.txt").write_text(str(error), encoding="utf-8")
            self.send_error(502, "owned model relay failed")
        finally:
            connection.close()


print(f"relay ready on 127.0.0.1:{settings['relay_port']}", flush=True)
ThreadingHTTPServer(("127.0.0.1", settings["relay_port"]), Relay).serve_forever()
