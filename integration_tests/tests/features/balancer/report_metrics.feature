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
        Then metrics report:
            | slots_idle        | 2 |
            | slots_processing  | 0 |
            | requests_buffered | 0 |
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
            | req-4 |
            | req-5 |
            | req-6 |
            | req-7 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-8  |
            | req-9  |
            | req-10 |
            | req-11 |
            | req-12 |
            | req-13 |
            | req-14 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-15 |
            | req-16 |
            | req-17 |
            | req-18 |
            | req-19 |
            | req-20 |
            | req-21 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-22 |
            | req-23 |
            | req-24 |
            | req-25 |
            | req-26 |
            | req-27 |
            | req-28 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-29 |
            | req-30 |
            | req-31 |
            | req-32 |
            | req-33 |
            | req-34 |
            | req-35 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-36 |
            | req-37 |
            | req-38 |
            | req-39 |
            | req-41 |
            | req-42 |
            | req-43 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-44 |
            | req-45 |
            | req-46 |
            | req-47 |
            | req-48 |
            | req-49 |
            | req-50 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-51 |
            | req-52 |
            | req-53 |
            | req-54 |
            | req-55 |
            | req-56 |
            | req-57 |
        Then metrics are stored
            | req-58 |
            | req-59 |
            | req-60 |
            | req-61 |
            | req-62 |
            | req-63 |
            | req-64 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-65 |
            | req-66 |
            | req-67 |
            | req-68 |
            | req-69 |
            | req-70 |
        Then average metrics are:
            | slots_idle        | 1 | 
            | slots_processing  | 2 |
            | requests_buffered | 2 |
