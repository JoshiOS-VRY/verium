#!/usr/bin/env bash
# Point the Xcode "Build Rust Code" phase at ios-xcode-prebuild-rust.sh (no tauri WebSocket).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APPLE_DIR="${ROOT}/src-tauri/gen/apple"
PBX="${APPLE_DIR}/vericonomy-wallet.xcodeproj/project.pbxproj"
YML="${APPLE_DIR}/project.yml"
PREBUILD_REL='${SRCROOT}/../../../scripts/ios-xcode-prebuild-rust.sh'

if [[ ! -f "${PBX}" ]]; then
  echo "error: missing ${PBX} — run npm run tauri:ios:init first." >&2
  exit 1
fi

chmod +x "${ROOT}/scripts/ios-xcode-prebuild-rust.sh"

python3 - "${PBX}" "${YML}" "${PREBUILD_REL}" <<'PY'
import pathlib
import re
import sys

pbx_path = pathlib.Path(sys.argv[1])
yml_path = pathlib.Path(sys.argv[2])
prebuild_rel = sys.argv[3]
content = pbx_path.read_text(encoding="utf-8")

# Must end with `";` — a missing semicolon breaks the whole Xcode project.
new_line = f'shellScript = "bash \\"{prebuild_rel}\\";'

if "ios-xcode-prebuild-rust.sh" in content and new_line not in content:
    # Repair a prior broken patch (truncated closing quote/semicolon).
    repaired, n = re.subn(
        r'shellScript = ".*ios-xcode-prebuild-rust\.sh.*?(?:"|;)',
        new_line,
        content,
        count=1,
    )
    if n == 1:
        pbx_path.write_text(repaired, encoding="utf-8")
        print("Repaired project.pbxproj shellScript line")
        content = repaired
    else:
        print("error: found ios-xcode-prebuild-rust.sh but could not repair shellScript", file=sys.stderr)
        sys.exit(1)
elif "tauri ios xcode-script" in content:
    updated, n = re.subn(
        r'shellScript = ".*?";',
        new_line,
        content,
        count=1,
        flags=re.DOTALL,
    )
    if n != 1:
        print("error: could not patch shellScript in project.pbxproj", file=sys.stderr)
        sys.exit(1)
    pbx_path.write_text(updated, encoding="utf-8")
    print("Patched project.pbxproj to use prebuilt libapp.a")
elif new_line in content:
    print("project.pbxproj already patched")
else:
    print("warning: unexpected Xcode project layout — Rust prebuild patch skipped", file=sys.stderr)

if yml_path.is_file():
    yml = yml_path.read_text(encoding="utf-8")
    marker = "tauri ios xcode-script"
    if marker in yml:
        lines = yml.splitlines()
        out = []
        skip = False
        for line in lines:
            if marker in line:
                out.append(f'          bash "{prebuild_rel}"')
                skip = True
                continue
            if skip:
                if line.strip().startswith("name:"):
                    skip = False
                    out.append(line)
                continue
            out.append(line)
        yml_path.write_text("\n".join(out) + "\n", encoding="utf-8")
        print("Patched project.yml")
PY
