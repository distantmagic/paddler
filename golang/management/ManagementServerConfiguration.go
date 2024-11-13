package management

import "github.com/distantmagic/paddler/netcfg"

type ManagementServerConfiguration struct {
	EnableDashboard bool
	HttpAddress     *netcfg.HttpAddressConfiguration
}
