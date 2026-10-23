"""Real loopback HTTP/process integration; does not establish container isolation."""
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import unittest
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
LAB = ROOT / "hunting-ground/containers/lab.py"
HOUND = ROOT / "target/debug/hound"


def http(url, data=None):
    with urllib.request.urlopen(url, data=data, timeout=3) as response:
        return json.load(response)


class FirstScent(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="hound001a-")
        self.addCleanup(self.temp.cleanup)
        self.truth = Path(self.temp.name) / "ground-truth.json"
        self.evidence = Path(self.temp.name) / "observations.jsonl"
        # Reserve all chosen ports until the complete set is allocated.
        sockets = [socket.socket() for _ in range(3)]
        for stream in sockets:
            stream.bind(("127.0.0.1", 0))
        ports = [stream.getsockname()[1] for stream in sockets]
        for stream in sockets:
            stream.close()
        self.urls = {node: f"http://127.0.0.1:{port}" for node, port in zip(("node-c", "node-d", "node-e"), ports)}
        self.processes = {}
        for node in ("node-e", "node-d", "node-c"):
            args = ["python3", str(LAB), "serve", "--node", node, "--bind", "127.0.0.1", "--port", self.urls[node].rsplit(":", 1)[1]]
            next_node = {"node-c": "node-d", "node-d": "node-e"}.get(node)
            if next_node:
                args += ["--next", self.urls[next_node] + "/ingest"]
            process = subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
            self.processes[node] = process
            self.addCleanup(self.stop, process)
            deadline = time.monotonic() + 5
            while True:
                try:
                    http(self.urls[node] + "/health")
                    break
                except OSError:
                    if process.poll() is not None or time.monotonic() > deadline:
                        self.fail(f"{node} did not become healthy")
                    time.sleep(0.02)

    @staticmethod
    def stop(process):
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=3)
        if process.stderr:
            process.stderr.close()

    def run_lab(self, *args, check=True):
        return subprocess.run(["python3", str(LAB), *args, "--truth", str(self.truth)], capture_output=True, text=True, check=check, timeout=10)

    def sniff(self, sensors=None, check=True):
        args = [str(HOUND), "sniff", "--output", str(self.evidence)]
        for node, url in (sensors or self.urls).items():
            args += ["--sensor", f"{node}={url}/observations"]
        return subprocess.run(args, capture_output=True, text=True, check=check, timeout=15)

    def traffic(self):
        self.run_lab("init")
        for node in ("node-a", "node-b"):
            self.run_lab("source", "--node", node, "--next", self.urls["node-c"] + "/ingest")

    def test_real_traffic_journal_provenance_deduplication_and_evaluator(self):
        for url in self.urls.values():
            self.assertEqual(http(url + "/observations")["observations"], [])
        self.traffic()
        result = self.sniff()
        self.assertIn("recorded 18 new records", result.stdout)
        contents = self.evidence.read_bytes()
        records = [json.loads(line) for line in contents.splitlines()]
        self.assertEqual(len(records), 18)
        for node, url in self.urls.items():
            received = http(url + "/observations")["observations"]
            collected = [r for r in records if r["observation"]["sensor_id"] == node]
            self.assertEqual([r["observation"] for r in collected], received)
            self.assertEqual(len(received), 6)
            for record in collected:
                self.assertEqual(record["collection"]["sensor_url"], url + "/observations")
                self.assertGreaterEqual(record["collection"]["collected_at_unix_ms"], record["observation"]["timestamp_unix_ms"])
        self.assertIn("recorded 0 new records", self.sniff().stdout)
        self.assertEqual(self.evidence.read_bytes(), contents)
        report = json.loads(self.run_lab("evaluate", "--evidence", str(self.evidence)).stdout)
        self.assertTrue(report["delivery_complete"])
        self.assertEqual(report["observed_destination_messages"], 6)
        self.assertFalse(report["attribution_evaluated"])

    def test_ground_truth_routes_absent_and_sensor_is_read_only(self):
        self.run_lab("init")
        for url in self.urls.values():
            for path in ("/ground-truth", "/scenario/ground-truth.json", "/evaluate", "/observations?file=secret", "/../scenario/ground-truth.json"):
                with self.assertRaises(urllib.error.HTTPError) as caught:
                    http(url + path)
                self.assertEqual(caught.exception.code, 404)
                caught.exception.close()
            with self.assertRaises(urllib.error.HTTPError) as caught:
                http(url + "/observations", b"{}")
            self.assertEqual(caught.exception.code, 405)
            caught.exception.close()
            with self.assertRaises(urllib.error.HTTPError) as caught:
                http(url + "/ingest", json.dumps({"message_id": "m", "operation": "probe", "intrusion_origin": "node-a"}).encode())
            self.assertEqual(caught.exception.code, 400)
            caught.exception.close()
            self.assertEqual(http(url + "/observations")["observations"], [])

    def test_failed_sensor_does_not_create_a_partial_journal(self):
        self.traffic()
        result = self.sniff({"node-c": self.urls["node-c"], "wrong-sensor": self.urls["node-d"]}, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match", result.stderr)
        self.assertFalse(self.evidence.exists())

    def test_failed_forwarding_retains_receipts_but_evaluation_fails(self):
        self.stop(self.processes["node-e"])
        self.run_lab("init")
        source = self.run_lab("source", "--node", "node-a", "--next", self.urls["node-c"] + "/ingest", check=False)
        self.assertNotEqual(source.returncode, 0)
        self.sniff({node: self.urls[node] for node in ("node-c", "node-d")})
        self.assertEqual(len(self.evidence.read_text().splitlines()), 2)
        result = self.run_lab("evaluate", "--evidence", str(self.evidence), check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(json.loads(result.stdout)["delivery_complete"])

    def test_empty_environment_creates_no_fabricated_observations(self):
        self.assertIn("Retrieved 0 observations", self.sniff().stdout)
        self.assertEqual(self.evidence.read_text(), "")


if __name__ == "__main__":
    if not HOUND.is_file():
        raise SystemExit("Build the CLI first: cargo build --locked -p hound-cli")
    unittest.main()
