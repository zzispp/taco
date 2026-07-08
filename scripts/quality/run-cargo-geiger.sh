#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
report_dir="$repo_root/target/quality"
report_path="$report_dir/cargo-geiger-report.json"
geiger_log_path="$report_dir/cargo-geiger-stderr.log"
manifest_path="$repo_root/apps/backend/Cargo.toml"

mkdir -p "$report_dir"

# cargo-geiger 0.13 cannot use a virtual workspace manifest as its root package.
# The backend crate is the composition root and depends on the workspace crates
# that form the runtime dependency graph, so this keeps the workspace scan real.
if ! cargo geiger --all-features --workspace --manifest-path "$manifest_path" --output-format Json > "$report_path" 2> "$geiger_log_path"; then
    cat "$geiger_log_path" >&2
    exit 1
fi

python3 - "$repo_root" "$report_path" "$geiger_log_path" <<'PY'
import json
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

repo_root = Path(sys.argv[1]).resolve()
report_path = Path(sys.argv[2]).resolve()
geiger_log_path = Path(sys.argv[3]).resolve()

with report_path.open() as report_file:
    report = json.load(report_file)

packages = report.get("packages", [])
if not packages:
    raise SystemExit("cargo-geiger produced an empty package report")

packages_without_metrics = report.get("packages_without_metrics", [])
if packages_without_metrics:
    raise SystemExit(f"cargo-geiger missed package metrics: {len(packages_without_metrics)}")


def package_path(package_id: dict) -> Path | None:
    source = package_id.get("source", {})
    raw_path = source.get("Path") if isinstance(source, dict) else None
    if not raw_path:
        return None
    parsed = urlparse(raw_path)
    if parsed.scheme != "file":
        return None
    return Path(unquote(parsed.path)).resolve()


def unsafe_total(counter_block: dict) -> int:
    return sum(int(counter.get("unsafe_", 0)) for counter in counter_block.values())

workspace_unsafe = []
dependency_unsafe = 0
for entry in packages:
    package = entry["package"]
    package_id = package["id"]
    unsafety = entry["unsafety"]
    unsafe_count = unsafe_total(unsafety["used"]) + unsafe_total(unsafety["unused"])
    path = package_path(package_id)
    is_workspace = path is not None and (path == repo_root or repo_root in path.parents)
    if is_workspace and unsafe_count:
        workspace_unsafe.append(f"{package_id['name']}:{unsafe_count}")
    if not is_workspace and unsafe_total(unsafety["used"]):
        dependency_unsafe += 1

used_but_not_scanned = report.get("used_but_not_scanned_files", [])
workspace_unscanned = []
for raw_path in used_but_not_scanned:
    path = Path(raw_path).resolve()
    if path == repo_root or repo_root in path.parents:
        try:
            path.relative_to(repo_root / "target")
        except ValueError:
            workspace_unscanned.append(str(path))

print(f"cargo-geiger report: {report_path}")
print(f"cargo-geiger stderr log: {geiger_log_path}")
print(f"cargo-geiger packages scanned: {len(packages)}")
print(f"cargo-geiger dependency packages with used unsafe: {dependency_unsafe}")
print(f"cargo-geiger dependency files reported as used-but-not-scanned: {len(used_but_not_scanned)}")

if workspace_unscanned:
    raise SystemExit("workspace source files were not scanned: " + ", ".join(workspace_unscanned[:10]))
if workspace_unsafe:
    raise SystemExit("workspace packages contain unsafe code: " + ", ".join(workspace_unsafe))
PY
