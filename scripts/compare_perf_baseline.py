#!/usr/bin/env python3
"""Compare current perf snapshot output against checked-in baselines."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from statistics import median
from typing import Any, cast

ROOT = Path(__file__).resolve().parents[1]
BASELINES = {
    "encode": ROOT / "perf" / "baselines" / "encode_deep_snapshot_baseline.json",
    "decode": ROOT / "perf" / "baselines" / "decode_snapshot_baseline.json",
}

C1_ABSOLUTE_SLACK_MS = 0.025


def run_snapshot(scenario: str, warmups: int, samples: int, timeout: int) -> dict[str, Any]:
    command = [
        "cargo",
        "run",
        "--release",
        "--bin",
        "qs_perf",
        "--",
        "--scenario",
        scenario,
        "--format",
        "json",
        "--warmups",
        str(warmups),
        "--samples",
        str(samples),
    ]
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    payload = json.loads(completed.stdout)
    if not isinstance(payload, dict):
        raise SystemExit(f"{scenario}: qs_perf returned non-object JSON")
    return cast(dict[str, Any], payload)


def validate_run_shapes(scenario: str, runs: list[dict[str, Any]]) -> None:
    series = [run[scenario] for run in runs]
    expected = series[0]

    for index, baseline_entry in enumerate(expected):
        for current_run in series[1:]:
            current_entry = current_run[index]
            if baseline_entry.keys() != current_entry.keys():
                raise SystemExit(f"{scenario}: entry {index} schema changed across repetitions")
            for key, expected_value in baseline_entry.items():
                if key == "ms_per_op":
                    continue
                actual_value = current_entry[key]
                if actual_value != expected_value:
                    raise SystemExit(
                        f"{scenario}: entry {index} field {key!r} changed across repetitions "
                        f"({expected_value!r} != {actual_value!r})"
                    )


def aggregate_runs(
    scenario: str,
    runs: list[dict[str, Any]],
    repetitions: int,
) -> dict[str, Any]:
    if len(runs) != repetitions:
        raise SystemExit(f"{scenario}: expected {repetitions} runs, got {len(runs)} while comparing")

    series = [run[scenario] for run in runs]
    expected_len = len(series[0])
    if any(len(entries) != expected_len for entries in series[1:]):
        raise SystemExit(f"{scenario}: entry count changed across repetitions")

    validate_run_shapes(scenario, runs)

    aggregated_entries = []
    for index, baseline_entry in enumerate(series[0]):
        aggregated = dict(baseline_entry)
        aggregated["ms_per_op"] = median(float(entries[index]["ms_per_op"]) for entries in series)
        aggregated_entries.append(aggregated)

    return {scenario: aggregated_entries}


def allowed_threshold(
    kind: str,
    name: str,
    expected_ms: float,
    tolerance_pct: float,
) -> float:
    allowed = expected_ms * (1.0 + tolerance_pct / 100.0)
    if kind == "decode" and name == "C1":
        allowed = max(allowed, expected_ms + C1_ABSOLUTE_SLACK_MS)
    return allowed


def compare_entries(kind: str, baseline: list[dict], current: list[dict], tolerance_pct: float) -> None:
    if len(baseline) != len(current):
        raise SystemExit(f"{kind}: baseline entry count {len(baseline)} != current {len(current)}")

    for expected, actual in zip(baseline, current):
        name = expected.get("name", expected.get("depth"))
        expected_ms = float(expected["ms_per_op"])
        actual_ms = float(actual["ms_per_op"])
        allowed = allowed_threshold(kind, str(name), expected_ms, tolerance_pct)
        if actual_ms > allowed:
            raise SystemExit(
                f"{kind} {name}: {actual_ms:.6f} ms/op exceeds baseline {expected_ms:.6f} by more than {tolerance_pct:.1f}%"
            )

        for metric in ("length", "keys"):
            if metric in expected and expected[metric] != actual.get(metric):
                raise SystemExit(f"{kind} {name}: {metric} changed from {expected[metric]} to {actual.get(metric)}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Compare qs_rust perf snapshots to committed baselines.")
    parser.add_argument("--scenario", choices=("encode", "decode", "all"), default="all")
    parser.add_argument("--tolerance-pct", type=float, default=20.0)
    parser.add_argument("--warmups", type=int, default=5)
    parser.add_argument("--samples", type=int, default=7)
    parser.add_argument("--timeout", type=int, default=300, help="seconds to wait for each qs_perf run")
    parser.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()

    if args.repetitions <= 0:
        raise SystemExit("--repetitions must be > 0")

    scenarios = ("encode", "decode") if args.scenario == "all" else (args.scenario,)
    for scenario in scenarios:
        baseline_path = BASELINES[scenario]
        baseline = json.loads(baseline_path.read_text())
        if baseline.get("status") == "pending":
            raise SystemExit(f"{scenario}: baseline capture is pending: {baseline.get('reason', 'no reason recorded')}")
        try:
            current_runs = [
                run_snapshot(scenario, args.warmups, args.samples, args.timeout) for _ in range(args.repetitions)
            ]
        except subprocess.TimeoutExpired as exc:
            raise SystemExit(
                f"{scenario}: qs_perf timed out after {args.timeout}s while running {' '.join(exc.cmd)}"
            ) from exc
        current = aggregate_runs(scenario, current_runs, args.repetitions)
        compare_entries(scenario, baseline[scenario], current[scenario], args.tolerance_pct)
        print(f"{scenario}: OK")


if __name__ == "__main__":
    main()
