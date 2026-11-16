#!/usr/bin/env python3
"""Assert expected properties of Rust C2 NDJSON replay output."""
import argparse
import json
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", help="Rust C2 output NDJSON")
    parser.add_argument("--min-updates", type=int, default=1)
    parser.add_argument("--require-recommendation", action="append", default=[])
    args = parser.parse_args()

    updates = []
    with open(args.input, encoding="utf-8") as stream:
        for line in stream:
            event = json.loads(line)
            if event.get("kind") == "track_update":
                updates.append(event)

    if len(updates) < args.min_updates:
        print(f"expected at least {args.min_updates} track updates, got {len(updates)}", file=sys.stderr)
        return 1
    recommendations = {event.get("recommendation") for event in updates}
    missing = [name for name in args.require_recommendation if name not in recommendations]
    if missing:
        print(f"missing recommendations: {', '.join(missing)}", file=sys.stderr)
        return 1
    print(f"replay assertions passed: {len(updates)} track updates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
