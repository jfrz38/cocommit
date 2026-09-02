#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "Usage: $0 <version> <target> <binary> <output-directory>" >&2
  exit 2
fi

version="$1"
target="$2"
binary="$3"
output_directory="$4"

case "$target" in
  x86_64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin)
    archive="cocommit-$version-$target.tar.gz"
    binary_name="cocommit"
    ;;
  x86_64-pc-windows-msvc)
    archive="cocommit-$version-$target.zip"
    binary_name="cocommit.exe"
    ;;
  *)
    echo "Unsupported distribution target: $target" >&2
    exit 1
    ;;
esac

test -f "$binary"
mkdir -p "$output_directory"
output_directory="$(cd "$output_directory" && pwd)"
work_directory="$(mktemp -d)"
trap 'rm -rf "$work_directory"' EXIT

root="cocommit-$version-$target"
staging="$work_directory/$root"
mkdir -p "$staging"
cp "$binary" "$staging/$binary_name"
cp LICENSE "$staging/LICENSE"
printf '%s\n' \
  "cocommit $version for $target" \
  '' \
  'Extract this archive and add its directory to PATH.' \
  'Git must be installed and available on PATH.' \
  '' \
  'This binary is not code signed or notarized.' \
  'Verify the release checksum and GitHub Artifact Attestation before use.' \
  > "$staging/INSTALL.md"

chmod 0755 "$staging/$binary_name"
find "$staging" -exec touch -t 198001010000 {} +

if [[ "$archive" == *.tar.gz ]]; then
  tar --sort=name --mtime='@0' --owner=0 --group=0 --numeric-owner \
    --create --gzip --file "$output_directory/$archive" -C "$work_directory" "$root"
else
  (
    cd "$work_directory"
    zip -X -q "$output_directory/$archive" "$root/" "$root/LICENSE" "$root/INSTALL.md" "$root/$binary_name"
  )
fi

echo "$output_directory/$archive"
