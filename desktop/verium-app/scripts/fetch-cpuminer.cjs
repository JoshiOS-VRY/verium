#!/usr/bin/env node
/**
 * Fetch or copy veriumMiner (cpuminer) into src-tauri/binaries/cpuminer-<triple>{.exe}
 *
 * Version is pinned in cpuminer.lock.json (never duplicate miner source in this repo).
 *
 * Usage:
 *   node scripts/fetch-cpuminer.cjs
 *   node scripts/fetch-cpuminer.cjs --monorepo     # prefer ../../../veriumMiner build
 *   CPUMINER_LOCAL=/path/to/cpuminer node scripts/fetch-cpuminer.cjs
 *   CPUMINER_REQUIRE=1 node scripts/fetch-cpuminer.cjs   # fail instead of stub (CI)
 */

const fs = require("node:fs");
const path = require("node:path");
const { execFileSync, spawnSync } = require("node:child_process");
const https = require("node:https");

const ROOT = path.resolve(__dirname, "..");
const BINARIES = path.join(ROOT, "src-tauri", "binaries");
const LOCK_PATH = path.join(ROOT, "cpuminer.lock.json");
const MONOREPO_MINER = path.resolve(ROOT, "../../../veriumMiner");

function log(msg) {
  process.stdout.write(`[fetch-cpuminer] ${msg}\n`);
}

function readLock() {
  if (!fs.existsSync(LOCK_PATH)) {
    throw new Error(`missing ${LOCK_PATH}`);
  }
  const lock = JSON.parse(fs.readFileSync(LOCK_PATH, "utf8"));
  if (!lock.version) throw new Error("cpuminer.lock.json missing version");
  lock.repo = lock.repo || "JoshiOS-VRY/veriumMiner";
  return lock;
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
  throw new Error("Could not detect target triple; set CPUMINER_TARGET_TRIPLE");
}

function sidecarPath(triple) {
  const ext = triple.includes("windows") ? ".exe" : "";
  return path.join(BINARIES, `cpuminer-${triple}${ext}`);
}

function releasePlatformFragment(triple) {
  if (triple.includes("windows") && triple.includes("x86_64")) return "windows-x86_64";
  if (triple.includes("windows") && triple.includes("aarch64")) return "windows-arm64";
  if (triple.includes("apple") && triple.includes("aarch64")) return "macos-arm64";
  if (triple.includes("apple") && triple.includes("x86_64")) return "macos-x86_64";
  if (triple.includes("linux") && triple.includes("aarch64")) return "linux-arm64";
  if (triple.includes("linux")) return "linux-x86_64";
  return triple;
}

function pickReleaseAsset(assets, triple, version) {
  const platform = releasePlatformFragment(triple);
  const ver = String(version).replace(/^v/, "");
  const preferred = [
    `veriumminer-${ver}-${platform}.zip`,
    `veriumminer-${ver}-${platform}.tar.gz`,
    `veriumminer-v${ver}-${platform}.zip`,
    `veriumminer-v${ver}-${platform}.tar.gz`,
  ].map((n) => n.toLowerCase());

  for (const want of preferred) {
    const hit = assets.find((a) => a.name.toLowerCase() === want);
    if (hit) return hit;
  }

  return assets.find((a) => {
    const n = a.name.toLowerCase();
    return (
      n.includes("veriumminer") &&
      n.includes(platform) &&
      (n.endsWith(".zip") || n.endsWith(".tar.gz"))
    );
  });
}

function findFileRecursive(dir, name) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const hit = findFileRecursive(full, name);
      if (hit) return hit;
    } else if (entry.name.toLowerCase() === name.toLowerCase()) {
      return full;
    }
  }
  return null;
}

function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "vericonomy-wallet" } }, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          return resolve(fetchJson(res.headers.location));
        }
        const chunks = [];
        res.on("data", (c) => chunks.push(c));
        res.on("end", () => {
          try {
            resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
          } catch (e) {
            reject(e);
          }
        });
      })
      .on("error", reject);
  });
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "vericonomy-wallet" } }, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          return resolve(download(res.headers.location, dest));
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      })
      .on("error", reject);
  });
}

function writeStub(dest) {
  const stub =
    process.platform === "win32"
      ? Buffer.from("MZ\x00\x00stub cpuminer — npm run fetch:cpuminer\n")
      : Buffer.from("#!/bin/sh\necho 'stub cpuminer' >&2\nexit 1\n");
  fs.writeFileSync(dest, stub);
  log(`Wrote build placeholder stub at ${dest}`);
}

function copyBinary(src, dest) {
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }
  log(`Installed ${dest} from ${src}`);
}

