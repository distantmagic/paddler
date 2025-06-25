Feature: Observe llama.cpp instances

    Background:
        Given balancer is running
        Given llama.cpp server "llama-1" is running (has 1 slot)
        Given llama.cpp server "llama-2" is running (has 1 slot)

    @serial
    Scenario: Agent attaches llama.cpp
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        Then balancer state is:
        |  agent  | error |
        | agent-1 |      false       |
        | agent-2 |      false       |
        When llama.cpp server "llama-1" stops running
        Then balancer state is:
        |  agent  | error |
        | agent-1 |      true        |
        | agent-2 |      false       |