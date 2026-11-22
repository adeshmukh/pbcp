#!/usr/bin/env bash
set -euo pipefail

: "${GH_REPO:?}" "${GITHUB_SHA:?}" "${RELEASE_TAG:?}" "${VERSION:?}"

state=$(gh api "repos/$GH_REPO/releases" --paginate --jq \
  '.[] | select(.tag_name == env.RELEASE_TAG) | if .draft then (if .target_commitish == env.GITHUB_SHA then "draft" else "conflicting-draft" end) else "published" end')
case "$state" in
  published)
    echo "Release $RELEASE_TAG already published; leaving it unchanged."
    exit 0
    ;;
  draft)
    gh release upload "$RELEASE_TAG" ./*.tar.gz SHA256SUMS --clobber
    ;;
  conflicting-draft)
    echo "::error::Draft $RELEASE_TAG belongs to another commit; rerun its failed workflow or remove that unpublished draft."
    exit 1
    ;;
  "")
    notes=(--generate-notes --notes-file ../docs/releases/RELEASE_NOTES.md)
    if [[ -n "${PREVIOUS_TAG:-}" ]]; then
      notes+=(--notes-start-tag "$PREVIOUS_TAG")
    fi
    gh release create "$RELEASE_TAG" ./*.tar.gz SHA256SUMS \
      --target "$GITHUB_SHA" --title "pbcp $VERSION" --draft "${notes[@]}"
    ;;
  *)
    echo "::error::Unexpected release state: $state"
    exit 1
    ;;
esac
gh release edit "$RELEASE_TAG" --draft=false --latest
