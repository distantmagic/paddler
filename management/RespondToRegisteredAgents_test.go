package management

import (
    "encoding/json"
    "testing"
    "time"

    "github.com/distantmagic/paddler/llamacpp"
    "github.com/distantmagic/paddler/loadbalancer"
    "github.com/distantmagic/paddler/netcfg"
    "github.com/stretchr/testify/assert"
)

func TestRespondToRegisteredAgents(t *testing.T) {
    var targets []*loadbalancer.LlamaCppTarget

    timeLocation, _ := time.LoadLocation("UTC")

    httpAddress := &netcfg.HttpAddressConfiguration{
        Host:   "127.0.0.1",
        Port:   8088,
        Scheme: "http",
    }

    targets = append(targets, &loadbalancer.LlamaCppTarget{
        LlamaCppSlotsAggregatedStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
            SlotsIdle:       4,
            SlotsProcessing: 0,
        },
        LlamaCppTargetConfiguration: &loadbalancer.LlamaCppTargetConfiguration{
			Id: "01921f1d-e817-72c9-bb92-f6363899042e",
            Name: "Agent-1",
            LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
                HttpAddress: httpAddress,
            },
        },
        LastUpdate:                 time.Date(2007, 9, 24, 4, 20, 20, 2000000000, timeLocation),
        TotalUpdates:               3000,
        RemainingTicksUntilRemoved: 2,
    })

	loadBalancerTargetCollection := &loadbalancer.LoadBalancerTargetCollection{
		Targets: targets,
	}

    llamaCppLastRegisteredAgentJson, _ := json.Marshal(loadBalancerTargetCollection)

    assert.Equal(
        t,
        `{"targets":[{"last_update":"2007-09-24T04:20:22Z","llamacpp_slots_aggregated_status":{"status":"","slots_idle":4,"slots_processing":0},"llamacpp_target_configuration":{"Id":"01921f1d-e817-72c9-bb92-f6363899042e","Name":"Agent-1","llama_cpp_configuration":{"http_address":{"host":"127.0.0.1","port":8088,"scheme":"http"}}},"remaining_ticks_until_removed":2,"total_updates":3000}]}`,
        string(llamaCppLastRegisteredAgentJson),
    )
}