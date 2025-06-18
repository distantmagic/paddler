Feature: Observe llama.cpp instances

    Background:
        Given balancer is running
        Given llama.cpp server "llama-1" is running (has 3 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered

    @serial
    Scenario: Agent detaches llama.cpp
        When llama.cpp server "llama-1" stops running
        Then dashboard report:
        |  agent  | slots_idle | slots_processing |  error |
        | agent-1 |     3      |        0         |  None  |

