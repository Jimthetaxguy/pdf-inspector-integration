#!/usr/bin/env bash
set -euo pipefail

# Keep obvious private identifiers out of candidate text. Split literal path
# and key markers so this policy file does not match its own rules.
patterns=(
  '/''Users/'
  '/home/[[:alnum:]_.-]+/'
  '[A-Za-z]:\\Users\\[[:alnum:]_.-]+\\'
  '[[:alnum:]._%+-]+@[[:alnum:].-]+\.[[:alpha:]]{2,}'
  '(^|[^0-9])[0-9]{3}-[0-9]{2}-[0-9]{4}([^0-9]|$)'
  '(^|[^0-9])[0-9]{2}-[0-9]{7}([^0-9]|$)'
  '-----BEGIN ''([A-Z ]+ )?PRIVATE KEY-----'
  '\.claude/''plans/'
  '\.codex/''sessions/'
)

pattern=$(IFS='|'; printf '%s' "${patterns[*]}")

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf '%s\n' 'Public-hygiene check requires a Git worktree; refusing to pass.' >&2
  exit 2
}
cd "$repo_root"

# Scan the index separately so staged content cannot be hidden by a clean
# unstaged copy. Quiet mode prevents a match from echoing private content.
if git grep --cached -qI -E "$pattern" -- . >/dev/null 2>&1; then
  printf '%s\n' \
    'Public-hygiene check failed: staged content contains a disallowed identifier.' >&2
  exit 1
else
  status=$?
  if [[ $status -ne 1 ]]; then
    printf '%s\n' 'Public-hygiene index scan failed; refusing to pass.' >&2
    exit 2
  fi
fi

# Scan tracked working copies and untracked, non-ignored candidates. The child
# emits no paths or matching lines: a sensitive filename is itself private.
if ! git ls-files --cached --others --exclude-standard -z |
  xargs -0 sh -c '
    pattern=$1
    shift
    for path do
      if printf "%s\n" "$path" | grep -qE "$pattern"; then
        exit 10
      fi

      disk_path=./$path
      if [ -L "$disk_path" ]; then
        link_target=$(readlink "$disk_path") || exit 11
        if printf "%s\n" "$link_target" | grep -qE "$pattern"; then
          exit 10
        fi
        continue
      fi

      [ -e "$disk_path" ] || continue
      grep -IqE "$pattern" -- "$disk_path" >/dev/null 2>&1
      status=$?
      case $status in
        0) exit 10 ;;
        1) ;;
        *) exit 11 ;;
      esac
    done
  ' public-hygiene-scan "$pattern" 2>/dev/null
then
  printf '%s\n' \
    'Public-hygiene candidate scan failed or found a disallowed identifier; refusing to pass.' >&2
  exit 1
fi

printf '%s\n' 'Candidate-text obvious-identifier check passed.'
