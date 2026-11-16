#!/usr/bin/env python3
"""Transform synthetic NDJSON observations for deterministic resilience tests.

This tool only changes checked-in or generated synthetic scenarios. It never
connects to sensors, networks, RF systems, or effectors.
"""
import argparse
import json
import sys
from typing import Iterable


def transform(lines: Iterable[str], drop_every: int, delay_ms: int, contradict_every: int):
    emitted = 0
    for index, raw in enumerate(lines, start=1):
        if not raw.strip():
            continue
        event = json.loads(raw)
        if event.get("kind") != "observation":
            yield event
            continue
        if drop_every and index % drop_every == 0:
            continue
        event = dict(event)
        event["event_id"] = f"fault-{event.get('event_id', index)}"
        if delay_ms:
            event["timestamp_ms"] = int(event.get("timestamp_ms", 0)) + delay_ms
            event["fault_injected"] = "delay"
        if contradict_every and index % contradict_every == 0:
            position = dict(event["position"])
            position["x_m"] = float(position["x_m"]) * -1.0
            position["y_m"] = float(position["y_m"]) * -1.0
            event["position"] = position
            event["confidence"] = min(float(event.get("confidence", 0.0)), 0.25)
            event["fault_injected"] = "contradiction"
        emitted += 1
        yield event


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", help="input NDJSON scenario, or - for stdin")
    parser.add_argument("--output", default="-", help="output NDJSON path, default stdout")
    parser.add_argument("--drop-every", type=int, default=0, metavar="N", help="drop every Nth observation")
    parser.add_argument("--delay-ms", type=int, default=0, help="add a fixed timestamp delay")
    parser.add_argument("--contradict-every", type=int, default=0, metavar="N", help="invert position and lower confidence every Nth observation")
    args = parser.parse_args()
    for name, value in (("drop-every", args.drop_every), ("contradict-every", args.contradict_every)):
        if value < 0:
            parser.error(f"--{name} must be non-negative")
    if args.delay_ms < 0:
        parser.error("--delay-ms must be non-negative")

    source = sys.stdin if args.input == "-" else open(args.input, encoding="utf-8")
    output = sys.stdout if args.output == "-" else open(args.output, "w", encoding="utf-8")
    try:
        for event in transform(source, args.drop_every, args.delay_ms, args.contradict_every):
            output.write(json.dumps(event, separators=(",", ":")) + "\n")
    finally:
        if source is not sys.stdin:
            source.close()
        if output is not sys.stdout:
            output.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
