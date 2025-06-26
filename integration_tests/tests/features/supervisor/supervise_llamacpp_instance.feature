Feature: Supervise llama.cpp instance

    @serial
    Scenario: supervisor starts and stops a llama.cpp instance
        Given balancer is running
        Given supervisor "supervisor-1" is running
        Given agent "agent-1" is running (observes llama-server managed by "supervisor-1")
        Given agent "agent-1" is registered
        Then balancer state is:
            | agent   | is_connect_error |
            | agent-1 | false            |
        When supervisor "supervisor-1" stops llama-server
        Then balancer state is:
            | agent   | is_connect_error |
            | agent-1 | true             |
        When supervisor "supervisor-1" starts llama-server
        Then balancer state is:
            | agent   | is_connect_error |
            | agent-1 | false            |
        When supervisor "supervisor-1" stops
        Then balancer state is:
            | agent   | is_connect_error |
            | agent-1 | true             |
