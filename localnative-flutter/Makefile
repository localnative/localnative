.PHONY: help setup bridge build-rust clean run-android run-ios run-macos

help:
	@echo "Local Native Flutter - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  setup        - Install Rust targets and tools"
	@echo "  bridge       - Generate flutter_rust_bridge bindings"
	@echo "  build-rust   - Build Rust library for all platforms"
	@echo "  clean        - Clean build artifacts"
	@echo "  run-android  - Run on Android device/emulator"
	@echo "  run-ios      - Run on iOS simulator"
	@echo "  run-macos    - Run on macOS"

setup:
	@echo "Setting up development environment..."
	chmod +x scripts/*.sh
	./scripts/setup-rust-targets.sh
	cargo install flutter_rust_bridge_codegen
	flutter pub get

bridge:
	@echo "Generating bridge code..."
	chmod +x scripts/generate-bridge.sh
	./scripts/generate-bridge.sh

build-rust:
	@echo "Building Rust library..."
	chmod +x scripts/build-rust.sh
	./scripts/build-rust.sh

clean:
	@echo "Cleaning build artifacts..."
	flutter clean
	cd rust && cargo clean
	rm -rf android/app/src/main/jniLibs
	rm -rf ios/Runner/Frameworks
	rm -f lib/bridge_generated.dart
	rm -f ios/Runner/bridge_generated.h

run-android: bridge build-rust
	@echo "Running on Android..."
	flutter run -d android

run-ios: bridge build-rust
	@echo "Running on iOS..."
	flutter run -d ios

run-macos: bridge build-rust
	@echo "Running on macOS..."
	flutter run -d macos

# Development workflow
dev: bridge
	@echo "Starting development mode..."
	flutter run
