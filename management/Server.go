package management

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	ManagementServerConfiguration *ManagementServerConfiguration
	Logger                        hclog.Logger
	RespondToDashboard            http.Handler
	RespondToHealth               http.Handler
	RespondToRegisterTarget       http.Handler
	RespondToStatic               http.Handler
}

func (self *Server) Serve(serverEventsChannel chan<- goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.ManagementServerConfiguration.HttpAddress.GetHostWithPort(),
	)

	mux := http.NewServeMux()

	self.Logger.Debug("dashboard", "enabled", self.ManagementServerConfiguration.EnableDashboard)

	if self.ManagementServerConfiguration.EnableDashboard {
		mux.Handle("/dashboard", self.RespondToDashboard)
		mux.Handle("/static/", self.RespondToStatic)
	}

	mux.Handle("/health", self.RespondToHealth)
	mux.Handle("/register/target", self.RespondToRegisterTarget)

	err := http.ListenAndServe(
		self.ManagementServerConfiguration.HttpAddress.GetHostWithPort(),
		mux,
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to listen",
			Error:   err,
		}
	}
}
