.DEFAULT_GOAL := build

RUST_LOG ?= debug

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

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

.PHONY: run.agent
run.agent: esbuild
	cargo run -- agent \
		--external-llamacp-addr "127.0.0.1:8081" \
		--local-llamacpp-addr="localhost:8081" \
		--local-llamacpp-api-key "test" \
		--management-addr="localhost:8095" \
		--name "wohoo"

.PHONY: run.balancer
run.balancer: esbuild
	cargo run --features web_dashboard \
		-- balancer \
		--management-addr="127.0.0.1:8095"  \
		--management-dashboard-enable \
		--reverseproxy-addr="127.0.0.1:8096"
