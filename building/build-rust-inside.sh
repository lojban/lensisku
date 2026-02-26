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

# Clean out the old build
rm -rf target

# Set C++ standard to C++17 for dependencies that compile C++ code
# This is required for tectonic_xetex_layout which uses ICU headers
# that require C++17 features (auto template parameters)
export CXXFLAGS="-std=c++17"

cargo build --release
