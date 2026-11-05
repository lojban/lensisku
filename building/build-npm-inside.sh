#!/bin/bash

# Error trapping from https://gist.github.com/oldratlee/902ad9a398affca37bfcfab64612e7d1
__error_trapper() {
  local parent_lineno="$1"
  local code="$2"
  local commands="$3"
  echo "error exit status $code, at file $0 on or near line $parent_lineno: $commands"
}
trap '__error_trapper "${LINENO}/${BASH_LINENO}" "$?" "$BASH_COMMAND"' ERR

set -euE -o pipefail
shopt -s failglob

scriptdir="$(readlink -f "$(dirname "$0")")"

cd "$scriptdir/../frontend"

# Build away from the directory nginx serves. Keep its old hashed assets for open tabs.
rm -rf node_modules
staging_dir="$(mktemp -d "$PWD/.frontend-build.XXXXXX")"
trap 'rm -rf -- "$staging_dir"' EXIT

# Activate the packageManager pin via Corepack so pnpm does not attempt a broken
# self-switch into ~/.local/share/pnpm/.tools (ENOEXEC on Alpine).
corepack enable
corepack prepare "$(node -p "require('./package.json').packageManager")" --activate

pnpm install --frozen-lockfile

FRONTEND_BUILD_OUT_DIR="$staging_dir" pnpm run build
node "$scriptdir/publish-frontend.mjs" "$staging_dir" "$PWD/dist"
