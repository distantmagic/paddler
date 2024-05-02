.DEFAULT_GOAL := paddler

GO_SOURCES := \
	$(wildcard *.go) \
	$(wildcard */*.go)

paddler: $(GO_SOURCES)
	go build -o paddler

.PHONY: clean
clean:
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
