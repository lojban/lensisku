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

cd "$scriptdir/.."

# Set C++ standard to C++17 for dependencies that compile C++ code
export CXXFLAGS="${CXXFLAGS:--std=c++17}"

# Skip wiping target for much faster incremental rebuilds; set CLEAN=1 for a clean build
if [[ "${CLEAN:-0}" == "1" ]]; then
  rm -rf target
fi

# Default: release (binary at target/release/lensisku for run_program). Set CARGO_PROFILE=release-fast for faster builds (binary at target/release-fast/lensisku).
profile="${CARGO_PROFILE:-release}"
if [[ "$profile" == "release" ]]; then
  cargo build --release
else
  cargo build --profile "$profile"
fi
