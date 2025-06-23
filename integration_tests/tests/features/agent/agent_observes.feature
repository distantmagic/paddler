Feature: Observe llama.cpp instances

    Background:
        Given balancer is running (2 max requests)
        Given llama.cpp server "llama-1" is running (has 3 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered

    @serial
    Scenario: Agent attaches llama.cpp
        Then dashboard report:
        |  agent  | error |
        | agent-1 | None |
        When llama.cpp server "llama-1" stops running
        Then dashboard report:
        |  agent  | error |
        | agent-1 | Request to llamacpp Failed. Is it running? error sending request |