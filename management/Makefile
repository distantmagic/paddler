CSS_ENTRYPOINTS := $(wildcard resources/css/mgmt-*.css)
CSS_SOURCES := $(wildcard resources/css/*.css)
TS_ENTRYPOINTS := \
	$(wildcard resources/ts/controller_*.ts) \
	$(wildcard resources/ts/global_*.ts)
TS_SOURCES := $(wildcard resources/ts/*.ts)

ESBUILD_TARGET_DIRECTORY ?= static

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

.pnp.cjs: yarn.lock
	yarnpkg install --immutable
	touch .pnp.cjs;

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: build
build: esbuild

.PHONY: clean
clean:
	rm -rf esbuild-meta-mgmt.json
	rm -rf static
	rm -rf .pnp.cjs

.PHONY: esbuild
esbuild: $(CSS_SOURCES) .pnp.cjs
	yarnpkg run esbuild \
		--bundle \
		--asset-names="./[name]" \
		--entry-names="./[name]" \
		--format=esm \
		--loader:.jpg=file \
		--loader:.otf=file \
		--loader:.svg=file \
		--loader:.ttf=file \
		--loader:.webp=file \
		--metafile=esbuild-meta-mgmt.json \
		--minify \
		--outdir=$(ESBUILD_TARGET_DIRECTORY) \
		--sourcemap \
		--splitting \
		--target=safari16 \
		--tree-shaking=true \
		$(CSS_ENTRYPOINTS) \
		$(TS_ENTRYPOINTS) \
	;
