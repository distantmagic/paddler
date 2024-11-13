package llamacpp

import (
	"github.com/distantmagic/paddler/netcfg"
)

type LlamaCppConfiguration struct {
	HttpAddress *netcfg.HttpAddressConfiguration `json:"http_address"`
	ApiKey      string
}

func (self *LlamaCppConfiguration) String() string {
	return self.HttpAddress.String()
}
