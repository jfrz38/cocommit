#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <assets-directory> <version>" >&2
  exit 2
fi

assets="$1"
version="$2"
targets=(
  x86_64-unknown-linux-gnu
  x86_64-pc-windows-msvc
  x86_64-apple-darwin
  aarch64-apple-darwin
)

expected=(
  "cocommit-$version.crate"
  "cocommit-$version.cdx.json"
  "cocommit-$version-SHA256SUMS"
)
for target in "${targets[@]}"; do
  case "$target" in
    x86_64-pc-windows-msvc) expected+=("cocommit-$version-$target.zip") ;;
    *) expected+=("cocommit-$version-$target.tar.gz") ;;
  esac
done

actual="$(find "$assets" -maxdepth 1 -type f -printf '%f\n' | sort)"
if ! diff --unified <(printf '%s\n' "${expected[@]}" | sort) <(printf '%s\n' "$actual"); then
  echo 'Release assets do not match the approved distribution contract.' >&2
  exit 1
fi

(cd "$assets" && sha256sum --check "cocommit-$version-SHA256SUMS")
jq -e '.bomFormat == "CycloneDX"' "$assets/cocommit-$version.cdx.json" >/dev/null
for target in "${targets[@]}"; do
  case "$target" in
    x86_64-pc-windows-msvc) archive="$assets/cocommit-$version-$target.zip" ;;
    *) archive="$assets/cocommit-$version-$target.tar.gz" ;;
  esac
  .github/scripts/verify-distribution-archive.sh "$archive" "$version" "$target"
done

echo 'Release assets match the distribution contract.'