function monorepoBuildCandidates() {
  const exe = process.platform === "win32" ? "cpuminer.exe" : "cpuminer";
  const dirs = ["build", "build-msys", "build-release"];
  const out = [];
  for (const dir of dirs) {
    out.push(path.join(MONOREPO_MINER, dir, exe));
  }
  return out;
}

function resolveMonorepoBinary() {
  if (!fs.existsSync(MONOREPO_MINER)) return null;
  for (const candidate of monorepoBuildCandidates()) {
    if (fs.existsSync(candidate) && fs.statSync(candidate).size > 100_000) {
      return candidate;
    }
  }
  return null;
}

function extractArchive(archivePath, triple, dest) {
  const binName = triple.includes("windows") ? "cpuminer.exe" : "cpuminer";
  const extractDir = path.join(BINARIES, "cpuminer-extract");
  fs.rmSync(extractDir, { recursive: true, force: true });
  fs.mkdirSync(extractDir, { recursive: true });

  if (archivePath.endsWith(".zip")) {
    if (process.platform === "win32") {
      execFileSync(
        "powershell",
        [
          "-NoProfile",
          "-Command",
          `Expand-Archive -Force -Path '${archivePath}' -DestinationPath '${extractDir}'`,
        ],
        { stdio: "inherit" },
      );
    } else {
      spawnSync("unzip", ["-q", archivePath, "-d", extractDir], { stdio: "inherit" });
    }
  } else if (archivePath.endsWith(".tar.gz")) {
    spawnSync("tar", ["-xzf", archivePath, "-C", extractDir], { stdio: "inherit" });
  } else {
    throw new Error(`unsupported archive: ${archivePath}`);
  }

  const extracted = findFileRecursive(extractDir, binName);
  if (!extracted) throw new Error(`${binName} not found in release archive`);
  copyBinary(extracted, dest);
  fs.rmSync(extractDir, { recursive: true, force: true });
}

async function fetchRelease(dest, lock, triple) {
  const version = process.env.CPUMINER_VERSION || lock.version;
  const repo = process.env.CPUMINER_REPO || lock.repo;
  const release = await fetchJson(
    `https://api.github.com/repos/${repo}/releases/tags/v${version.replace(/^v/, "")}`,
  );
  const assets = release.assets || [];
  const asset = pickReleaseAsset(assets, triple, version);
  if (!asset) {
    const names = assets.map((a) => a.name).join(", ") || "(none)";
    throw new Error(
      `No release asset for ${triple} in ${repo} v${version}. Assets: ${names}`,
    );
  }

  const tmp = path.join(BINARIES, asset.name);
  log(`Downloading ${repo} v${version} → ${asset.name}`);
  await download(asset.browser_download_url, tmp);
  extractArchive(tmp, triple, dest);
  fs.unlinkSync(tmp);
}

async function main() {
  const requireReal =
    process.env.CPUMINER_REQUIRE === "1" || process.argv.includes("--require");
  const useMonorepo =
    process.env.CPUMINER_MONOREPO === "1" || process.argv.includes("--monorepo");
  const skipIfPresent =
    process.env.CPUMINER_SKIP_IF_PRESENT === "1" ||
    process.argv.includes("--skip-if-present");
  const writeStubOnly =
    process.env.CPUMINER_STUB === "1" || process.argv.includes("--stub");
  const force =
    process.env.CPUMINER_FORCE === "1" || process.argv.includes("--force");

  const lock = readLock();
  const triple = detectTriple();
  const dest = sidecarPath(triple);
  fs.mkdirSync(BINARIES, { recursive: true });

  if (writeStubOnly) {
    writeStub(dest);
    return;
  }

  if (!force && skipIfPresent && fs.existsSync(dest) && fs.statSync(dest).size > 100_000) {
    log(`Skip — already present: ${dest}`);
    return;
  }

  if (process.env.CPUMINER_LOCAL) {
    const src = path.resolve(process.env.CPUMINER_LOCAL);
    if (!fs.existsSync(src)) throw new Error(`CPUMINER_LOCAL not found: ${src}`);
    copyBinary(src, dest);
    return;
  }

  if (useMonorepo) {
    const built = resolveMonorepoBinary();
    if (built) {
      copyBinary(built, dest);
      return;
    }
    log(`No monorepo build in ${MONOREPO_MINER} — falling back to release v${lock.version}`);
  }

  try {
    await fetchRelease(dest, lock, triple);
  } catch (e) {
    if (requireReal) {
      throw e;
    }
    log(`Fetch failed (${e.message}) — writing stub so Tauri can compile.`);
    log("Pool mining will use the in-process veriumd fallback until fetch succeeds.");
    writeStub(dest);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
