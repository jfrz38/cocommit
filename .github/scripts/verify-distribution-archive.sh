#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <archive> <version> <target>" >&2
  exit 2
fi

archive="$1"
version="$2"
target="$3"
root="cocommit-$version-$target"

case "$target" in
  x86_64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin)
    expected_binary='cocommit'
    actual="$(tar --list --gzip --file "$archive" | sed 's:/$::' | sort -u)"
    ;;
  x86_64-pc-windows-msvc)
    expected_binary='cocommit.exe'
    actual="$(unzip -Z1 "$archive" | sed 's:/$::' | sort -u)"
    ;;
  *)
    echo "Unsupported distribution target: $target" >&2
    exit 1
    ;;
esac

expected="$(printf '%s\n' "$root" "$root/INSTALL.md" "$root/LICENSE" "$root/$expected_binary")"
if ! diff --unified <(printf '%s\n' "$expected") <(printf '%s\n' "$actual"); then
  echo "Archive $archive does not match the expected layout." >&2
  exit 1
fi

work_directory="$(mktemp -d)"
trap 'rm -rf "$work_directory"' EXIT
if [[ "$archive" == *.tar.gz ]]; then
  tar --extract --gzip --file "$archive" -C "$work_directory"
else
  unzip -q "$archive" -d "$work_directory"
fi

test -f "$work_directory/$root/LICENSE"
grep -Fx "cocommit $version for $target" "$work_directory/$root/INSTALL.md" >/dev/null
grep -Fx 'This binary is not code signed or notarized.' "$work_directory/$root/INSTALL.md" >/dev/null
test -f "$work_directory/$root/$expected_binary"
echo 'Distribution archive layout is valid.'
