#!/usr/bin/env python3
"""Capture local Rust perf baselines from qs_perf JSON output."""

from __future__ import annotations

import argparse
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from statistics import median
from typing import Any, cast

ROOT = Path(__file__).resolve().parents[1]
BASELINES = {
    "encode": ROOT / "perf" / "baselines" / "encode_deep_snapshot_baseline.json",
    "decode": ROOT / "perf" / "baselines" / "decode_snapshot_baseline.json",
}


def snapshot_command(scenario: str, warmups: int, samples: int) -> list[str]:
    return [
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


def write_baseline(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def describe_capture_failure(scenario: str, command: list[str], exc: BaseException) -> str:
    if isinstance(exc, subprocess.TimeoutExpired):
        return f"{scenario}: qs_perf timed out after {exc.timeout}s while running {' '.join(command)}"
    if isinstance(exc, subprocess.CalledProcessError):
        return f"{scenario}: qs_perf exited with status {exc.returncode} while running {' '.join(command)}"
    if isinstance(exc, json.JSONDecodeError):
        return f"{scenario}: qs_perf returned invalid JSON: {exc.msg}"
    return str(exc)


def pending_baseline(
    scenario: str,
    command: list[str],
    reason: str,
    repetitions: int,
    warmups: int,
    samples: int,
    timeout: int,
) -> dict[str, Any]:
    return {
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "command": command,
        "reason": reason,
        "repetitions": repetitions,
        "samples": samples,
        "scenario": scenario,
        "status": "pending",
        "timeout_seconds": timeout,
        "warmups": warmups,
    }


def run_snapshot(scenario: str, warmups: int, samples: int, timeout: int) -> dict[str, Any]:
    command = snapshot_command(scenario, warmups, samples)
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
    payload = cast(dict[str, Any], payload)
    payload["captured_at"] = datetime.now(timezone.utc).isoformat()
    payload["command"] = command
    return payload


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
    warmups: int,
    samples: int,
) -> dict[str, Any]:
    if len(runs) != repetitions:
        raise SystemExit(f"{scenario}: expected {repetitions} runs, got {len(runs)} during capture")

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

    return {
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "command": runs[0]["command"],
        "repetitions": repetitions,
        "aggregator": "median",
        scenario: aggregated_entries,
    }


def capture_scenario(
    scenario: str,
    repetitions: int,
    warmups: int,
    samples: int,
    timeout: int,
) -> None:
    command = snapshot_command(scenario, warmups, samples)
    try:
        runs = [run_snapshot(scenario, warmups, samples, timeout) for _ in range(repetitions)]
        baseline = aggregate_runs(
            scenario,
            runs,
            repetitions=repetitions,
            warmups=warmups,
            samples=samples,
        )
    except (SystemExit, Exception) as exc:
        reason = describe_capture_failure(scenario, command, exc)
        write_baseline(
            BASELINES[scenario],
            pending_baseline(
                scenario,
                command,
                reason,
                repetitions=repetitions,
                warmups=warmups,
                samples=samples,
                timeout=timeout,
            ),
        )
        raise SystemExit(reason) from exc

    write_baseline(BASELINES[scenario], baseline)
    print(f"{scenario}: wrote {BASELINES[scenario]}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Capture qs_rust perf baselines from qs_perf.")
    parser.add_argument("--scenario", choices=("encode", "decode", "all"), default="all")
    parser.add_argument("--warmups", type=int, default=5)
    parser.add_argument("--samples", type=int, default=7)
    parser.add_argument("--timeout", type=int, default=1800)
    parser.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()

    if args.repetitions <= 0:
        raise SystemExit("--repetitions must be > 0")

    scenarios = ("encode", "decode") if args.scenario == "all" else (args.scenario,)
    for scenario in scenarios:
        capture_scenario(
            scenario,
            repetitions=args.repetitions,
            warmups=args.warmups,
            samples=args.samples,
            timeout=args.timeout,
        )


if __name__ == "__main__":
    main()
