#!/usr/bin/env python3
"""Run the Firecrawl AnyDoc abuse corpus without vendoring its fixtures."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import signal
import struct
import subprocess
import sys
import time


CASES = (
    ("deepxml--errors.docx", 1, "resource_limit"),
    ("imagebomb--errors.docx", 1, "resource_limit"),
    ("zipbomb--errors.docx", 1, "resource_limit"),
    ("hugespan--errors.pptx", 3, "resource_limit"),
    ("emptyrowrepeat--errors.ods", 4, "resource_limit"),
    ("hugespan--errors.ods", 4, "resource_limit"),
    ("hugerepeat--errors.ods", 4, "resource_limit"),
)
MAGIC = b"ADW1"
VERSION = 2
HEADER_BYTES = 16


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Measure reviewed Firecrawl AnyDoc abuse fixtures through the real worker."
    )
    parser.add_argument(
        "--mirror",
        type=pathlib.Path,
        required=True,
        help="local firecrawl-anydoc mirror containing tests/fixtures/abuse",
    )
    parser.add_argument(
        "--worker",
        type=pathlib.Path,
        default=pathlib.Path("target/release/pdf-inspector-mcp"),
        help="workspace worker executable, relative to the repository by default",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=20.0,
        help="per-fixture wall-time limit",
    )
    return parser.parse_args()


def command_for(worker: pathlib.Path) -> list[str]:
    if sys.platform == "darwin":
        return ["/usr/bin/sandbox-exec", "-n", "no-network", str(worker), "--anydoc-worker"]
    if sys.platform == "linux":
        return [str(worker), "--anydoc-worker"]
    raise RuntimeError("this evaluator requires Darwin or Linux worker containment")


def frame_for(code: int, payload: bytes) -> bytes:
    body = bytes([code]) + payload
    return MAGIC + bytes([VERSION, 0, 0, 0]) + struct.pack("<Q", len(body)) + body


def decode_response(stdout: bytes) -> dict[str, object]:
    if len(stdout) < HEADER_BYTES or stdout[:4] != MAGIC or stdout[4] != VERSION:
        raise ValueError("worker returned an invalid frame")
    payload_len = struct.unpack("<Q", stdout[8:HEADER_BYTES])[0]
    payload = stdout[HEADER_BYTES : HEADER_BYTES + payload_len]
    if len(payload) != payload_len or len(stdout) != HEADER_BYTES + payload_len:
        raise ValueError("worker returned a truncated frame")
    value = json.loads(payload)
    if not isinstance(value, dict):
        raise ValueError("worker returned a non-object response")
    return value


def run_case(worker: pathlib.Path, fixture: pathlib.Path, code: int, expected: str, timeout: float) -> dict[str, object]:
    payload = frame_for(code, fixture.read_bytes())
    environment = os.environ.copy()
    environment["ANYDOC_RESOURCE_EVIDENCE"] = "1"
    started = time.monotonic()
    process = subprocess.Popen(
        command_for(worker),
        env=environment,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    timed_out = False
    try:
        stdout, _stderr = process.communicate(payload, timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGKILL)
        stdout, _stderr = process.communicate()
    elapsed_ms = round((time.monotonic() - started) * 1000)

    result: dict[str, object] = {
        "fixture": fixture.name,
        "input_bytes": fixture.stat().st_size,
        "expected": expected,
        "exit_code": process.returncode,
        "wall_clock_ms": elapsed_ms,
        "stdout_bytes": len(stdout),
    }
    if timed_out:
        result["actual"] = "timeout"
        result["ok"] = False
        return result
    try:
        response = decode_response(stdout)
        error = response.get("error")
        actual = error.get("code") if isinstance(error, dict) else "complete"
        result["actual"] = actual
        resource = response.get("resource")
        if isinstance(resource, dict) and isinstance(resource.get("peak_rss_bytes"), int):
            result["peak_rss_bytes"] = resource["peak_rss_bytes"]
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        result["actual"] = "invalid_response"
        result["parse_error"] = str(exc)
    result["ok"] = result.get("actual") == expected and process.returncode == 0
    return result


def main() -> int:
    args = parse_args()
    mirror = args.mirror.expanduser().resolve()
    worker = args.worker.expanduser().resolve()
    abuse = mirror / "tests" / "fixtures" / "abuse"
    if not abuse.is_dir():
        print("abuse fixture directory is missing", file=sys.stderr)
        return 2
    if not worker.is_file():
        print("worker executable is missing", file=sys.stderr)
        return 2
    if args.timeout_seconds <= 0:
        print("timeout must be positive", file=sys.stderr)
        return 2

    results = []
    for name, code, expected in CASES:
        fixture = abuse / name
        if not fixture.is_file():
            print(f"missing abuse fixture: {name}", file=sys.stderr)
            return 2
        result = run_case(worker, fixture, code, expected, args.timeout_seconds)
        results.append(result)
        print(json.dumps(result, sort_keys=True))

    passed = sum(1 for result in results if result["ok"])
    print(json.dumps({"passed": passed, "total": len(results), "ok": passed == len(results)}, sort_keys=True))
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
