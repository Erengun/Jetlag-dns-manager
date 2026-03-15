#!/usr/bin/env bash

set -e

BASEDIR=$(dirname "$0")

if [[ -z "${CARGOKIT_TOOL_TEMP_DIR}" ]]; then
  echo "ERROR: CARGOKIT_TOOL_TEMP_DIR is not set." >&2
  exit 1
fi

mkdir -p "$CARGOKIT_TOOL_TEMP_DIR"

cd "$CARGOKIT_TOOL_TEMP_DIR"

# Write a very simple bin package in temp folder that depends on build_tool package
# from Cargokit. This is done to ensure that we don't pollute Cargokit folder
# with .dart_tool contents.

BUILD_TOOL_PKG_DIR="$BASEDIR/build_tool"

if [[ -z $FLUTTER_ROOT ]]; then
  echo "ERROR: FLUTTER_ROOT environment variable is not set." >&2
  exit 1
else
  DART="$FLUTTER_ROOT/bin/cache/dart-sdk/bin/dart"
fi

cat << EOF > "pubspec.yaml"
name: build_tool_runner
version: 1.0.0
publish_to: none

environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  build_tool:
    path: "$BUILD_TOOL_PKG_DIR"
EOF

mkdir -p "bin"

cat << EOF > "bin/build_tool_runner.dart"
import 'package:build_tool/build_tool.dart' as build_tool;
Future<void> main(List<String> args) async {
  await build_tool.runMain(args);
}
EOF

# Determine the shasum command: prefer shasum, fall back to sha1sum
if [ -x "$(command -v shasum)" ]; then
  SHASUM_CMD="shasum"
elif [ -x "$(command -v sha1sum)" ]; then
  SHASUM_CMD="sha1sum"
else
  echo "ERROR: Neither shasum nor sha1sum found in PATH." >&2
  exit 1
fi

# Dart run will not cache any package that has a path dependency, which
# is the case for our build_tool_runner. So instead we precompile the package
# ourselves.
# To invalidate the cached kernel we hash the directory listing of
# build_tool plus pubspec.yaml and bin/build_tool_runner.dart (matching
# the Windows script) using ls -lTR on macOS and ls -lR --full-time
# on other systems.

if [[ "$OSTYPE" == "darwin"* ]]; then
  PACKAGE_HASH=$( (ls -lTR "$BUILD_TOOL_PKG_DIR"; cat "pubspec.yaml" "bin/build_tool_runner.dart") | $SHASUM_CMD)
else
  PACKAGE_HASH=$( (ls -lR --full-time "$BUILD_TOOL_PKG_DIR"; cat "pubspec.yaml" "bin/build_tool_runner.dart") | $SHASUM_CMD)
fi

PACKAGE_HASH_FILE=".package_hash"

if [ -f "$PACKAGE_HASH_FILE" ]; then
    EXISTING_HASH=$(cat "$PACKAGE_HASH_FILE")
    if [ "$PACKAGE_HASH" != "$EXISTING_HASH" ]; then
        rm "$PACKAGE_HASH_FILE"
    fi
fi

# Run pub get if needed.
if [ ! -f "$PACKAGE_HASH_FILE" ]; then
    "$DART" pub get --no-precompile
    "$DART" compile kernel bin/build_tool_runner.dart
    echo "$PACKAGE_HASH" > "$PACKAGE_HASH_FILE"
fi

# Rebuild the tool if it was deleted by Android Studio
if [ ! -f "bin/build_tool_runner.dill" ]; then
  "$DART" compile kernel bin/build_tool_runner.dart
fi

set +e

"$DART" bin/build_tool_runner.dill "$@"

exit_code=$?

# 253 means invalid snapshot version.
if [ "$exit_code" -eq 253 ]; then
  "$DART" pub get --no-precompile
  "$DART" compile kernel bin/build_tool_runner.dart
  "$DART" bin/build_tool_runner.dill "$@"
  exit_code=$?
fi

exit $exit_code
