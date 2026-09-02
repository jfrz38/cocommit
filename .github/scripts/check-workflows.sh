#!/usr/bin/env bash
set -euo pipefail

workflows=(.github/workflows/*.yml)

if grep -nE '^[[:space:]]*uses:' "${workflows[@]}" | grep -vE '@[0-9a-f]{40}[[:space:]]+# v[0-9]'; then
  echo 'Every GitHub Action must use a full SHA and a readable version comment.' >&2
  exit 1
fi

grep -q '^  supply-chain:$' .github/workflows/ci.yml
grep -A8 '^  supply-chain:$' .github/workflows/ci.yml | grep -q 'contents: read'
grep -A8 '^  publish:$' .github/workflows/publish.yml | grep -q 'environment: crates-publish'
grep -A8 '^  attest-assets:$' .github/workflows/release.yml | grep -q 'attestations: write'
grep -q 'check advisories licenses sources' Makefile
grep -q 'candidate_sha' .github/workflows/release.yml
grep -q 'SHA256SUMS' .github/workflows/release.yml
grep -q 'attestation verify' .github/workflows/publish.yml
grep -q 'x86_64-unknown-linux-gnu' .github/workflows/release.yml
grep -q 'x86_64-pc-windows-msvc' .github/workflows/release.yml
grep -q 'x86_64-apple-darwin' .github/workflows/release.yml
grep -q 'aarch64-apple-darwin' .github/workflows/release.yml
grep -q '^  assemble-release-assets:$' .github/workflows/release.yml
grep -q '^  verify-release-assets:$' .github/workflows/release.yml
grep -A8 '^  build-binaries:$' .github/workflows/release.yml | grep -q 'contents: read'
grep -A8 '^  verify-release-assets:$' .github/workflows/release.yml | grep -q 'contents: read'
grep -q 'verify-release-assets.sh' .github/workflows/release.yml
grep -q 'verify-release-assets.sh' .github/workflows/publish.yml

echo 'Workflow supply-chain contract checks passed.'
