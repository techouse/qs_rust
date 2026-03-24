import json
import tempfile
import sys
import unittest
from argparse import Namespace
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import cross_port_perf


class SpecMissingRowsReasonTests(unittest.TestCase):
    def test_decode_spec_requires_all_canonical_cases(self) -> None:
        spec = cross_port_perf.CommandSpec("kotlin", "decode", ROOT, ["dummy"])
        reason = cross_port_perf.spec_missing_rows_reason(
            spec,
            [],
            [{"case": "C1"}, {"case": "C2"}],
        )
        self.assertEqual(reason, "missing canonical decode cases: C3")

    def test_all_spec_requires_encode_rows_and_full_decode_matrix(self) -> None:
        spec = cross_port_perf.CommandSpec("kotlin", "all", ROOT, ["dummy"])
        reason = cross_port_perf.spec_missing_rows_reason(
            spec,
            [{"depth": 2000}],
            [{"case": "C1"}, {"case": "C2"}],
        )
        self.assertEqual(reason, "missing canonical decode cases: C3")

    def test_all_spec_passes_with_encode_rows_and_full_decode_matrix(self) -> None:
        spec = cross_port_perf.CommandSpec("kotlin", "all", ROOT, ["dummy"])
        reason = cross_port_perf.spec_missing_rows_reason(
            spec,
            [{"depth": 2000}],
            [{"case": "C1"}, {"case": "C2"}, {"case": "C3"}],
        )
        self.assertIsNone(reason)

    def test_encode_spec_only_requires_encode_rows(self) -> None:
        spec = cross_port_perf.CommandSpec("rust", "encode", ROOT, ["dummy"])
        reason = cross_port_perf.spec_missing_rows_reason(
            spec,
            [{"depth": 2000}],
            [],
        )
        self.assertIsNone(reason)


class PathNormalizationTests(unittest.TestCase):
    def test_abbreviate_home_rewrites_home_prefixed_paths(self) -> None:
        with mock.patch.object(cross_port_perf, "HOME_DIR", Path("/Users/example")):
            self.assertEqual(
                cross_port_perf.abbreviate_home("/Users/example/Work/qs.py"),
                "~/Work/qs.py",
            )
            self.assertEqual(
                cross_port_perf.abbreviate_home("in /Users/example/Work/QsNet"),
                "in ~/Work/QsNet",
            )

    def test_sibling_path_prefers_env_override(self) -> None:
        with mock.patch.dict("os.environ", {"QS_PYTHON_REPO": "~/src/custom_qs.py"}, clear=False):
            self.assertEqual(
                cross_port_perf.sibling_path("QS_PYTHON_REPO", "qs.py"),
                Path("~/src/custom_qs.py").expanduser(),
            )


class SnapshotRetentionTests(unittest.TestCase):
    def test_partial_coverage_keeps_parseable_rows(self) -> None:
        spec = cross_port_perf.CommandSpec("kotlin", "all", ROOT, ["dummy"])
        run_result = {
            "language": "kotlin",
            "scenario": "all",
            "cwd": str(ROOT),
            "command": ["dummy"],
            "returncode": 0,
            "stdout": "ignored",
            "stderr": "",
            "timed_out": False,
        }
        encode_entries = [
            {
                "language": "kotlin",
                "runtime": "kotlin",
                "depth": 2000,
                "ms_per_op": 1.25,
                "length": 6006,
                "alloc_bytes_per_op": None,
            }
        ]
        decode_entries = [
            {
                "language": "kotlin",
                "runtime": "kotlin",
                "case": "C1",
                "count": 100,
                "comma": False,
                "utf8": False,
                "value_len": 8,
                "ms_per_op": 0.5,
                "keys": 100,
                "alloc_bytes_per_op": None,
            },
            {
                "language": "kotlin",
                "runtime": "kotlin",
                "case": "C2",
                "count": 1000,
                "comma": False,
                "utf8": False,
                "value_len": 40,
                "ms_per_op": 1.0,
                "keys": 1000,
                "alloc_bytes_per_op": None,
            },
        ]

        with tempfile.TemporaryDirectory() as tempdir:
            json_output = Path(tempdir) / "snapshot.json"
            markdown_output = Path(tempdir) / "snapshot.md"

            with mock.patch.object(
                cross_port_perf,
                "parse_args",
                return_value=Namespace(json_output=json_output, markdown_output=markdown_output, timeout=30),
            ):
                with mock.patch.object(cross_port_perf, "all_specs", return_value=[spec]):
                    with mock.patch.object(cross_port_perf, "run_command", return_value=run_result):
                        with mock.patch.object(
                            cross_port_perf,
                            "parse_snapshot_output",
                            return_value=(encode_entries, decode_entries),
                        ):
                            cross_port_perf.main()

            snapshot = json.loads(json_output.read_text())
            self.assertEqual(snapshot["status"], "partial")
            self.assertEqual(snapshot["encode"], encode_entries)
            self.assertEqual(snapshot["decode"], decode_entries)
            self.assertEqual(
                snapshot["failures"],
                [
                    {
                        "language": "kotlin",
                        "scenario": "all",
                        "reason": "coverage failed: missing canonical decode cases: C3",
                    }
                ],
            )
            markdown = markdown_output.read_text()
            self.assertIn("| kotlin | kotlin | 2000 | 1.250 | 6006 | n/a |", markdown)
            self.assertIn("| kotlin | kotlin | C1 | 100 | false | false | 8 | 0.500 | 100 | n/a |", markdown)


if __name__ == "__main__":
    unittest.main()
