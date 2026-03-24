import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import capture_perf_baselines


class CaptureScenarioTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.baselines = {
            "encode": Path(self.tempdir.name) / "encode.json",
            "decode": Path(self.tempdir.name) / "decode.json",
        }
        self.original_baselines = capture_perf_baselines.BASELINES
        capture_perf_baselines.BASELINES = self.baselines

    def tearDown(self) -> None:
        capture_perf_baselines.BASELINES = self.original_baselines
        self.tempdir.cleanup()

    def test_timeout_overwrites_stale_baseline_with_pending_state(self) -> None:
        stale = {"captured_at": "old", "encode": [{"depth": 2000, "ms_per_op": 0.1}]}
        self.baselines["encode"].write_text(json.dumps(stale))

        with mock.patch.object(
            capture_perf_baselines,
            "run_snapshot",
            side_effect=subprocess.TimeoutExpired(cmd=["cargo", "run"], timeout=12),
        ):
            with self.assertRaises(SystemExit) as ctx:
                capture_perf_baselines.capture_scenario(
                    "encode",
                    repetitions=3,
                    warmups=5,
                    samples=7,
                    timeout=12,
                )

        self.assertIn("timed out after 12s", str(ctx.exception))
        payload = json.loads(self.baselines["encode"].read_text())
        self.assertEqual(payload["status"], "pending")
        self.assertIn("timed out after 12s", payload["reason"])
        self.assertEqual(payload["command"], capture_perf_baselines.snapshot_command("encode", 5, 7))
        self.assertNotIn("encode", payload)

    def test_non_zero_exit_overwrites_stale_baseline_with_pending_state(self) -> None:
        stale = {"captured_at": "old", "decode": [{"name": "C1", "ms_per_op": 0.1}]}
        self.baselines["decode"].write_text(json.dumps(stale))

        with mock.patch.object(
            capture_perf_baselines,
            "run_snapshot",
            side_effect=subprocess.CalledProcessError(101, ["cargo", "run"]),
        ):
            with self.assertRaises(SystemExit) as ctx:
                capture_perf_baselines.capture_scenario(
                    "decode",
                    repetitions=3,
                    warmups=5,
                    samples=7,
                    timeout=30,
                )

        self.assertIn("exited with status 101", str(ctx.exception))
        payload = json.loads(self.baselines["decode"].read_text())
        self.assertEqual(payload["status"], "pending")
        self.assertIn("exited with status 101", payload["reason"])
        self.assertNotIn("decode", payload)

    def test_invalid_json_overwrites_stale_baseline_with_pending_state(self) -> None:
        stale = {"captured_at": "old", "encode": [{"depth": 2000, "ms_per_op": 0.1}]}
        self.baselines["encode"].write_text(json.dumps(stale))

        with mock.patch.object(
            capture_perf_baselines,
            "run_snapshot",
            side_effect=json.JSONDecodeError("Expecting value", "not-json", 0),
        ):
            with self.assertRaises(SystemExit) as ctx:
                capture_perf_baselines.capture_scenario(
                    "encode",
                    repetitions=2,
                    warmups=5,
                    samples=7,
                    timeout=30,
                )

        self.assertIn("returned invalid JSON", str(ctx.exception))
        payload = json.loads(self.baselines["encode"].read_text())
        self.assertEqual(payload["status"], "pending")
        self.assertIn("returned invalid JSON", payload["reason"])

    def test_successful_capture_writes_aggregated_baseline(self) -> None:
        command = capture_perf_baselines.snapshot_command("encode", 5, 7)
        runs = [
            {
                "command": command,
                "captured_at": "first",
                "encode": [{"depth": 2000, "iterations": 20, "length": 6006, "ms_per_op": 3.0}],
            },
            {
                "command": command,
                "captured_at": "second",
                "encode": [{"depth": 2000, "iterations": 20, "length": 6006, "ms_per_op": 1.0}],
            },
            {
                "command": command,
                "captured_at": "third",
                "encode": [{"depth": 2000, "iterations": 20, "length": 6006, "ms_per_op": 2.0}],
            },
        ]

        with mock.patch.object(capture_perf_baselines, "run_snapshot", side_effect=runs):
            with mock.patch("builtins.print"):
                capture_perf_baselines.capture_scenario(
                    "encode",
                    repetitions=3,
                    warmups=5,
                    samples=7,
                    timeout=30,
                )

        payload = json.loads(self.baselines["encode"].read_text())
        self.assertNotIn("status", payload)
        self.assertEqual(payload["aggregator"], "median")
        self.assertEqual(payload["command"], command)
        self.assertEqual(payload["encode"][0]["ms_per_op"], 2.0)


if __name__ == "__main__":
    unittest.main()
