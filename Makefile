.DEFAULT_GOAL := paddler

CSS_SOURCES := \
	$(wildcard */*.css) \
	$(wildcard */*/*.css) \
	$(wildcard */*/*/*.css)

GO_SOURCES := \
	$(wildcard *.go) \
	$(wildcard */*.go)

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

paddler: $(CSS_SOURCES) $(GO_SOURCES)
	$(MAKE) -C management build
	go build -o paddler

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: clean
clean:
	$(MAKE) -C management clean
	rm -f log.db
	rm -f paddler
	rm -rf snapshots
	rm -f stable.db

.PHONY: fmt
fmt:
	go fmt ./...

.PHONY: lint
lint:
	golangci-lint run
