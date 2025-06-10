.DEFAULT_GOAL := build

RUST_LOG ?= debug

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: test
test:
	$(MAKE) -C integration test
	$(MAKE) -C paddler test

.PHONY: build
build:
	$(MAKE) -C integration build
	$(MAKE) -C paddler build

esbuild:
	$(MAKE) -C paddler esbuild

.PHONY: clean
clean:
	rm -rf esbuild-meta.json
	rm -rf node_modules
	rm -rf target

.PHONY: run.agent
run.agent: esbuild
	cargo run --bin paddler agent \
		--external-llamacpp-addr "127.0.0.1:8081" \
		--local-llamacpp-addr="localhost:8081" \
		--management-addr="localhost:8095" \
		--name "wohoo"

.PHONY: run.balancer
run.balancer: esbuild
	cargo run --bin paddler --features web_dashboard \
		-- balancer \
		--management-addr="127.0.0.1:8095" \
		--management-dashboard-enable \
		--reverseproxy-addr="127.0.0.1:8096"
