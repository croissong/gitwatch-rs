#!/usr/bin/env bash

command rm -r /tmp/gitwatch-ai
mkdir -p /tmp/gitwatch-ai

git log -n 100 >/tmp/gitwatch-ai/git-log.txt
git diff --staged -U30 >/tmp/gitwatch-ai/git-diff.txt


cat (status dirname)/gitwatch-prompt.txt | aichat --model claude:claude-3-5-haiku-latest --no-stream -f /tmp/gitwatch-ai/
