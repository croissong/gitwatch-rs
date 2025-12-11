#!/usr/bin/env bash

set -euo pipefail

if ! command -v aichat &>/dev/null; then
  echo "Install & setup aichat for the complete example" >&2
  echo "Generated commit message"
  exit 0
fi

{
  echo "---GIT_LOG_START---"
  git log -n 10
  echo "---GIT_LOG_END---"
  echo "---GIT_DIFF_START---"
  git diff --staged -U30
  echo "---GIT_DIFF_END---"
  cat gitwatch-prompt.md
} | aichat --no-stream
