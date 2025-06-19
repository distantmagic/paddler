.DEFAULT_GOAL := build

RUST_LOG ?= debug

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

package-lock.json: package.json
	npm install --package-lock-only

node_modules: package-lock.json
	npm install --from-lockfile
	touch node_modules

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: build
build:
	./jarmuz-release.mjs
	cargo build --features web_dashboard --release

.PHONY: clean
clean:
	rm -rf esbuild-meta.json
	rm -rf node_modules
	rm -rf target

.PHONY: integration_tests
integration_tests:
	cargo build
	$(MAKE) -C integration_tests test

.PHONY: test
test: integration_tests
	cargo test
