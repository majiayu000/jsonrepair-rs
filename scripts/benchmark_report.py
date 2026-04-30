#!/usr/bin/env python3
"""Generate a small latency and throughput report for repair benchmark cases."""

from __future__ import annotations

import argparse
import datetime as dt
import shutil
import statistics
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LOCAL_BIN = ROOT / "target" / "debug" / "jsonrepair"


@dataclass(frozen=True)
class Case:
    name: str
    input: str


@dataclass(frozen=True)
class Adapter:
    name: str
    command: list[str]
    available: bool
    note: str


@dataclass(frozen=True)
class Measurement:
    adapter: str
    case: str
    bytes_in: int
    warmups: int
    iterations: int
    median_ms: float
    min_ms: float
    max_ms: float
    throughput_mib_s: float
    status: str
    note: str


def main() -> int:
    args = parse_args()
    cases = build_cases()
    adapters = select_adapters(args.adapters)

    if args.build_local and any(adapter.name == "jsonrepair-rs" for adapter in adapters):
        subprocess.run(["cargo", "build", "--quiet", "--bin", "jsonrepair"], cwd=ROOT, check=True)

    measurements: list[Measurement] = []
    for adapter in adapters:
        if not adapter.available:
            for case in cases:
                measurements.append(
                    Measurement(
                        adapter=adapter.name,
                        case=case.name,
                        bytes_in=len(case.input.encode("utf-8")),
                        warmups=0,
                        iterations=0,
                        median_ms=0.0,
                        min_ms=0.0,
                        max_ms=0.0,
                        throughput_mib_s=0.0,
                        status="skipped",
                        note=adapter.note,
                    )
                )
            continue

        for case in cases:
            measurements.append(run_case(adapter, case, args.warmups, args.iterations, args.timeout))

    report = render_report(measurements, args.iterations)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    else:
        print(report)
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark representative JSON repair inputs against optional Rust competitors."
    )
    parser.add_argument(
        "--adapters",
        default="jsonrepair-rs,llm-json",
        help="comma-separated adapters to run. Available: jsonrepair-rs, llm-json",
    )
    parser.add_argument("--iterations", type=int, default=30, help="measured iterations per adapter/case")
    parser.add_argument("--warmups", type=int, default=3, help="warmup iterations per adapter/case")
    parser.add_argument("--timeout", type=float, default=10.0, help="per-iteration timeout in seconds")
    parser.add_argument("--output", type=Path, help="write a Markdown report to this path")
    parser.add_argument(
        "--no-build-local",
        action="store_false",
        dest="build_local",
        help="do not build target/debug/jsonrepair before running the local adapter",
    )
    return parser.parse_args()


def build_cases() -> list[Case]:
    valid_large = "[{}]".format(
        ",".join(
            f'{{"id": {idx}, "name": "item_{idx}", "value": {idx * 10}}}'
            for idx in range(1000)
        )
    )
    broken_large = "[{}]".format(
        ",".join(
            f"{{'id': {idx}, 'name': 'item_{idx}', 'value': {idx * 10},}}"
            for idx in range(1000)
        )
    )
    comments = "{\n" + "".join(
        f'  // comment {idx}\n  "key_{idx}": {idx},\n' for idx in range(100)
    ) + '  "last": true\n}'
    string_escapes = "[" + ",".join([r'"hello\nworld\t\"quoted\""' for _ in range(200)]) + "]"

    return [
        Case("valid_small", r'{"name": "John", "age": 30, "items": [1, 2, 3]}'),
        Case("broken_small", "{'name': 'John', 'age': 30, 'items': [1, 2, 3,]}"),
        Case("valid_large_1k", valid_large),
        Case("broken_large_1k", broken_large),
        Case("nested_100", "[" * 100 + "]" * 100),
        Case("comments_100", comments),
        Case("string_escapes_200", string_escapes),
    ]


