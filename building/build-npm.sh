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

LOCAL_DIR="$HOME/.local/npm-dotnpm"

if [[ ! -d $LOCAL_DIR ]]
then
  mkdir -p "$LOCAL_DIR"
  chcon -R -t container_file_t "$LOCAL_DIR"
fi

SCRIPT_DIR="$(readlink -f "$(dirname "$0")")"
SRC_DIR="$(readlink -f "$(dirname "$0")/..")"
PNPM_STORE="${PNPM_STORE:-$HOME/.local/share/pnpm/store}"

cd "$SCRIPT_DIR"

podman build -t "build-npm" --build-arg USERNAME="$(id -un)" --build-arg UID="$(id -u)" --build-arg GID="$(id -g)" -f Dockerfile.npm .

mkdir -p "$PNPM_STORE"
[[ -d $LOCAL_DIR ]] || mkdir -p "$LOCAL_DIR"
chcon -R -t container_file_t "$PNPM_STORE" 2>/dev/null || true

podman run --userns=keep-id --rm \
  -e COREPACK_ENABLE_DOWNLOAD_PROMPT=0 \
  -v "$SRC_DIR:/src" \
  -v "$LOCAL_DIR:/home/$(id -un)/.npm" \
  -v "$PNPM_STORE:/home/$(id -un)/.local/share/pnpm/store" \
  -w /src/frontend --entrypoint=/bin/bash -it build-npm ../building/build-npm-inside.sh
