#!/bin/bash
set -e

echo "Generating flutter_rust_bridge bindings..."

# Generate Dart and C bindings
flutter_rust_bridge_codegen

echo "Bridge code generated successfully!"
echo "Generated files:"
echo "  - lib/bridge_generated.dart"
echo "  - ios/Runner/bridge_generated.h"
