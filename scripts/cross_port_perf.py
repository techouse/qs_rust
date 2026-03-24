#!/usr/bin/env python3
"""Run and normalize local cross-port perf snapshots."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JSON = ROOT / "perf" / "comparison" / "latest_snapshot.json"
DEFAULT_MD = ROOT / "docs" / "performance_comparison.md"
CANONICAL_DECODE_CASES = frozenset({"C1", "C2", "C3"})
HOME_DIR = Path.home()
DEFAULT_SIBLING_ROOT = Path(os.environ.get("QS_WORK_ROOT", str(HOME_DIR / "Work"))).expanduser()


def sibling_path(env_var: str, relative: str) -> Path:
    override = os.environ.get(env_var)
    if override:
        return Path(override).expanduser()
    return DEFAULT_SIBLING_ROOT / relative


def abbreviate_home(text: str) -> str:
    home = str(HOME_DIR)
    if text == home:
        return "~"
    return text.replace(f"{home}{os.sep}", f"~{os.sep}")

SIBLINGS = {
    "rust": ROOT,
    "python": sibling_path("QS_PYTHON_REPO", "qs.py"),
    "dart": sibling_path("QS_DART_REPO", "qs.dart"),
    "kotlin": sibling_path("QS_KOTLIN_REPO", "qs-kotlin"),
    "swift": sibling_path("QS_SWIFT_REPO", "QsSwift/Bench"),
    "csharp": sibling_path("QS_CSHARP_REPO", "QsNet"),
}


@dataclass(frozen=True)
class CommandSpec:
    language: str
    scenario: str
    cwd: Path
    command: list[str]


def rust_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="rust",
            scenario="encode",
            cwd=ROOT,
            command=[
                "cargo",
                "run",
                "--release",
                "--bin",
                "qs_perf",
                "--",
                "--scenario",
                "encode",
                "--format",
                "json",
            ],
        ),
        CommandSpec(
            language="rust",
            scenario="decode",
            cwd=ROOT,
            command=[
                "cargo",
                "run",
                "--release",
                "--bin",
                "qs_perf",
                "--",
                "--scenario",
                "decode",
                "--format",
                "json",
            ],
        ),
    ]


def python_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="python",
            scenario="encode",
            cwd=SIBLINGS["python"],
            command=[
                "python3",
                "scripts/bench_encode_depth.py",
                "--runs",
                "7",
                "--warmups",
                "5",
            ],
        ),
        CommandSpec(
            language="python",
            scenario="decode",
            cwd=SIBLINGS["python"],
            command=[
                "python3",
                "scripts/bench_decode_snapshot.py",
                "--samples",
                "7",
                "--warmups",
                "5",
            ],
        ),
    ]


def dart_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="dart",
            scenario="encode",
            cwd=SIBLINGS["dart"],
            command=["dart", "run", "tool/perf_snapshot.dart"],
        ),
        CommandSpec(
            language="dart",
            scenario="decode",
            cwd=SIBLINGS["dart"],
            command=["dart", "run", "tool/decode_perf_snapshot.dart"],
        ),
    ]


def kotlin_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="kotlin",
            scenario="all",
            cwd=SIBLINGS["kotlin"],
            command=["./gradlew", ":comparison:run", "--args", "perf"],
        )
    ]


def swift_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="swift",
            scenario="encode",
            cwd=SIBLINGS["swift"],
            command=["swift", "run", "-c", "release", "QsSwiftBench", "perf"],
        ),
        CommandSpec(
            language="swift",
            scenario="decode",
            cwd=SIBLINGS["swift"],
            command=["swift", "run", "-c", "release", "QsSwiftBench", "perf-decode"],
        ),
    ]


def csharp_specs() -> list[CommandSpec]:
    return [
        CommandSpec(
            language="csharp",
            scenario="encode",
            cwd=SIBLINGS["csharp"],
            command=[
                "dotnet",
                "run",
                "-c",
                "Release",
                "--project",
                "benchmarks/QsNet.Benchmarks",
                "--",
                "--filter",
                "*Encode_DeepNesting*",
            ],
        ),
        CommandSpec(
            language="csharp",
            scenario="decode",
            cwd=SIBLINGS["csharp"],
            command=[
                "dotnet",
                "run",
                "-c",
                "Release",
                "--project",
                "benchmarks/QsNet.Benchmarks",
                "--",
                "--filter",
                "*Decode_Public*",
            ],
        ),
    ]


def all_specs() -> list[CommandSpec]:
    return rust_specs() + python_specs() + dart_specs() + kotlin_specs() + swift_specs() + csharp_specs()


ENCODE_LINE_RE = re.compile(r"^\s*depth=\s*(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*len=(\d+)\s*$")
DECODE_LINE_RE = re.compile(
    r"^\s*(?:(C\d+):\s*)?count=\s*(\d+)\s*,\s*comma=\s*(true|false)\s*,\s*utf8=\s*(true|false)\s*,\s*len=\s*(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*keys=(\d+)\s*$",
    re.IGNORECASE,
)
PYTHON_ENCODE_RE = re.compile(r"^depth=(\d+)\s+median=([0-9.]+)s\s+runs=\[")
KOTLIN_ENCODE_RE = re.compile(
    r"^\s*depth=\s*(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*([0-9.na/ ]+)\s*(MiB/op|KiB/op|n/a)\s*\|\s*len=(\d+)\s*$"
)
KOTLIN_DECODE_RE = re.compile(
    r"^\s*count=\s*(\d+),\s*comma=(true|false),\s*utf8=(true|false),\s*len=\s*(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*([0-9.na/ ]+)\s*(MiB/op|KiB/op|n/a)\s*\|\s*keys=(\d+)\s*$"
)
SWIFT_ENCODE_RE = re.compile(r"^\s*(swift|objc)\s+depth=\s*(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*len=(\d+)\s*$")
SWIFT_DECODE_RE = re.compile(
    r"^\s*(swift|objc)-decode\s+(C\d+)\s+count=(\d+)\s+comma=(true|false)\s+utf8=(true|false)\s+len=(\d+):\s*([0-9.]+)\s*ms/op\s*\|\s*keys=(\d+)\s*$"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run and normalize local cross-port perf snapshots.")
    parser.add_argument("--json-output", type=Path, default=DEFAULT_JSON)
    parser.add_argument("--markdown-output", type=Path, default=DEFAULT_MD)
    parser.add_argument("--timeout", type=int, default=1800)
    return parser.parse_args()


def run_command(spec: CommandSpec, timeout: int) -> dict[str, Any]:
    env = os.environ.copy()
    try:
        completed = subprocess.run(
            spec.command,
            cwd=spec.cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=env,
        )
        return {
            "language": spec.language,
            "scenario": spec.scenario,
            "cwd": abbreviate_home(str(spec.cwd)),
            "command": spec.command,
            "returncode": completed.returncode,
            "stdout": abbreviate_home(completed.stdout),
            "stderr": abbreviate_home(completed.stderr),
            "timed_out": False,
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "language": spec.language,
            "scenario": spec.scenario,
            "cwd": abbreviate_home(str(spec.cwd)),
            "command": spec.command,
            "returncode": None,
            "stdout": abbreviate_home(exc.stdout or ""),
            "stderr": abbreviate_home(exc.stderr or ""),
            "timed_out": True,
        }


def parse_bool(value: str) -> bool:
    return value.lower() == "true"


def parse_alloc_to_bytes(value: str, unit: str) -> Optional[int]:
    if unit == "n/a":
        return None
    numeric = float(value.strip())
    if unit == "MiB/op":
        return int(numeric * 1024 * 1024)
    if unit == "KiB/op":
        return int(numeric * 1024)
    return None


def decode_case_name(count: int, comma: bool, utf8: bool, value_len: int) -> str:
    if count == 100 and not comma and not utf8 and value_len == 8:
        return "C1"
    if count == 1000 and not comma and not utf8 and value_len == 40:
        return "C2"
    if count == 1000 and comma and utf8 and value_len == 40:
        return "C3"
    return f"count={count}|comma={str(comma).lower()}|utf8={str(utf8).lower()}|len={value_len}"


def filter_canonical_decode_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [entry for entry in entries if entry["case"] in CANONICAL_DECODE_CASES]


def parse_rust_snapshot_json(
    stdout: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    payload = json.loads(stdout)
    encode = [
        {
            "language": "rust",
            "runtime": "rust",
            "depth": int(entry["depth"]),
            "ms_per_op": float(entry["ms_per_op"]),
            "length": int(entry["length"]),
            "alloc_bytes_per_op": None,
        }
        for entry in payload.get("encode", [])
    ]
    decode = [
        {
            "language": "rust",
            "runtime": "rust",
            "case": entry["name"],
            "count": int(entry["count"]),
            "comma": bool(entry["comma"]),
            "utf8": bool(entry["utf8"]),
            "value_len": int(entry["value_len"]),
            "ms_per_op": float(entry["ms_per_op"]),
            "keys": int(entry["keys"]),
            "alloc_bytes_per_op": None,
        }
        for entry in payload.get("decode", [])
    ]
    return encode, decode


def parse_snapshot_output(
    language: str, scenario: str, stdout: str
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if language == "rust":
        return parse_rust_snapshot_json(stdout)

    encode: list[dict[str, Any]] = []
    decode: list[dict[str, Any]] = []

    for raw_line in stdout.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        if language in {"rust", "dart"}:
            if match := ENCODE_LINE_RE.match(line):
                depth, ms_per_op, length = match.groups()
                encode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "depth": int(depth),
                        "ms_per_op": float(ms_per_op),
                        "length": int(length),
                        "alloc_bytes_per_op": None,
                    }
                )
                continue
            if match := DECODE_LINE_RE.match(line):
                name, count, comma, utf8, value_len, ms_per_op, keys = match.groups()
                decode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "case": name
                        or decode_case_name(
                            int(count),
                            parse_bool(comma),
                            parse_bool(utf8),
                            int(value_len),
                        ),
                        "count": int(count),
                        "comma": parse_bool(comma),
                        "utf8": parse_bool(utf8),
                        "value_len": int(value_len),
                        "ms_per_op": float(ms_per_op),
                        "keys": int(keys),
                        "alloc_bytes_per_op": None,
                    }
                )
                continue

        if language == "python":
            if match := PYTHON_ENCODE_RE.match(line):
                depth, seconds = match.groups()
                encode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "depth": int(depth),
                        "ms_per_op": float(seconds) * 1000.0,
                        "length": None,
                        "alloc_bytes_per_op": None,
                    }
                )
                continue
            if match := DECODE_LINE_RE.match(line):
                name, count, comma, utf8, value_len, ms_per_op, keys = match.groups()
                decode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "case": name
                        or decode_case_name(
                            int(count),
                            parse_bool(comma),
                            parse_bool(utf8),
                            int(value_len),
                        ),
                        "count": int(count),
                        "comma": parse_bool(comma),
                        "utf8": parse_bool(utf8),
                        "value_len": int(value_len),
                        "ms_per_op": float(ms_per_op),
                        "keys": int(keys),
                        "alloc_bytes_per_op": None,
                    }
                )
                continue

        if language == "kotlin":
            if match := KOTLIN_ENCODE_RE.match(line):
                depth, ms_per_op, alloc_value, alloc_unit, length = match.groups()
                encode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "depth": int(depth),
                        "ms_per_op": float(ms_per_op),
                        "length": int(length),
                        "alloc_bytes_per_op": parse_alloc_to_bytes(alloc_value, alloc_unit),
                    }
                )
                continue
            if match := KOTLIN_DECODE_RE.match(line):
                (
                    count,
                    comma,
                    utf8,
                    value_len,
                    ms_per_op,
                    alloc_value,
                    alloc_unit,
                    keys,
                ) = match.groups()
                decode.append(
                    {
                        "language": language,
                        "runtime": language,
                        "case": decode_case_name(
                            int(count),
                            parse_bool(comma),
                            parse_bool(utf8),
                            int(value_len),
                        ),
                        "count": int(count),
                        "comma": parse_bool(comma),
                        "utf8": parse_bool(utf8),
                        "value_len": int(value_len),
                        "ms_per_op": float(ms_per_op),
                        "keys": int(keys),
                        "alloc_bytes_per_op": parse_alloc_to_bytes(alloc_value, alloc_unit),
                    }
                )
                continue

        if language == "swift":
            if match := SWIFT_ENCODE_RE.match(line):
                runtime, depth, ms_per_op, length = match.groups()
                encode.append(
                    {
                        "language": language,
                        "runtime": runtime,
                        "depth": int(depth),
                        "ms_per_op": float(ms_per_op),
                        "length": int(length),
                        "alloc_bytes_per_op": None,
                    }
                )
                continue
            if match := SWIFT_DECODE_RE.match(line):
                runtime, name, count, comma, utf8, value_len, ms_per_op, keys = match.groups()
                decode.append(
                    {
                        "language": language,
                        "runtime": runtime,
                        "case": name,
                        "count": int(count),
                        "comma": parse_bool(comma),
                        "utf8": parse_bool(utf8),
                        "value_len": int(value_len),
                        "ms_per_op": float(ms_per_op),
                        "keys": int(keys),
                        "alloc_bytes_per_op": None,
                    }
                )
                continue

        if language == "csharp":
            parsed = parse_csharp_benchmark_line(line)
            if parsed is not None:
                if parsed["kind"] == "encode":
                    encode.append(parsed["entry"])
                else:
                    decode.append(parsed["entry"])

    return encode, decode


def parse_csharp_benchmark_line(line: str) -> Optional[dict[str, Any]]:
    if not line.startswith("|") or "---" in line or "Method" in line:
        return None

    columns = [column.strip() for column in line.strip("|").split("|")]
    if len(columns) < 5:
        return None

    method = columns[0]
    if method == "Encode_DeepNesting" and len(columns) >= 5:
        depth = int(columns[1])
        ms_per_op = parse_benchmarkdotnet_time(columns[2])
        allocated = parse_benchmarkdotnet_bytes(columns[-1])
        return {
            "kind": "encode",
            "entry": {
                "language": "csharp",
                "runtime": "csharp",
                "depth": depth,
                "ms_per_op": ms_per_op,
                "length": None,
                "alloc_bytes_per_op": allocated,
            },
        }

    if method == "Decode_Public" and len(columns) >= 8:
        count = int(columns[1])
        comma = columns[2].lower() == "true"
        utf8 = columns[3].lower() == "true"
        value_len = int(columns[4])
        ms_per_op = parse_benchmarkdotnet_time(columns[5])
        allocated = parse_benchmarkdotnet_bytes(columns[-1])
        return {
            "kind": "decode",
            "entry": {
                "language": "csharp",
                "runtime": "csharp",
                "case": decode_case_name(count, comma, utf8, value_len),
                "count": count,
                "comma": comma,
                "utf8": utf8,
                "value_len": value_len,
                "ms_per_op": ms_per_op,
                "keys": count,
                "alloc_bytes_per_op": allocated,
            },
        }

    return None


def parse_benchmarkdotnet_time(value: str) -> float:
    match = re.match(r"([0-9][0-9,]*(?:\.[0-9]+)?)\s*(ns|us|μs|ms|s)", value)
    if not match:
        raise ValueError(f"unable to parse BenchmarkDotNet time: {value!r}")
    numeric, unit = match.groups()
    scale = {
        "ns": 1e-6,
        "us": 1e-3,
        "μs": 1e-3,
        "ms": 1.0,
        "s": 1000.0,
    }[unit]
    return float(numeric.replace(",", "")) * scale


def parse_benchmarkdotnet_bytes(value: str) -> Optional[int]:
    match = re.match(r"([0-9][0-9,]*(?:\.[0-9]+)?)\s*(B|KB|MB|GB)", value)
    if not match:
        return None
    numeric, unit = match.groups()
    scale = {
        "B": 1,
        "KB": 1024,
        "MB": 1024 * 1024,
        "GB": 1024 * 1024 * 1024,
    }[unit]
    return int(float(numeric.replace(",", "")) * scale)


def render_markdown(snapshot: dict[str, Any]) -> str:
    lines = [
        "# Performance Comparison",
        "",
        f"Status: `{snapshot['status']}`",
        "",
        f"Captured: `{snapshot['captured_at']}`",
        "",
        "This summary is informative only. Cross-language numbers are machine- and command-specific.",
        "",
        "## Encode",
        "",
        "| Language | Runtime | Depth | ms/op | Length | Alloc/op |",
        "| --- | --- | ---: | ---: | ---: | ---: |",
    ]
    for entry in sorted(
        snapshot["encode"],
        key=lambda item: (item["depth"], item["language"], item["runtime"]),
    ):
        alloc = entry["alloc_bytes_per_op"]
        lines.append(
            f"| {entry['language']} | {entry['runtime']} | {entry['depth']} | {entry['ms_per_op']:.3f} | {entry['length'] if entry['length'] is not None else 'n/a'} | {alloc if alloc is not None else 'n/a'} |"
        )

    lines.extend(
        [
            "",
            "## Decode",
            "",
            "| Language | Runtime | Case | Count | Comma | UTF8 | Len | ms/op | Keys | Alloc/op |",
            "| --- | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: |",
        ]
    )
    for entry in sorted(
        snapshot["decode"],
        key=lambda item: (item["case"], item["language"], item["runtime"]),
    ):
        alloc = entry["alloc_bytes_per_op"]
        lines.append(
            f"| {entry['language']} | {entry['runtime']} | {entry['case']} | {entry['count']} | {str(entry['comma']).lower()} | {str(entry['utf8']).lower()} | {entry['value_len']} | {entry['ms_per_op']:.3f} | {entry['keys']} | {alloc if alloc is not None else 'n/a'} |"
        )

    lines.extend(["", "## Commands", ""])
    for run in snapshot["runs"]:
        command = " ".join(run["command"])
        status = "timeout" if run["timed_out"] else f"rc={run['returncode']}"
        lines.append(f"- `{run['language']}/{run['scenario']}`: `{command}` in `{run['cwd']}` (`{status}`)")

    if snapshot["failures"]:
        lines.extend(["", "## Failures", ""])
        for failure in snapshot["failures"]:
            lines.append(f"- `{failure['language']}/{failure['scenario']}`: {failure['reason']}")

    return "\n".join(lines) + "\n"


def required_decode_cases(entries: list[dict[str, Any]]) -> set[str]:
    return {
        str(entry["case"])
        for entry in entries
        if str(entry.get("case")) in CANONICAL_DECODE_CASES
    }


def spec_missing_rows_reason(
    spec: CommandSpec,
    encode_entries: list[dict[str, Any]],
    decode_entries: list[dict[str, Any]],
) -> Optional[str]:
    failures: list[str] = []

    if spec.scenario in {"encode", "all"} and not encode_entries:
        failures.append("produced no parseable encode rows")

    if spec.scenario in {"decode", "all"}:
        missing_cases = sorted(CANONICAL_DECODE_CASES - required_decode_cases(decode_entries))
        if missing_cases:
            failures.append(f"missing canonical decode cases: {', '.join(missing_cases)}")

    if failures:
        return "; ".join(failures)

    if spec.scenario in {"encode", "decode", "all"}:
        return None

    raise ValueError(f"unsupported scenario: {spec.scenario}")


def covered_families(snapshot: dict[str, Any]) -> set[str]:
    families = set()
    for entry in snapshot["encode"] + snapshot["decode"]:
        language = entry["language"]
        runtime = entry["runtime"]
        if language == "swift":
            families.add(runtime)
        else:
            families.add(language)
    return families


def main() -> None:
    args = parse_args()
    snapshot: dict[str, Any] = {
        "status": "complete",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "encode": [],
        "decode": [],
        "runs": [],
        "failures": [],
    }

    for spec in all_specs():
        result = run_command(spec, timeout=args.timeout)
        snapshot["runs"].append(result)
        if result["timed_out"]:
            snapshot["status"] = "partial"
            snapshot["failures"].append(
                {
                    "language": spec.language,
                    "scenario": spec.scenario,
                    "reason": f"command timed out after {args.timeout}s",
                }
            )
            continue

        if result["returncode"] != 0:
            snapshot["status"] = "partial"
            snapshot["failures"].append(
                {
                    "language": spec.language,
                    "scenario": spec.scenario,
                    "reason": f"command exited with {result['returncode']}",
                }
            )
            continue

        try:
            encode_entries, decode_entries = parse_snapshot_output(
                spec.language,
                spec.scenario,
                result["stdout"],
            )
            decode_entries = filter_canonical_decode_entries(decode_entries)
        except Exception as exc:  # pragma: no cover - best effort capture
            snapshot["status"] = "partial"
            snapshot["failures"].append(
                {
                    "language": spec.language,
                    "scenario": spec.scenario,
                    "reason": f"parse failed: {exc}",
                }
            )
            continue

        snapshot["encode"].extend(encode_entries)
        snapshot["decode"].extend(decode_entries)

        missing_rows_reason = spec_missing_rows_reason(spec, encode_entries, decode_entries)
        if missing_rows_reason is not None:
            snapshot["status"] = "partial"
            snapshot["failures"].append(
                {
                    "language": spec.language,
                    "scenario": spec.scenario,
                    "reason": f"coverage failed: {missing_rows_reason}",
                }
            )
            continue

    if not snapshot["encode"] and not snapshot["decode"]:
        snapshot["status"] = "pending"
    elif snapshot["status"] == "complete":
        missing_families = sorted(
            {"rust", "python", "dart", "kotlin", "swift", "objc", "csharp"} - covered_families(snapshot)
        )
        if missing_families:
            snapshot["status"] = "partial"
            for family in missing_families:
                snapshot["failures"].append(
                    {
                        "language": family,
                        "scenario": "coverage",
                        "reason": "missing parsed family coverage",
                    }
                )

    args.json_output.parent.mkdir(parents=True, exist_ok=True)
    args.json_output.write_text(json.dumps(snapshot, indent=2, sort_keys=True) + "\n")

    markdown = render_markdown(snapshot)
    args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
    args.markdown_output.write_text(markdown)


if __name__ == "__main__":
    main()
