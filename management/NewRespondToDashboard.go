package management

import (
	"embed"
	"html/template"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

//go:embed resources/views/*
var dashboardTemplatesFilesystem embed.FS

func formatDate(t time.Time) string {
	return t.Format("2006-01-02 15:04:05 -0700 MST")
}

func NewRespondToDashboard(
	loadBalancer *loadbalancer.LoadBalancer,
	serverEventsChannel chan<- goroutine.ResultMessage,
) (*RespondToDashboard, error) {
	dashboardTemplates, err := template.
		New("base").
		Funcs(template.FuncMap{
			"formatDate": formatDate,
		}).
		ParseFS(dashboardTemplatesFilesystem, "resources/views/*.html")

	if err != nil {
		return nil, err
	}

	respondToDashboard := &RespondToDashboard{
		DashboardTemplates:  dashboardTemplates,
		LoadBalancer:        loadBalancer,
		ServerEventsChannel: serverEventsChannel,
	}

	return respondToDashboard, nil
}
