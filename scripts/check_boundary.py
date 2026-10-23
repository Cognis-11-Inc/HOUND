"""Run inside HOUND's exact Compose image/network/volume context."""
import json
import os
from pathlib import Path
import socket
import urllib.error
import urllib.request


def main():
    assert os.getuid() == 10001, "HOUND is not running as the configured non-root user"
    assert "HOUND_SCENARIO_SEED" not in os.environ, "private seed leaked into HOUND"
    status = dict(line.split(":", 1) for line in Path("/proc/self/status").read_text().splitlines() if ":" in line)
    assert int(status["CapEff"].strip(), 16) == 0, "effective capabilities remain"
    assert status["NoNewPrivs"].strip() == "1", "no-new-privileges is disabled"
    root_mount = next(line.split() for line in Path("/proc/mounts").read_text().splitlines() if line.split()[1] == "/")
    assert "ro" in root_mount[3].split(","), "root filesystem is writable"
    assert not Path("/scenario").exists(), "private scenario is visible to HOUND"
    assert not Path("/var/run/docker.sock").exists(), "Docker socket is visible"
    for node in ("node-c", "node-d", "node-e"):
        base = f"http://{node}:8080"
        with urllib.request.urlopen(base + "/observations", timeout=3) as response:
            batch = json.load(response)
        assert set(batch) == {"schema_version", "observations"}
        for observation in batch["observations"]:
            assert set(observation) == {"observation_id", "sensor_id", "timestamp_unix_ms",
                                        "event", "source_endpoint", "destination_endpoint", "provenance"}
            assert set(observation["event"]) == {"protocol", "method", "path", "message_id", "operation", "payload_bytes"}
        for path in ("/ground-truth", "/scenario/ground-truth.json", "/evaluate",
                     "/observations?file=/scenario/ground-truth.json", "/../scenario/ground-truth.json"):
            try:
                urllib.request.urlopen(base + path, timeout=3)
            except urllib.error.HTTPError as error:
                assert error.code == 404
                error.close()
            else:
                raise AssertionError(f"hidden interface was readable: {node}{path}")
        request = urllib.request.Request(base + "/observations", data=b"{}", method="POST")
        try:
            urllib.request.urlopen(request, timeout=3)
        except urllib.error.HTTPError as error:
            assert error.code == 405
            error.close()
        else:
            raise AssertionError("sensor observation interface is writable")
    try:
        addresses = socket.getaddrinfo("evaluator", 8080, type=socket.SOCK_STREAM)
    except socket.gaierror:
        addresses = []
    for family, kind, protocol, _, address in addresses:
        with socket.socket(family, kind, protocol) as connection:
            connection.settimeout(1)
            assert connection.connect_ex(address) != 0, "evaluator reachable from HOUND"
    print("Boundary checks passed in the HOUND container context.")


if __name__ == "__main__":
    main()
