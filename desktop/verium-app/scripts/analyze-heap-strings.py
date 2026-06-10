#!/usr/bin/env python3
"""Analyze Chrome/WebView heap snapshot strings export."""

import json
import re
import collections
import sys
from pathlib import Path


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(
        r"c:\Users\josh\OneDrive\Desktop\HeapSnapshot-strings-20260609T092603.json"
    )
    print(f"Loading {path}...")
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)

    print(f"Total strings: {len(data):,}")

    lens = [len(s) if isinstance(s, str) else 0 for s in data]
    total_bytes = sum(lens)
    print(f"Total char bytes: {total_bytes / 1024 / 1024:.1f} MB")
    print(f"Avg len: {sum(lens) / len(lens):.0f}")
    print(f"Max len: {max(lens):,}")
    print(f"Strings >10KB: {sum(1 for l in lens if l > 10000):,}")
    print(f"Strings >100KB: {sum(1 for l in lens if l > 100000):,}")

    largest = sorted(
        enumerate(data),
        key=lambda x: len(x[1]) if isinstance(x[1], str) else 0,
        reverse=True,
    )[:25]
    print("\n=== Top 25 largest strings (prefix) ===")
    for _, s in largest:
        if not isinstance(s, str):
            continue
        prefix = s[:120].replace("\n", " ")
        print(f"{len(s):>8,}  {prefix}...")

    patterns = [
        ("vite_hmr", r"createHotContext|/@vite/client|import\.meta\.hot"),
        ("source_map_base64", r"sourceMappingURL=data:application/json;base64"),
        ("react_query", r"@tanstack_react-query|QueryClient|queryKey"),
        ("tauri_invoke", r"@tauri-apps|invoke\("),
        ("listtransactions", r"listtransactions|list_transactions"),
        ("listunspent", r"listunspent"),
        ("explorer", r"explorer-blocks|explorer-stats|fetchExplorer"),
        ("getpeerinfo", r"getpeerinfo|get_peer_info"),
        ("getblockchaininfo", r"getblockchaininfo"),
        ("getmininginfo", r"getmininginfo"),
        ("memory_telemetry", r"\[memory\]|memory-profiler|MemoryDiagnostics"),
        ("json_rpc", r'"jsonrpc"|"result"|"error"'),
        ("txid_hex", r"[a-f0-9]{64}"),
        ("lucide", r"lucide-react"),
        ("recharts", r"recharts|d3-"),
        ("mining_chart", r"MiningHashrateChart|HashSample"),
    ]

    bucket_bytes: collections.Counter[str] = collections.Counter()
    bucket_count: collections.Counter[str] = collections.Counter()
    for s in data:
        if not isinstance(s, str):
            continue
        sl = len(s)
        matched = False
        for name, pat in patterns:
            if re.search(pat, s, re.I):
                bucket_bytes[name] += sl
                bucket_count[name] += 1
                matched = True
                break
        if not matched:
            bucket_bytes["other"] += sl
            bucket_count["other"] += 1

    print("\n=== Retention by pattern (bytes) ===")
    for name, b in bucket_bytes.most_common():
        print(f"{name:22} {b / 1024 / 1024:8.2f} MB  ({bucket_count[name]:,} strings)")

    ctr = collections.Counter(data)
    dups = [
        (s, c, len(s) * c)
        for s, c in ctr.items()
        if c > 1 and isinstance(s, str)
    ]
    dups.sort(key=lambda x: x[2], reverse=True)
    print("\n=== Top 15 duplicated exact strings (wasted = len*count) ===")
    for s, c, waste in dups[:15]:
        preview = s[:90].replace("\n", " ")
        print(f"x{c:>4}  waste={waste / 1024:>7.1f}KB  len={len(s):>6,}  {preview}...")

    src_paths: collections.Counter[str] = collections.Counter()
    for s in data:
        if isinstance(s, str) and "verium-app/src" in s:
            m = re.search(r"verium-app/src/([^\s\"\\]+)", s)
            if m:
                src_paths[m.group(1).split("?")[0]] += len(s)
    print("\n=== Top 20 source paths by retained string bytes ===")
    for p, b in src_paths.most_common(20):
        print(f"{b / 1024:>8.1f} KB  {p}")

    # Length distribution
    buckets = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 16, 32, 64, 128, 256, 512, 1024]
    hist: collections.Counter[int] = collections.Counter()
    for l in lens:
        for b in buckets:
            if l <= b:
                hist[b] += 1
                break
    print("\n=== String length distribution ===")
    for b in buckets:
        print(f"  len<={b:4}: {hist[b]:,}")

    common = collections.Counter(s for s in data if isinstance(s, str) and len(s) <= 32)
    print("\n=== Top 30 most frequent strings (len<=32) ===")
    for s, c in common.most_common(30):
        preview = repr(s)[:90]
        print(f"  x{c:>7,}  [{len(s):>2}] {preview}")

    addr = sum(
        1
        for s in data
        if isinstance(s, str) and re.match(r"^[VR][A-Za-z0-9]{25,40}$", s)
    )
    txid = sum(
        1 for s in data if isinstance(s, str) and re.match(r"^[a-f0-9]{64}$", s)
    )
    numeric = sum(
        1 for s in data if isinstance(s, str) and re.match(r"^-?\d+(\.\d+)?$", s)
    )
    print(f"\nWallet addresses (VR*): {addr:,}")
    print(f"Txids (64 hex): {txid:,}")
    print(f"Numeric strings: {numeric:,}")

    dev = sum(
        1
        for s in data
        if isinstance(s, str) and ("vite" in s.lower() or "/@vite/" in s)
    )
    mem = sum(1 for s in data if isinstance(s, str) and "[memory]" in s)
    print(f"Dev/vite-related strings: {dev:,}")
    print(f"[memory] telemetry strings: {mem:,}")

    tw_count = sum(
        1
        for s in data
        if isinstance(s, str) and ("flex " in s or "text-fg" in s or "border-" in s)
    )
    tw_bytes = sum(
        len(s)
        for s in data
        if isinstance(s, str) and ("flex " in s or "text-fg" in s or "border-" in s)
    )
    print(f"Tailwind class strings: {tw_count:,} ({tw_bytes / 1024 / 1024:.2f} MB)")

    # Estimate unique vs total for 1-char strings (internalized atom pressure)
    one_char = [s for s in data if isinstance(s, str) and len(s) == 1]
    print(f"\n1-char strings: {len(one_char):,} total, {len(set(one_char)):,} unique")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
