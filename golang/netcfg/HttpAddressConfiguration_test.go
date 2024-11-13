package netcfg

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestUrlIsGenerated(t *testing.T) {
	config := &HttpAddressConfiguration{
		Host:   "localhost",
		Port:   8081,
		Scheme: "http",
	}

	assert.Equal(
		t,
		"http://localhost:8081/hi",
		config.BuildUrlWithPath("hi").String(),
	)
}
