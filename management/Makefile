CSS_ENTRYPOINTS := $(wildcard resources/css/mgmt-*.css)
CSS_SOURCES := $(wildcard resources/css/*.css)

ESBUILD_TARGET_DIRECTORY ?= static

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

node_modules: yarn.lock
	yarnpkg install --check-files --frozen-lockfile --non-interactive;
	touch node_modules;

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: build
build: esbuild

.PHONY: clean
clean:
	rm -rf node_modules

.PHONY: esbuild
esbuild: $(CSS_SOURCES) node_modules
	./node_modules/.bin/esbuild \
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
	;
