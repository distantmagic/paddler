Feature: Report llama.cpp metrics

    Background:
        Given balancer is running
        Given statsd is running
 
    # @serial
    # Scenario: There is no agent attached
    #     Then metrics report:
    #         | requests_buffered | 0 |
    #         | slots_idle        | 0 | 
    #         | slots_processing  | 0 |
            
    @serial
    Scenario: There are multiple agents attached
        Given llama.cpp server "llama-1" is running (has 1 slots)
        Given llama.cpp server "llama-2" is running (has 1 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        # Then metrics are stored
        # Then metrics report:
        #     | slots_idle        | 2 |
        #     | slots_processing  | 0 |
        #     | requests_buffered | 0 |
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-4 |
            | req-5 |
            | req-6 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-7 |
            | req-8 |
            | req-9 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-10 |
            | req-11 |
            | req-12 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-13 |
            | req-14 |
            | req-15 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-16 |
            | req-17 |
            | req-18 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-19 |
            | req-20 |
            | req-21 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-22 |
            | req-23 |
            | req-24 |
        Then metrics are stored
            | req-25 |
            | req-26 |
            | req-27 |
        Then metrics are stored
        When multiple requests are sent to "/chat/completions"
            | req-28 |
            | req-29 |
            | req-30 |
        Then metrics report:
            | slots_idle        | 0 | 
            | slots_processing  | 2 |
            | requests_buffered | 1 |
        # Then "req-1" response code is 200
        # Then "req-1" request landed in "llama-1"
        # Then "req-2" response code is 200
        # Then "req-2" request landed in "llama-2"
        # Then "req-3" response code is 200
        # Then "req-3" request landed in "llama-1"
