#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
FFI_CRATE="flux-ffi"
LIB_NAME="libflux_ffi"
OUTPUT_DIR="$ROOT_DIR/target/ios"
HEADER_SRC="$ROOT_DIR/crates/bff/ffi/include/flux_ffi.h"

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
)

echo "==> Building $FFI_CRATE for iOS targets..."

for target in "${TARGETS[@]}"; do
    echo "  -> $target"
    cargo build --release --package "$FFI_CRATE" --target "$target"
done

echo "==> Creating fat library for simulator..."
mkdir -p "$OUTPUT_DIR"

SIM_LIB="$ROOT_DIR/target/aarch64-apple-ios-sim/release/${LIB_NAME}.a"
DEVICE_LIB="$ROOT_DIR/target/aarch64-apple-ios/release/${LIB_NAME}.a"

echo "==> Creating XCFramework..."

FRAMEWORK_DIR="$OUTPUT_DIR/FluxFFI.xcframework"
rm -rf "$FRAMEWORK_DIR"

HEADERS_DIR="$OUTPUT_DIR/Headers"
mkdir -p "$HEADERS_DIR"
cp "$HEADER_SRC" "$HEADERS_DIR/flux_ffi.h"

cat > "$HEADERS_DIR/module.modulemap" << 'MODULEMAP'
module FluxFFI {
    header "flux_ffi.h"
    export *
}
MODULEMAP

xcodebuild -create-xcframework \
    -library "$DEVICE_LIB" -headers "$HEADERS_DIR" \
    -library "$SIM_LIB" -headers "$HEADERS_DIR" \
    -output "$FRAMEWORK_DIR"

echo ""
echo "==> Done! XCFramework created at:"
echo "    $FRAMEWORK_DIR"
echo ""
echo "   Contents:"
ls -la "$FRAMEWORK_DIR"
