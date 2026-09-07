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
grep -q 'check advisories licenses sources' Makefile
grep -q 'cargo install cargo-deny --version 0.20.2 --locked' .github/workflows/release.yml
grep -q 'jfrz38/check-version-change@c087b716b54ed45079f2440924db036f636539b3 # v1.3.1' .github/workflows/release.yml
grep -q 'jfrz38/check-version-change@c087b716b54ed45079f2440924db036f636539b3 # v1.3.1' .github/workflows/publish.yml
grep -q 'compare-ref: HEAD\^' .github/workflows/release.yml
grep -q "steps.version-change.outputs.changed == 'true'" .github/workflows/release.yml
grep -q 'compare-source: registry' .github/workflows/publish.yml
grep -q 'fail-on-unchanged: true' .github/workflows/publish.yml
grep -q 'fail-on-not-higher: true' .github/workflows/publish.yml
grep -q 'releases/latest' .github/workflows/publish.yml
grep -q 'test "\$GITHUB_REF" = "refs/heads/main"' .github/workflows/publish.yml
if grep -q '^    inputs:$' .github/workflows/publish.yml; then
  echo 'Publish to crates.io must select the latest release rather than accept a tag input.' >&2
  exit 1
fi
grep -q 'runs-on: ubuntu-latest' .github/workflows/release.yml
grep -q 'runs-on: ubuntu-latest' .github/workflows/publish.yml

echo 'Workflow release contract checks passed.'
