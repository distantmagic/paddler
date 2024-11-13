package loadbalancer

import "github.com/distantmagic/paddler/netcfg"

type StatsdConfiguration struct {
	EnableStatsdReporter bool
	HttpAddress          *netcfg.HttpAddressConfiguration
}
