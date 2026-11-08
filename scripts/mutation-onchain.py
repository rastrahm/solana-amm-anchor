#!/usr/bin/env python3
"""On-chain mutation testing for the AMM program.

`cargo mutants` only runs `cargo test`, but instruction validations are covered by the
TypeScript suite (`anchor test`). This script disables one validation at a time
(`require!` family or Anchor `constraint = ...`), runs `anchor test`, and reports whether
some test caught it. Source files are always restored, even on Ctrl+C.

Usage (no local validator may be running on :8899):
    python3 scripts/mutation-onchain.py            # all validations
    python3 scripts/mutation-onchain.py deposit    # only files whose path contains "deposit"
"""

from __future__ import annotations

import os
import pathlib
import re
import signal
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
SUITE_TIMEOUT_S = 600
TARGETS = [
    *sorted((ROOT / "programs/amm/src/instructions").glob("*.rs")),
    ROOT / "programs/amm/src/helpers/mint_policy.rs",
]
REQUIRE_RE = re.compile(r"^(\s*)(require(?:_keys_neq|_keys_eq)?)!\(")
CONSTRAINT_RE = re.compile(r"constraint = (.+?) @ ")


def statement_end(lines: list[str], start: int) -> int:
    """Index of the line closing the macro call that opens on `start`."""
    depth = 0
    for i in range(start, len(lines)):
        for ch in lines[i]:
            depth += ch == "("
            depth -= ch == ")"
        if depth == 0:
            return i
    raise ValueError(f"unbalanced macro starting at line {start + 1}")


def mutants(path: pathlib.Path):
    lines = path.read_text().splitlines(keepends=True)
    for i, line in enumerate(lines):
        if m := REQUIRE_RE.match(line):
            end = statement_end(lines, i)
            call = "".join(lines[i : end + 1])
            args = call[call.index("(") + 1 : call.rindex(")")]
            mutated = f"{m.group(1)}let _ = ({args});\n"
            label = " ".join(call.split())[:90]
            yield i + 1, label, lines[:i] + [mutated] + lines[end + 1 :]
        elif CONSTRAINT_RE.search(line):
            mutated = CONSTRAINT_RE.sub("constraint = true @ ", line)
            yield i + 1, " ".join(line.split())[:90], lines[:i] + [mutated] + lines[i + 1 :]


def run_suite() -> tuple[str, str]:
    # A mutant can make a test hang (e.g. waiting on a tx that never fails); that counts
    # as caught. The whole process group is killed so the local validator does not leak.
    proc = subprocess.Popen(
        ["anchor", "test"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    try:
        out, _ = proc.communicate(timeout=SUITE_TIMEOUT_S)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.communicate()
        return "TIMEOUT", f"suite exceeded {SUITE_TIMEOUT_S}s"
    failing = re.search(r"(\d+) failing", out)
    if failing:
        return "CAUGHT", f"{failing.group(1)} failing"
    if re.search(r"error(\[E\d+\])?: ", out) and "passing" not in out:
        return "UNVIABLE", "does not compile"
    if proc.returncode != 0:
        return "CAUGHT", f"exit {proc.returncode}"
    passing = re.search(r"(\d+) passing", out)
    return "SURVIVED", f"{passing.group(1) if passing else '?'} passing"


def main() -> int:
    only = sys.argv[1] if len(sys.argv) > 1 else ""
    results = []
    for path in TARGETS:
        if only and only not in str(path):
            continue
        original = path.read_text()
        try:
            for line_no, label, mutated in mutants(path):
                path.write_text("".join(mutated))
                started = time.time()
                status, detail = run_suite()
                rel = path.relative_to(ROOT)
                row = f"{status:9} {rel}:{line_no}  {label}  ({detail}, {time.time() - started:.0f}s)"
                print(row, flush=True)
                results.append(status)
                path.write_text(original)
        finally:
            path.write_text(original)

    survived = results.count("SURVIVED")
    print(
        f"\n{len(results)} mutants: {results.count('CAUGHT')} caught, "
        f"{results.count('TIMEOUT')} timeouts, {survived} survived, "
        f"{results.count('UNVIABLE')} unviable",
        flush=True,
    )
    return 1 if survived else 0


if __name__ == "__main__":
    sys.exit(main())
