#!/bin/sh

# The file is used by ./.github/workflows/binaries.yaml to install build dependencies
# as part of a GitHub Action.
# ---

echo "Installing build dependencies"

# Install the protobuf compiler and development libraries
apk add protoc protobuf-dev

# Proto files are stored in the ./proto directory so we need to create it before the git submodule pull
mkdir -p ./proto

# rustup update
# cargo clean