package llamacpp

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestUrlIsGenerated(t *testing.T) {
	llamaCppConfiguration := &LlamaCppConfiguration{
		Host:   "localhost",
		Port:   8081,
		Scheme: "http",
	}

	assert.Equal(
		t,
		"http://localhost:8081/hi",
		llamaCppConfiguration.BuildUrlWithPath("hi").String(),
	)
}
