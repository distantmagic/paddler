package reverseproxy

import "github.com/distantmagic/paddler/netcfg"

type ReverseProxyConfiguration struct {
	HttpAddress *netcfg.HttpAddressConfiguration
}
