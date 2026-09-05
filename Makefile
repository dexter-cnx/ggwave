SHELL := /bin/bash
.DEFAULT_GOAL := help

ROOT := $(CURDIR)
FLUTTER_PKG := $(ROOT)/packages/ggwave_flutter
FLUTTER_EXAMPLE := $(FLUTTER_PKG)/example
DART_PKG := $(ROOT)/packages/ggwave_dart
FRB_VERSION := 2.8.0
TARGET ?= lib/main.dart
DEVICE ?=

.PHONY: help doctor deps frb-install frb-generate bootstrap prepare-android clean clean-generated analyze test preflight run-android build-android apk-check

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "; printf "Usage: make <target> [DEVICE=<id>] [TARGET=<dart entrypoint>]\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

doctor: ## Show toolchain versions and verify required commands
	@command -v flutter >/dev/null || { echo "flutter not found"; exit 2; }
	@command -v dart >/dev/null || { echo "dart not found"; exit 2; }
	@command -v cargo >/dev/null || { echo "cargo not found"; exit 2; }
	@command -v rustup >/dev/null || { echo "rustup not found"; exit 2; }
	@command -v flutter_rust_bridge_codegen >/dev/null || { echo "flutter_rust_bridge_codegen not found; run: make frb-install"; exit 2; }
	@echo "Flutter: $$(flutter --version | head -n 1)"
	@echo "Dart: $$(dart --version 2>&1)"
	@echo "Rust: $$(rustc --version)"
	@echo "FRB: $$(flutter_rust_bridge_codegen --version 2>&1 || true)"

deps: ## Resolve Dart/Flutter dependencies for all packages
	cd $(DART_PKG) && dart pub get
	cd $(FLUTTER_PKG) && flutter pub get
	cd $(FLUTTER_EXAMPLE) && flutter pub get

frb-install: ## Install the pinned flutter_rust_bridge_codegen CLI
	cargo install flutter_rust_bridge_codegen --version $(FRB_VERSION) --locked

frb-generate: ## Generate FRB Dart/Rust bindings only
	cd $(FLUTTER_PKG) && flutter_rust_bridge_codegen generate

bootstrap: doctor deps ## Generate FRB bindings and native Flutter scaffolds reproducibly
	bash $(ROOT)/tool/bootstrap_flutter_native.sh

prepare-android: bootstrap ## Prepare the local Android validation runner
	bash $(ROOT)/tool/prepare_flutter_android_validation.sh

clean: ## Clean Flutter build outputs without deleting generated FRB bindings
	cd $(FLUTTER_PKG) && flutter clean
	cd $(FLUTTER_EXAMPLE) && flutter clean

clean-generated: ## Remove generated native scaffolds; bootstrap recreates them
	rm -rf $(FLUTTER_PKG)/android \
		$(FLUTTER_PKG)/ios \
		$(FLUTTER_PKG)/macos \
		$(FLUTTER_PKG)/linux \
		$(FLUTTER_PKG)/windows \
		$(FLUTTER_PKG)/cargokit \
		$(FLUTTER_PKG)/rust_builder \
		$(FLUTTER_EXAMPLE)/android

analyze: ## Run Dart and Flutter analyzers
	cd $(DART_PKG) && dart analyze
	cd $(FLUTTER_PKG) && flutter analyze
	cd $(FLUTTER_EXAMPLE) && flutter analyze

test: ## Run Dart and Flutter tests
	cd $(DART_PKG) && dart test
	cd $(FLUTTER_PKG) && flutter test
	cd $(FLUTTER_EXAMPLE) && flutter test

preflight: bootstrap analyze test ## Reproduce the local/CI generation + analysis + test gate
	@echo "Preflight passed"

run-android: prepare-android ## Bootstrap then run Android example; DEVICE=<adb id> is required
	@if [ -z "$(DEVICE)" ]; then echo "DEVICE is required, e.g. make run-android DEVICE=RF8Y909V0LV"; exit 2; fi
	cd $(FLUTTER_EXAMPLE) && flutter run -d $(DEVICE) -t $(TARGET)

build-android: prepare-android ## Bootstrap then build Android debug APK
	cd $(FLUTTER_EXAMPLE) && flutter build apk --debug -t $(TARGET)

apk-check: build-android ## Verify native ggwave and libc++ shared libraries are packaged
	@APK="$(FLUTTER_EXAMPLE)/build/app/outputs/flutter-apk/app-debug.apk"; \
	unzip -l "$$APK" | grep -q 'libggwave_flutter_native.so' || { echo "Missing libggwave_flutter_native.so"; exit 1; }; \
	unzip -l "$$APK" | grep -q 'libc++_shared.so' || { echo "Missing libc++_shared.so"; exit 1; }; \
	echo "APK native library check passed: $$APK"
