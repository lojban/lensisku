#!/usr/bin/env bash
# Refuses to commit obvious secrets. Default: scans the lines being added to the index.
# `scripts/check-secrets.sh --all` scans every tracked file instead.
# Hits print the file, line number and rule only, never the matched text.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2

# Skipped paths: this script, lockfiles, translations and generated or binary-ish data.
skip='^(\.env\.example|scripts/check-secrets\.sh|Cargo\.lock|frontend/pnpm-lock\.yaml|frontend/src/locales/|locales/|tests/fixtures/|docs/pitfalls\.md)'

rules=(
  'private key|-----BEGIN [A-Z ]*PRIVATE KEY-----'
  'AWS access key|AKIA[0-9A-Z]{16}'
  'API key (sk-)|sk-[A-Za-z0-9_-]{24,}'
  'GitHub token|gh[pousr]_[A-Za-z0-9]{30,}'
  'Slack token|xox[baprs]-[A-Za-z0-9-]{10,}'
  'credential assignment|(pass(word|wd)?|secret|token|api[_-]?key)[A-Za-z0-9_]*[[:space:]]*[:=][[:space:]]*["'"'"']?[A-Za-z0-9/+_.-]{16,}["'"'"']?[[:space:]]*$'
)
# Values that are obviously placeholders.
fake='(example|placeholder|changeme|your[_-]|xxxx|\.\.\.|<[^>]*>|\$\{|\$\(|test|dummy|fake|process\.env|env::var|import\.meta)'

code='\.(vue|ts|js|mjs|sql|rs)$'
status=0
scan() { # $1 file, $2 line number, $3 text
  local name pat entry
  for entry in "${rules[@]}"; do
    name=${entry%%|*}; pat=${entry#*|}
    [ "$name" = "credential assignment" ] && [[ $1 =~ $code ]] && continue
    if printf '%s\n' "$3" | grep -Eiq -- "$pat" && ! printf '%s\n' "$3" | grep -Eiq -- "$fake"; then
      printf 'possible secret (%s): %s:%s\n' "$name" "$1" "$2"
      status=1
    fi
  done
}

if [ "${1:-}" = "--all" ]; then
  # git grep is fast on large tracked files; print file:line and the rule only.
  for entry in "${rules[@]}"; do
    name=${entry%%|*}; pat=${entry#*|}
    while IFS= read -r hit; do
      f=${hit%%:*}; rest=${hit#*:}; ln=${rest%%:*}; text=${rest#*:}
      [[ $f =~ $skip ]] && continue
      [ "$name" = "credential assignment" ] && [[ $f =~ $code ]] && continue
      printf '%s\n' "$text" | grep -Eiq -- "$fake" && continue
      printf 'possible secret (%s): %s:%s\n' "$name" "$f" "$ln"
      status=1
    done < <(git grep -nIEi -e "$pat" 2>/dev/null | cut -c1-2000)
  done
else
  file=""; n=0
  while IFS= read -r line; do
    case $line in
      '+++ b/'*) file=${line#+++ b/}; continue ;;
      '+++ '*|'--- '*) continue ;;
      '@@ '*) n=$(printf '%s' "$line" | sed -E 's/^@@ -[0-9,]+ \+([0-9]+).*/\1/'); n=$((n-1)); continue ;;
      '+'*) n=$((n+1)); [[ $file =~ $skip ]] && continue; scan "$file" "$n" "${line#+}" ;;
    esac
  done < <(git diff --cached -U0 --no-color --diff-filter=AM)
fi

if [ "$status" -ne 0 ]; then
  echo "Commit blocked: remove the value (use .env or the server's secrets file) or, if it is a fake, make it obvious (e.g. sk-test)." >&2
fi
exit "$status"
