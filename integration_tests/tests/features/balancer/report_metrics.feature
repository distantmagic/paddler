Feature: Report llama.cpp metrics

    Background:
        Given balancer is running (2 max requests)
        Given statsd is running
 
    @serial
    Scenario: There is no agent attached
        Then metrics report:
            | slots_idle | 0 | 
            | slots_processing | 0 |

    @serial
    Scenario: There are multiple agents attached
        Given llama.cpp server "llama-1" is running (has 4 slots)
        Given llama.cpp server "llama-2" is running (has 4 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        Then metrics report:
            | slots_idle | 8 |
            | slots_processing | 0 |
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
        Then metrics report:
            | slots_idle | 6 | 
            | slots_processing | 2 |
        Then "req-1" response code is 200
        Then "req-1" request landed in "llama-1"
        Then "req-2" response code is 200
        Then "req-2" request landed in "llama-2"