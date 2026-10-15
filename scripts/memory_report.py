#!/usr/bin/env python3
"""macOS process RSS/time experiment; synthetic cases, not production telemetry."""
import argparse
import hashlib
import json
import math
import platform
import re
import statistics
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("this experiment requires macOS /usr/bin/time -l (RSS in bytes)")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    binary = ROOT / "target/release/examples/memory_probe"
    # Exact public synthetic reproducer 1, not a captured customer payload:
    # https://github.com/GCWing/OpenBitFun/issues/3229#issuecomment-6030422809
    cases = [("public-heading", '{"plan": # Markdown heading"}',
              {"plan": "# Markdown heading"}, "public synthetic reproducer 1")]
    for target in (1024, 64 * 1024, 1024 * 1024, 8 * 1024 * 1024):
        # Amplification of examples/tool_arguments.rs and downstream_contract_tests.rs.
        for name, unit in (("markdown", r"# Heading\n// code\n/* example */"),
                           ("unicode", "中文 🦀")):
            count = math.ceil(target / len(unit.encode()))
            body = unit * count
            if name == "markdown":
                text = '{"content": ' + body + '"}'
                expected = {"content": body.replace(r"\n", "\n")}
            else:
                text = '{"query":"' + body  # truncated quote and object, like regression
                expected = {"query": body}
            cases.append((f"{name}-{target}", text, expected,
                          "synthetic repetition of existing regression shape"))
    metadata = {
        "date_utc": command("date", "-u", "+%Y-%m-%dT%H:%M:%SZ"),
        "base_revision": command("git", "rev-parse", "HEAD"),
        "rust": command("rustc", "-Vv"), "cargo": command("cargo", "-V"),
        "python": platform.python_version(), "os": command("sw_vers"),
        "machine": platform.machine(), "cpu": command("sysctl", "-n", "machdep.cpu.brand_string"),
        "memory_bytes": command("sysctl", "-n", "hw.memsize"),
        "load_at_start": command("uptime"), "tool": "/usr/bin/time -lp (Apple OS bundled)",
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "repeats": 3, "warmups": 1,
        "policy": "preserve_comment_markers=true, decode_unquoted_escapes=true, no byte limit",
        "scope": "whole process, file read + repair + stdout to /dev/null; no serde parsing",
        "wall_clock": "Python perf_counter_ns around time subprocess, includes time startup",
        "cases": [],
    }
    rows = []
    for name, text, expected, source in cases:
        input_path = output / f"{name}.input"
        raw = text.encode()
        input_path.write_bytes(raw)
        metadata["cases"].append({"name": name, "bytes": len(raw),
            "unicode_scalars": len(text), "sha256": hashlib.sha256(raw).hexdigest(),
            "source": source})
        for mode in ("string", "reader"):
            invocation = [str(binary), mode, str(input_path)]
            # One untimed warmup validates full content before measuring to a sink.
            repaired = subprocess.check_output(invocation)
            assert json.loads(repaired) == expected, (name, mode)
            for repetition in range(1, 4):
                with open("/dev/null", "wb") as sink:
                    started = time.perf_counter_ns()
                    run = subprocess.run(["/usr/bin/time", "-lp", *invocation],
                                         stdout=sink, stderr=subprocess.PIPE, check=True, text=True)
                    wall_seconds = (time.perf_counter_ns() - started) / 1e9
                (output / f"{name}-{mode}-{repetition}.time.txt").write_text(run.stderr)
                rss = int(re.search(r"(\d+)\s+maximum resident set size", run.stderr)[1])
                real = float(re.search(r"^real\s+([\d.]+)", run.stderr, re.M)[1])
                rows.append({"case": name, "mode": mode, "repeat": repetition,
                             "input_bytes": len(raw), "output_bytes": len(repaired),
                             "peak_rss_bytes": rss, "time_real_seconds": real,
                             "elapsed_seconds": wall_seconds,
                             "command": ["/usr/bin/time", "-lp", *invocation]})
    metadata["load_at_end"] = command("uptime")
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (output / "results.json").write_text(json.dumps(rows, indent=2) + "\n")
    lines = ["| Case | Input bytes | Mode | Peak RSS MiB (median; min–max) | Wall ms (median; min–max) |",
             "| --- | ---: | --- | ---: | ---: |"]
    for case in metadata["cases"]:
        for mode in ("string", "reader"):
            samples = [r for r in rows if r["case"] == case["name"] and r["mode"] == mode]
            rss = [r["peak_rss_bytes"] / 1024**2 for r in samples]
            times = [r["elapsed_seconds"] * 1000 for r in samples]
            lines.append(f"| {case['name']} | {case['bytes']} | {mode} | "
                         f"{statistics.median(rss):.2f}; {min(rss):.2f}–{max(rss):.2f} | "
                         f"{statistics.median(times):.2f}; {min(times):.2f}–{max(times):.2f} |")
    (output / "table.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
