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
build: node_modules
	./build.sh

.PHONY: clean
clean:
	rm -rf esbuild-meta-mgmt.json
	rm -rf node_modules
	rm -rf static
