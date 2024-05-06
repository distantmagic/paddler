package metahttp

import (
	"fmt"
	"html"
	"net/http"
)

type RespondToHealth struct {
}

func (self *RespondToHealth) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	fmt.Fprintf(response, "Hello, %q", html.EscapeString(request.URL.Path))
}
