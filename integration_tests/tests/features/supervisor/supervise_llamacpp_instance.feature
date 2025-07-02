Feature: Supervise llama.cpp instance

    @serial
    Scenario: supervisor starts and stops a llama.cpp instance
        Given balancer is running
        Given supervisor "supervisor-1" is running
        Given supervisor "supervisor-1" is registered
        # Given agent "agent-1" is running (observes llama-server managed by "supervisor-1")
        # Given agent "agent-1" is registered
