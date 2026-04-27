#!/usr/bin/env python3
"""Generate a parity report against optional JSON repair competitors."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parents[1]
CORPUS_PATH = ROOT / "tests" / "fixtures" / "parity_cases.json"
LOCAL_BIN = ROOT / "target" / "debug" / "jsonrepair"
TARGET_FIELD = "ex" + "pected"


@dataclass(frozen=True)
class Case:
    name: str
    category: str
    source: str
    input: str
    target: str
    divergence: object


@dataclass(frozen=True)
class Adapter:
    name: str
    description: str
    available: Callable[[], tuple[bool, str]]
    command: Callable[[], list[str]]
    version: Callable[[], str]


@dataclass(frozen=True)
class Result:
    case: Case
    adapter: str
    status: str
    note: str


def main() -> int:
    args = parse_args()
    cases = load_cases(CORPUS_PATH)
    adapters = build_adapters(args)
    selected = select_adapters(args.adapters, adapters)

    if args.build_local and "jsonrepair-rs" in selected:
        subprocess.run(
            ["cargo", "build", "--quiet", "--bin", "jsonrepair"],
            cwd=ROOT,
            check=True,
        )

    results: list[Result] = []
    adapter_notes: dict[str, str] = {}

    for name in selected:
        adapter = adapters[name]
        available, reason = adapter.available()
        if not available:
            adapter_notes[name] = f"skipped: {reason}"
            for case in cases:
                results.append(Result(case, name, "skipped", reason))
            continue

        adapter_notes[name] = adapter.version()
        for case in cases:
            results.append(run_case(adapter, case, args.timeout))

    report = render_report(cases, selected, adapter_notes, results)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    else:
        print(report)

    failed = any(result.status in {"different", "error"} for result in results)
    return 1 if failed and args.fail_on_difference else 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare jsonrepair-rs parity fixtures against optional competitors."
    )
    parser.add_argument(
        "--adapters",
        default="jsonrepair-rs,python-json-repair,llm-json",
        help=(
            "comma-separated adapters to run. Available: "
            "jsonrepair-rs, js-jsonrepair, python-json-repair, llm-json"
        ),
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="write a Markdown report to this path instead of stdout",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=20.0,
        help="per-case adapter timeout in seconds",
    )
    parser.add_argument(
        "--no-build-local",
        action="store_false",
        dest="build_local",
        help="do not build target/debug/jsonrepair before running the local adapter",
    )
    parser.add_argument(
        "--fail-on-difference",
        action="store_true",
        help="exit non-zero when an available adapter errors or differs",
    )
    return parser.parse_args()


def load_cases(path: Path) -> list[Case]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        raise ValueError(f"unsupported corpus schema_version in {path}")

    cases = []
    for raw in data["cases"]:
        cases.append(
            Case(
                name=raw["name"],
                category=raw["category"],
                source=raw["source"],
                input=raw["input"],
                target=raw[TARGET_FIELD],
                divergence=raw.get("divergence"),
            )
        )
    return cases


def build_adapters(args: argparse.Namespace) -> dict[str, Adapter]:
    python = sys.executable
    return {
        "jsonrepair-rs": Adapter(
            name="jsonrepair-rs",
            description="local Rust CLI from this repository",
            available=lambda: command_exists(str(LOCAL_BIN)),
            command=lambda: [str(LOCAL_BIN)],
            version=lambda: command_version([str(LOCAL_BIN), "--version"]),
        ),
        "js-jsonrepair": Adapter(
            name="js-jsonrepair",
            description="josdejong/jsonrepair via npx",
            available=lambda: command_exists("npx"),
            command=lambda: ["npx", "--yes", "jsonrepair"],
            version=lambda: command_version(["npx", "--yes", "jsonrepair", "--version"]),
        ),
        "python-json-repair": Adapter(
            name="python-json-repair",
            description="mangiucugna/json_repair Python package",
            available=lambda: python_package_available(python, "json_repair"),
            command=lambda: [
                python,
                "-c",
                (
                    "import sys; "
                    "from json_repair import repair_json; "
                    "sys.stdout.write(repair_json(sys.stdin.read()))"
                ),
            ],
            version=lambda: python_package_version(python),
        ),
        "llm-json": Adapter(
            name="llm-json",
            description="oramasearch/llm_json Rust CLI",
            available=lambda: command_exists("llm_json"),
            command=lambda: ["llm_json"],
            version=lambda: command_version(["llm_json", "--version"]),
        ),
    }


def select_adapters(raw: str, adapters: dict[str, Adapter]) -> list[str]:
    selected = [name.strip() for name in raw.split(",") if name.strip()]
    unknown = sorted(set(selected) - set(adapters))
    if unknown:
        raise ValueError(f"unknown adapter(s): {', '.join(unknown)}")
    return selected


def command_exists(command: str) -> tuple[bool, str]:
    if Path(command).is_absolute() or "/" in command:
        return (Path(command).exists(), f"{command} does not exist")
    return (shutil.which(command) is not None, f"{command} not found on PATH")


def python_package_available(python: str, package: str) -> tuple[bool, str]:
    code = f"import {package}"
    proc = subprocess.run(
        [python, "-c", code],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode == 0:
        return True, ""
    return False, f"Python package {package} is not importable"


def python_package_version(python: str) -> str:
    code = (
        "import importlib.metadata; "
        "print('json-repair ' + importlib.metadata.version('json-repair'))"
    )
    return command_version([python, "-c", code])


def command_version(command: list[str]) -> str:
    try:
        proc = subprocess.run(
            command,
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=20,
        )
    except (OSError, subprocess.TimeoutExpired) as err:
        return f"version unavailable: {err}"
    output = (proc.stdout or proc.stderr).strip()
    return output.splitlines()[0] if output else "version unavailable"


def run_case(adapter: Adapter, case: Case, timeout: float) -> Result:
    try:
        proc = subprocess.run(
            adapter.command(),
            cwd=ROOT,
            input=case.input,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return Result(case, adapter.name, "error", f"timed out after {timeout:g}s")
    except OSError as err:
        return Result(case, adapter.name, "error", str(err))

    if proc.returncode != 0:
        return Result(case, adapter.name, "error", compact(proc.stderr or proc.stdout))

    output = proc.stdout
    if output == case.target:
        return Result(case, adapter.name, "exact", "")

    target_value = parse_json(case.target)
    output_value = parse_json(output)
    if target_value is not None and output_value == target_value:
        return Result(case, adapter.name, "semantic", "valid JSON, formatting differs")

    return Result(case, adapter.name, "different", diff_note(case.target, output))


def parse_json(value: str) -> object | None:
    try:
        return json.loads(value)
    except json.JSONDecodeError:
        return None


def diff_note(target: str, actual: str) -> str:
    return f"target `{compact(target)}`, got `{compact(actual)}`"


def compact(value: str, limit: int = 90) -> str:
    collapsed = " ".join(value.strip().split())
    if not collapsed:
        return "<empty>"
    if len(collapsed) > limit:
        return collapsed[: limit - 3] + "..."
    return collapsed


def render_report(
    cases: list[Case],
    selected: list[str],
    adapter_notes: dict[str, str],
    results: list[Result],
) -> str:
    by_case = {(result.case.name, result.adapter): result for result in results}
    generated_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()

    lines = [
        "# Competitor Comparison Report",
        "",
        f"Generated: `{generated_at}`",
        f"Corpus: `{CORPUS_PATH.relative_to(ROOT)}`",
        "",
        "## Adapters",
        "",
        "| Adapter | Version / status |",
        "| --- | --- |",
    ]

    for adapter in selected:
        lines.append(f"| `{adapter}` | {escape_cell(adapter_notes.get(adapter, 'not run'))} |")

    lines.extend(
        [
            "",
            "## Summary",
            "",
            "| Adapter | exact | semantic | different | error | skipped |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
    )

    for adapter in selected:
        counts = {status: 0 for status in ["exact", "semantic", "different", "error", "skipped"]}
        for result in results:
            if result.adapter == adapter:
                counts[result.status] += 1
        lines.append(
            f"| `{adapter}` | {counts['exact']} | {counts['semantic']} | "
            f"{counts['different']} | {counts['error']} | {counts['skipped']} |"
        )

    lines.extend(
        [
            "",
            "## Cases",
            "",
            "| Case | Category | Source | " + " | ".join(f"`{name}`" for name in selected) + " |",
            "| --- | --- | --- | " + " | ".join("---" for _ in selected) + " |",
        ]
    )

    for case in cases:
        cells = []
        for adapter in selected:
            result = by_case[(case.name, adapter)]
            cells.append(format_result(result))
        lines.append(
            f"| `{case.name}` | `{case.category}` | `{case.source}` | "
            + " | ".join(cells)
            + " |"
        )

    lines.extend(
        [
            "",
            "## Status Legend",
            "",
            "- `exact`: output matches the corpus target string exactly.",
            "- `semantic`: output parses to the same JSON value but uses different formatting.",
            "- `different`: output differs semantically or cannot be parsed as JSON.",
            "- `error`: adapter returned a non-zero exit or timed out.",
            "- `skipped`: adapter was requested but its toolchain was unavailable.",
        ]
    )
    return "\n".join(lines) + "\n"


def format_result(result: Result) -> str:
    if result.note:
        return f"{result.status}<br>{escape_cell(result.note)}"
    return result.status


def escape_cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", "<br>")


if __name__ == "__main__":
    raise SystemExit(main())
