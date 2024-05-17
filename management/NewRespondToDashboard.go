package management

import (
	"embed"
	"html/template"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

//go:embed dashboard/views/*
var dashboardTemplatesFilesystem embed.FS

func NewRespondToDashboard(
	loadBalancer *loadbalancer.LoadBalancer,
	serverEventsChannel chan<- goroutine.ResultMessage,
) (*RespondToDashboard, error) {
	dashboardTemplates, err := template.ParseFS(dashboardTemplatesFilesystem, "dashboard/views/*.html")

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
