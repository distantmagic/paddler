package llamacpp

import (
	"github.com/distantmagic/paddler/netcfg"
)

type LlamaCppConfiguration struct {
	HttpAddress *netcfg.HttpAddressConfiguration
}

func (self *LlamaCppConfiguration) String() string {
	return self.HttpAddress.String()
}
