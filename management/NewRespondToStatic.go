package management

import (
	"embed"
	"net/http"
)

//go:embed static/*
var staticFilesystem embed.FS

func NewRespondToStatic() http.Handler {
	return http.FileServer(http.FS(staticFilesystem))
}
