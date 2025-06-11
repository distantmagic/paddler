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
build: esbuild
	cargo build --features web_dashboard --release

.PHONY: clean
clean:
	rm -rf esbuild-meta.json
	rm -rf node_modules
	rm -rf target

.PHONY: esbuild
esbuild: node_modules
	npm exec esbuild -- \
		--bundle \
		--asset-names="./[name]" \
		--entry-names="./[name]" \
		--format=esm \
		--loader:.jpg=file \
		--loader:.otf=file \
		--loader:.svg=file \
		--loader:.ttf=file \
		--loader:.webp=file \
		--metafile=esbuild-meta.json \
		--minify \
		--outdir=static \
		--sourcemap \
		--splitting \
		--target=safari16 \
		--tree-shaking=true \
		resources/css/reset.css \
		resources/css/page-dashboard.css \
		resources/ts/controller_dashboard.tsx \

.PHONY: integration_tests
integration_tests:
	cargo build
	$(MAKE) -C integration_tests test

.PHONY: test
test: integration_tests
	cargo test
