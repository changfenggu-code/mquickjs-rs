#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from datetime import datetime
from pathlib import Path


SECTIONS = {
    "criterion": {
        "title": "Rust Criterion",
        "placeholder": "_No Rust Criterion results recorded yet._",
    },
    "execution_only": {
        "title": "Rust-vs-C Execution-Only",
        "placeholder": "_No execution-only Rust-vs-C results recorded yet._",
    },
    "end_to_end": {
        "title": "Rust-vs-C End-to-End",
        "placeholder": "_No end-to-end Rust-vs-C results recorded yet._",
    },
}


def build_template() -> str:
    lines = [
        "# Latest Benchmark Results",
        "",
        "- This file keeps the latest result for each benchmark view.",
        "- Each section is overwritten independently when its corresponding command runs.",
        "",
    ]
    for key, meta in SECTIONS.items():
        lines.extend(
            [
                f"## {meta['title']}",
                "",
                f"<!-- BEGIN {key} -->",
                meta["placeholder"],
                f"<!-- END {key} -->",
                "",
            ]
        )
    return "\n".join(lines).rstrip() + "\n"


def read_text_auto(path: Path) -> str:
    raw = path.read_bytes()
    for encoding in ("utf-8", "utf-8-sig", "utf-16", "utf-16-le", "utf-16-be"):
        try:
            return raw.decode(encoding)
        except UnicodeDecodeError:
            continue
    return raw.decode("utf-8", errors="replace")


def ensure_report(path: Path) -> str:
    if not path.exists():
        template = build_template()
        path.write_text(template, encoding="utf-8")
        return template
    content = read_text_auto(path)
    for key in SECTIONS:
        if f"<!-- BEGIN {key} -->" not in content or f"<!-- END {key} -->" not in content:
            template = build_template()
            path.write_text(template, encoding="utf-8")
            return template
    return content


def replace_section(content: str, key: str, body: str) -> str:
    pattern = re.compile(
        rf"<!-- BEGIN {re.escape(key)} -->.*?<!-- END {re.escape(key)} -->",
        re.DOTALL,
    )
    replacement = f"<!-- BEGIN {key} -->\n{body.rstrip()}\n<!-- END {key} -->"
    updated, count = pattern.subn(lambda _match: replacement, content, count=1)
    if count != 1:
        raise RuntimeError(f"failed to replace section: {key}")
    return updated


def write_section(report_path: Path, key: str, fragment_path: Path) -> None:
    if key not in SECTIONS:
        raise RuntimeError(f"unknown section key: {key}")
    content = ensure_report(report_path)
    body = read_text_auto(fragment_path)
    updated = replace_section(content, key, body)
    report_path.write_text(updated, encoding="utf-8")


def render_criterion(output_path: Path) -> str:
    output = read_text_auto(output_path)
    output = output.replace("碌s", "µs")
    rows = []
    pattern = re.compile(r"^\s*(.*?)\s+time:\s+(.*)$")
    for line in output.splitlines():
        match = pattern.match(line)
        if not match:
            continue
        name = match.group(1).strip()
        timing = match.group(2).strip()
        if not name or name.startswith("Benchmarking "):
            continue
        rows.append((name, timing))

    generated_at = datetime.now().astimezone().strftime("%Y-%m-%d %H:%M:%S %z")
    lines = [
        f"- Generated at: `{generated_at}`",
        "- Metric: `Criterion Rust-only runtime benchmark`",
        "- Selection: `full js_benchmarks bench target`",
        "",
    ]
    if rows:
        lines.extend(
            [
                "| Benchmark | Criterion Time |",
                "|-----------|----------------|",
            ]
        )
        for name, timing in rows:
            lines.append(f"| {name} | {timing} |")
    else:
        lines.append("_No Criterion benchmark timings were parsed from the latest run._")
    return "\n".join(lines)


def usage() -> None:
    print(
        "usage:\n"
        "  update_latest_results.py write-section <report-path> <section-key> <fragment-path>\n"
        "  update_latest_results.py write-criterion <report-path> <criterion-output-path>",
        file=sys.stderr,
    )


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        usage()
        return 2
    command = argv[1]
    try:
        if command == "write-section":
            if len(argv) != 5:
                usage()
                return 2
            write_section(Path(argv[2]), argv[3], Path(argv[4]))
            return 0
        if command == "write-criterion":
            if len(argv) != 4:
                usage()
                return 2
            report_path = Path(argv[2])
            content = ensure_report(report_path)
            body = render_criterion(Path(argv[3]))
            updated = replace_section(content, "criterion", body)
            report_path.write_text(updated, encoding="utf-8")
            return 0
        usage()
        return 2
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
