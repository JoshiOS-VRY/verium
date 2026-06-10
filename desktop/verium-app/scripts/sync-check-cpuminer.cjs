#!/usr/bin/env node
/**
 * Fail when cpuminer.lock.json drifts from veriumMiner/CMakeLists.txt (monorepo)
 * or when the sidecar on disk is missing / still a stub.
 *
 *   node scripts/sync-check-cpuminer.cjs
 *   node scripts/sync-check-cpuminer.cjs --lock-only
 */

const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const ROOT = path.resolve(__dirname, "..");
const LOCK_PATH = path.join(ROOT, "cpuminer.lock.json");
const MINER_CMAKE = path.resolve(ROOT, "../../../veriumMiner/CMakeLists.txt");
const MIN_BYTES = 100_000;
const lockOnly = process.argv.includes("--lock-only");

function fail(msg) {
  process.stderr.write(`[sync-check-cpuminer] ERROR: ${msg}\n`);
  process.exit(1);
}

function readLock() {
  if (!fs.existsSync(LOCK_PATH)) {
    fail(`missing ${LOCK_PATH}`);
  }
  return JSON.parse(fs.readFileSync(LOCK_PATH, "utf8"));
}

function cmakeVersion() {
  if (!fs.existsSync(MINER_CMAKE)) {
    return null;
  }
  const text = fs.readFileSync(MINER_CMAKE, "utf8");
  const m = text.match(/project\s*\(\s*veriumminer\s*\n\s*VERSION\s+([0-9.]+)/i);
  return m ? m[1] : null;
}

function detectTriple() {
  if (process.env.CPUMINER_TARGET_TRIPLE) return process.env.CPUMINER_TARGET_TRIPLE;
  try {
    const out = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
    const m = out.match(/^host:\s*(\S+)/m);
    if (m) return m[1];
  } catch (_) {}
  if (process.platform === "win32" && process.arch === "x64") {
    return "x86_64-pc-windows-msvc";
  }
  return `${process.arch}-${process.platform}`;
}

function sidecarPath(triple) {
  const ext = triple.includes("windows") ? ".exe" : "";
  return path.join(ROOT, "src-tauri", "binaries", `cpuminer-${triple}${ext}`);
}

function main() {
  const lock = readLock();
  if (!lock.version) fail("cpuminer.lock.json missing \"version\"");

  const minerVer = cmakeVersion();
  if (minerVer && minerVer !== lock.version) {
    fail(
      `cpuminer.lock.json pins ${lock.version} but veriumMiner/CMakeLists.txt is ${minerVer}. ` +
        "Bump cpuminer.lock.json and run npm run fetch:cpuminer.",
    );
  }

  if (lockOnly) {
    process.stdout.write(
      `[sync-check-cpuminer] OK lock ${lock.version}` +
        (minerVer ? ` matches CMakeLists ${minerVer}` : "") +
        "\n",
    );
    return;
  }

  const triple = detectTriple();
  const sidecar = sidecarPath(triple);
  if (!fs.existsSync(sidecar)) {
    fail(`sidecar missing at ${sidecar} — run npm run fetch:cpuminer`);
  }
  const size = fs.statSync(sidecar).size;
  if (size < MIN_BYTES) {
    fail(
      `sidecar at ${sidecar} is a stub (${size} bytes). ` +
        "Run npm run fetch:cpuminer or npm run fetch:cpuminer:local",
    );
  }

  process.stdout.write(
    `[sync-check-cpuminer] OK v${lock.version} sidecar (${(size / 1_000_000).toFixed(1)} MB)\n`,
  );
}

main();
