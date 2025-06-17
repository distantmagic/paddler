Feature: Observe llama.cpp instances

    Background:
        Given balancer is running

    @serial
    Scenario: There are multiple agents attached
        Given llama.cpp server "llama-1" is running (has 4 slots)
        Given llama.cpp server "llama-2" is running (has 4 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is healthy
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is healthy
        Then dashboard report:
            | agent-1 | slots_idle | 4 | slots_processing | 0 | error | none |
            | agent-2 | slots_idle | 4 | slots_processing | 0 | error | none |
