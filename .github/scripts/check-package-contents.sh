#!/usr/bin/env bash
set -euo pipefail

files="$(cargo package --list --allow-dirty | tr '\\' '/')"

required=(.cargo_vcs_info.json CHANGELOG.md Cargo.lock Cargo.toml Cargo.toml.orig LICENSE README.md docs/configuration.md src/lib.rs src/main.rs)
for file in "${required[@]}"; do
  grep -Fx -- "$file" <<<"$files" >/dev/null
done

if grep -Ev '^(\.cargo_vcs_info\.json|CHANGELOG.md|Cargo.lock|Cargo.toml|Cargo.toml.orig|LICENSE|README.md|docs/configuration\.md|src/.+\.rs)$' <<<"$files"; then
  echo 'The Cargo package contains files outside the approved distribution set.' >&2
  exit 1
fi

echo 'Cargo package contents match the distribution contract.'
