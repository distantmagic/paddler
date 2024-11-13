.DEFAULT_GOAL := paddler-bin-linux-x64

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

godepgraph.png: $(GO_SOURCES)
	godepgraph \
		-p github.com/aws,github.com/hashicorp,github.com/smira,github.com/urfave \
		-s \
		-novendor \
		github.com/distantmagic/paddler | dot -Kfdp -Tpng -o godepgraph.png

paddler-bin-linux-x64: $(CSS_SOURCES) $(GO_SOURCES)
	$(MAKE) -C management build
	go build -o paddler-bin-linux-x64

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: clean
clean:
	$(MAKE) -C management clean
	rm -f log.db
	rm -f paddler-bin-linux-x64
	rm -rf snapshots
	rm -f stable.db

.PHONY: deps
deps: godepgraph.png
	open godepgraph.png

.PHONY: fmt
fmt:
	go fmt ./...
	tofu fmt -recursive infra

.PHONY: lint
lint:
	golangci-lint run
