# Commit Message Generation Rules
# Goal: Generate a concise, consistent commit message for org-mode note changes

## Input Context
You have been provided the git log via 'git log -n 10' and the git diff via 'git diff --staged -U30'.
Use the previous commits to keep the same style.

## Rules

### 1. Output Format
Only output the raw commit message, without any explanation or backticks.

### 2. Message Style
Describe changes in natural language, based on the context.
