#!/usr/bin/env bash

set -euo pipefail

{
  echo "---GIT_LOG_START---"
  git log -n 10
  echo "---GIT_LOG_END---"
  echo "---GIT_DIFF_START---"
  git diff --staged -U30
  echo "---GIT_DIFF_END---"
  cat gitwatch-prompt.md
} | aichat --model claude:claude-3-5-haiku-latest --no-stream
