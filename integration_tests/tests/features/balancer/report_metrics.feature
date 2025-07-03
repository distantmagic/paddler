Feature: Report llama.cpp metrics

    Background:
        Given balancer is running
        Given statsd is running
 
    @serial
    Scenario: There is no agent attached
        Then average metrics are:
            | requests_buffered | 0 |
            | slots_idle        | 0 | 
            | slots_processing  | 0 |
            
    @serial
    Scenario: There are multiple agents attached
        Given llama.cpp server "llama-1" is running (has 1 slots)
        Given llama.cpp server "llama-2" is running (has 1 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
            | req-4 |
            | req-5 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-6  |
            | req-7  |
            | req-8  |
            | req-9  |
            | req-10 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-11 |
            | req-12 |
            | req-13 |
            | req-14 |
            | req-15 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-16 |
            | req-17 |
            | req-18 |
            | req-19 |
            | req-20 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-21 |
            | req-22 |
            | req-23 |
            | req-24 |
            | req-25 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-26 |
            | req-27 |
            | req-28 |
            | req-29 |
            | req-30 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-31 |
            | req-32 |
            | req-33 |
            | req-34 |
            | req-35 |
        Then metrics are stored
        Then average metrics are:
            | slots_idle        | 1 | 
            | slots_processing  | 1 |
            | requests_buffered | 0 |
