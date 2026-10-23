"""HOUND-001A's controlled HTTP terrain and private evaluator. Stdlib only."""

import argparse
import copy
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import random
import threading
import time
from urllib.parse import urlsplit
import uuid

MAX_BODY = 4096
MAX_EVENTS = 1000


def post_message(url, message):
    """Forward to a configured next hop, never a destination supplied by traffic."""
    target = urlsplit(url)
    if target.scheme != "http" or target.path != "/ingest" or not target.hostname:
        raise ValueError("invalid laboratory next hop")
    connection = http.client.HTTPConnection(target.hostname, target.port, timeout=3)
    try:
        connection.request("POST", "/ingest", json.dumps(message).encode(),
                           {"Content-Type": "application/json"})
        response = connection.getresponse()
        response.read(MAX_BODY)
        if response.status != 200:
            raise RuntimeError(f"next hop returned {response.status}")
    finally:
        connection.close()


class TerrainServer(ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, address, node_id, next_hop=None):
        super().__init__(address, TerrainHandler)
        self.node_id = node_id
        self.next_hop = next_hop
        self.instance = uuid.uuid4().hex
        self.observations = []
        self.lock = threading.Lock()


class TerrainHandler(BaseHTTPRequestHandler):
    # HTTP/1.0 plus Content-Length gives the small Rust client bounded framing.
    def log_message(self, *_args):
        pass

    def setup(self):
        super().setup()
        self.connection.settimeout(3)

    def respond(self, status, value):
        body = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            self.respond(200, {"status": "ready"})
        elif self.path == "/observations":
            with self.server.lock:
                observations = copy.deepcopy(self.server.observations)
            self.respond(200, {"schema_version": 1, "observations": observations})
        else:
            self.respond(404, {"error": "unknown read-only interface"})

    def do_POST(self):
        if self.path != "/ingest":
            self.respond(405, {"error": "only traffic ingestion accepts POST"})
            return
        try:
            if self.headers.get("Transfer-Encoding") or len(self.headers.get_all("Content-Length", [])) != 1:
                raise ValueError("invalid body framing")
            length = int(self.headers["Content-Length"])
            if not 0 < length <= MAX_BODY:
                raise ValueError("body size limit")
            raw = self.rfile.read(length)
            message = json.loads(raw)
            if set(message) != {"message_id", "operation"}:
                raise ValueError("unexpected traffic fields")
            message_id = message["message_id"]
            if (not isinstance(message_id, str) or not 1 <= len(message_id) <= 128
                    or not all(c.isascii() and (c.isalnum() or c in "-_.:") for c in message_id)
                    or message["operation"] not in ("heartbeat", "probe", "transfer")):
                raise ValueError("invalid synthetic traffic")
        except (ValueError, TypeError, KeyError):
            self.respond(400, {"error": "invalid synthetic message"})
            return
        # Endpoints come from the actual accepted socket, not sender assertions.
        with self.server.lock:
            sequence = len(self.server.observations) + 1
            if sequence > MAX_EVENTS:
                self.respond(503, {"error": "sensor capacity reached; reset the lab"})
                return
            self.server.observations.append({
                "observation_id": f"{self.server.node_id}:{self.server.instance}:{sequence}",
                "sensor_id": self.server.node_id,
                "timestamp_unix_ms": time.time_ns() // 1_000_000,
                "event": {"protocol": "http", "method": "POST", "path": "/ingest",
                          "message_id": message_id, "operation": message["operation"],
                          "payload_bytes": len(raw)},
                "source_endpoint": {"address": self.client_address[0], "port": self.client_address[1]},
                "destination_endpoint": {"address": self.connection.getsockname()[0],
                                         "port": self.connection.getsockname()[1]},
                "provenance": {"capture_method": "http_server_receive",
                               "sensor_instance": self.server.instance, "sequence": sequence},
            })
        if self.server.next_hop:
            try:
                post_message(self.server.next_hop, message)
            except (OSError, RuntimeError):
                # The receipt remains evidence even if subsequent forwarding fails.
                self.respond(502, {"error": "forwarding failed; receipt retained"})
                return
        self.respond(200, {"received": True})


def initialize(path, seed=None):
    if path.exists():
        print("Using existing private scenario; remove lab volumes for a fresh run.")
        return
    rng = random.Random(seed) if seed is not None else random.SystemRandom()
    origin = rng.choice(["node-a", "node-b"])
    # Roles live only in this private file. Neither names nor payloads carry a role label.
    scenario = {"schema_version": 1, "scenario_id": uuid.uuid4().hex,
                "intrusion_origin": origin, "messages": []}
    for node in ("node-a", "node-b"):
        operations = ["probe", "probe", "transfer"] if node == origin else ["heartbeat"] * 3
        for operation in operations:
            scenario["messages"].append({"source": node, "message_id": f"{rng.getrandbits(128):032x}",
                                         "operation": operation})
    with path.open("x", encoding="utf-8") as output:
        json.dump(scenario, output)
    path.chmod(0o600)
    print("Private scenario initialized.")


def evaluate(truth_path, evidence_path):
    truth = json.loads(truth_path.read_text())
    records = [json.loads(line) for line in evidence_path.read_text().splitlines()]
    expected = {message["message_id"] for message in truth["messages"]}
    observed = {r["observation"]["event"]["message_id"] for r in records
                if r["observation"]["sensor_id"] == "node-e"}
    report = {"scenario_id": truth["scenario_id"], "observation_count": len(records),
              "expected_destination_messages": len(expected),
              "observed_destination_messages": len(expected & observed),
              "delivery_complete": expected <= observed, "attribution_evaluated": False}
    print(json.dumps(report, indent=2))
    if not report["delivery_complete"]:
        raise RuntimeError("not all scenario messages were observed at the destination")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["serve", "init", "source", "evaluate"])
    parser.add_argument("--node")
    parser.add_argument("--next")
    parser.add_argument("--bind", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--hold", action="store_true", help="keep the source host alive after sending")
    parser.add_argument("--truth", type=Path, default=Path("/scenario/ground-truth.json"))
    parser.add_argument("--evidence", type=Path, default=Path("/evidence/observations.jsonl"))
    args = parser.parse_args()
    if args.mode == "serve":
        if args.node not in ("node-c", "node-d", "node-e"):
            parser.error("serve requires a neutral sensor node identifier")
        TerrainServer((args.bind, args.port), args.node, args.next).serve_forever()
    elif args.mode == "init":
        initialize(args.truth, os.getenv("HOUND_SCENARIO_SEED") or None)
    elif args.mode == "source":
        if args.node not in ("node-a", "node-b") or not args.next:
            parser.error("source requires a neutral source identifier and next hop")
        truth = json.loads(args.truth.read_text())
        for message in truth["messages"]:
            if message["source"] == args.node:
                post_message(args.next, {k: message[k] for k in ("message_id", "operation")})
        print(f"{args.node}: synthetic traffic delivered")
        if args.hold:
            Path("/tmp/traffic.done").touch()
            threading.Event().wait()
    else:
        evaluate(args.truth, args.evidence)


if __name__ == "__main__":
    main()
