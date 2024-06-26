package loadbalancer

import (
	"net/http"
	"time"
)

type BufferedRequest struct {
	Done     chan bool
	Response http.ResponseWriter
	Request  *http.Request
	Timeout  time.Time
}
