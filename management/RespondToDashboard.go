package management

import (
	"container/list"
	"html/template"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RespondToDashboard struct {
	DashboardTemplates  *template.Template
	LoadBalancer        *loadbalancer.LoadBalancer
	ServerEventsChannel chan<- goroutine.ResultMessage
}

func iterList[T any](loadBalancerCollection *list.List) <-chan T {
	ch := make(chan T)

	go func() {
		defer close(ch)

		for element := loadBalancerCollection.Front(); element != nil; element = element.Next() {
			ch <- element.Value.(T)
		}
	}()

	return ch
}

func (self *RespondToDashboard) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	mutexToken := self.LoadBalancer.LoadBalancerTargetCollection.TargetsRBMutex.RLock()
	defer self.LoadBalancer.LoadBalancerTargetCollection.TargetsRBMutex.RUnlock(mutexToken)

	response.Header().Set("Content-Type", "text/html")
	response.WriteHeader(http.StatusOK)

	err := self.DashboardTemplates.ExecuteTemplate(response, "index.html", &RespondToDashboardTemplateProps{
		LlamaCppTargets:    iterList[*loadbalancer.LlamaCppTarget](self.LoadBalancer.LoadBalancerTargetCollection.Targets),
		LoadBalancerStatus: self.LoadBalancer.GetStatus(),
	})

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Error: err,
		}

		return
	}
}
