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

echo 'Workflow supply-chain contract checks passed.'