def select_adapters(raw: str) -> list[Adapter]:
    selected = [name.strip() for name in raw.split(",") if name.strip()]
    known = {
        "jsonrepair-rs": lambda: Adapter(
            name="jsonrepair-rs",
            command=[str(LOCAL_BIN)],
            available=LOCAL_BIN.exists(),
            note=f"{LOCAL_BIN} does not exist",
        ),
        "llm-json": lambda: Adapter(
            name="llm-json",
            command=["llm_json"],
            available=shutil.which("llm_json") is not None,
            note="llm_json not found on PATH",
        ),
    }
    unknown = sorted(set(selected) - set(known))
    if unknown:
        raise ValueError(f"unknown adapter(s): {', '.join(unknown)}")
    return [known[name]() for name in selected]


def run_case(adapter: Adapter, case: Case, warmups: int, iterations: int, timeout: float) -> Measurement:
    times: list[float] = []
    note = ""
    for index in range(warmups + iterations):
        started = time.perf_counter()
        try:
            proc = subprocess.run(
                adapter.command,
                cwd=ROOT,
                input=case.input,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=timeout,
            )
        except subprocess.TimeoutExpired:
            return empty_measurement(adapter, case, iterations, "error", f"timed out after {timeout:g}s")

        elapsed_ms = (time.perf_counter() - started) * 1000
        if proc.returncode != 0:
            return empty_measurement(
                adapter,
                case,
                iterations,
                "error",
                (proc.stderr or proc.stdout).strip().splitlines()[0],
            )
        if index >= warmups:
            times.append(elapsed_ms)

    median_ms = statistics.median(times)
    mib = len(case.input.encode("utf-8")) / (1024 * 1024)
    throughput = mib / (median_ms / 1000) if median_ms > 0 else 0.0
    return Measurement(
        adapter=adapter.name,
        case=case.name,
        bytes_in=len(case.input.encode("utf-8")),
        warmups=warmups,
        iterations=iterations,
        median_ms=median_ms,
        min_ms=min(times),
        max_ms=max(times),
        throughput_mib_s=throughput,
        status="ok",
        note=note,
    )


def empty_measurement(
    adapter: Adapter, case: Case, iterations: int, status: str, note: str
) -> Measurement:
    return Measurement(
        adapter=adapter.name,
        case=case.name,
        bytes_in=len(case.input.encode("utf-8")),
        warmups=0,
        iterations=iterations,
        median_ms=0.0,
        min_ms=0.0,
        max_ms=0.0,
        throughput_mib_s=0.0,
        status=status,
        note=note,
    )


def render_report(measurements: list[Measurement], iterations: int) -> str:
    lines = [
        "# Benchmark Metrics Report",
        "",
        f"Generated: {dt.datetime.now(dt.timezone.utc).date().isoformat()}",
        "",
        "This report times representative repair inputs through CLI adapters. It is intended",
        "for local comparison and trend inspection, not as a stable CI gate.",
        "",
        f"Measured iterations per adapter/case: `{iterations}`",
        "",
        "| Adapter | Case | Input bytes | Median ms | Min ms | Max ms | Throughput MiB/s | Status | Note |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |",
    ]
    for item in measurements:
        lines.append(
            "| {adapter} | {case} | {bytes_in} | {median:.3f} | {min_ms:.3f} | {max_ms:.3f} | {throughput:.2f} | {status} | {note} |".format(
                adapter=item.adapter,
                case=item.case,
                bytes_in=item.bytes_in,
                median=item.median_ms,
                min_ms=item.min_ms,
                max_ms=item.max_ms,
                throughput=item.throughput_mib_s,
                status=item.status,
                note=item.note.replace("|", "\\|"),
            )
        )

    successful = [item for item in measurements if item.status == "ok"]
    if successful:
        slowest = sorted(successful, key=lambda item: item.median_ms, reverse=True)[:3]
        throughput_cases = [item for item in successful if item.bytes_in >= 1024]
        lowest_throughput = sorted(throughput_cases, key=lambda item: item.throughput_mib_s)[:3]
        lines.extend(["", "## Current Hotspots", ""])
        lines.append("Slowest median latency:")
        for item in slowest:
            lines.append(f"- `{item.adapter}` / `{item.case}`: {item.median_ms:.3f} ms")
        lines.append("")
        lines.append("Lowest throughput among inputs >= 1 KiB:")
        for item in lowest_throughput:
            lines.append(
                f"- `{item.adapter}` / `{item.case}`: {item.throughput_mib_s:.2f} MiB/s"
            )

    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    raise SystemExit(main())
