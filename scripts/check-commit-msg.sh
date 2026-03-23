#!/usr/bin/env bash
#
# Validates that the commit message follows Conventional Commits.
#
# Allowed prefixes:
#   feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
#
# Format: <type>[optional scope]: <description>

commit_msg_file="$1"
commit_msg=$(head -1 "$commit_msg_file")

pattern="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .+"

if ! echo "$commit_msg" | grep -Eq "$pattern"; then
    echo "ERROR: Commit message does not follow Conventional Commits."
    echo ""
    echo "  Expected: <type>[scope]: <description>"
    echo "  Got:      $commit_msg"
    echo ""
    echo "  Allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert"
    echo ""
    echo "  Examples:"
    echo "    feat: add validation engine"
    echo "    fix(parser): handle empty ZIP archives"
    echo "    docs: update README installation section"
    exit 1
fi
