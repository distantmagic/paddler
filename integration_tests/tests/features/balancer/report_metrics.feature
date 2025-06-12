Feature: Report llama.cpp metrics

    Background:
        Given balancer is running
        Given statsd is running
        Given prometheus is running (scrapes every 1 second)

    @serial
    Scenario: There is no agent attached
        Given llama.cpp server "llama-1" is running (has 4 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is healthy

        # When 1 requests are proxied to balancer-1 in 127.0.0.1:8071
        # Then prometheus-1 must tell 1 slot is processing at 0.0.0.0:9090 from 0.0.0.0:9102
        # Then prometheus-1 must tell 6 slots are idle at 0.0.0.0:9090 from 0.0.0.0:9102