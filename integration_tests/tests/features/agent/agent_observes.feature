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

    @serial
    Scenario: Agent detaches llama.cpp
        When llama.cpp server "llama-1" stops running
        Then dashboard report:
        |  agent  | error |
        | agent-1 | Request to http://127.0.0.1:8000/slots Failed. Is it running? error sending request |