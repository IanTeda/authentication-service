#!/bin/sh

# The file is used by ./.github/workflows/binaries.yaml to install build dependencies
# as part of a GitHub Action.
# ---

echo "Installing build dependencies"
apk add protoc protobuf-dev