Feature: Report llama.cpp metrics

    Background:
        Given buffered requests timeout after 2 milliseconds
        Given balancer is running
        Given statsd is running
 
    @serial
    Scenario: There is no agent attached
        Then metrics report:
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
        Then metrics report:
            | slots_idle        | 2 |
            | slots_processing  | 0 |
            | requests_buffered | 0 |
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
        Then metrics report
            | req-4 |
            | req-5 |
            | req-6 |
        Then metrics report
            | req-7 |
            | req-8 |
            | req-9 |
        Then metrics report
            | req-10 |
            | req-11 |
            | req-12 |
        Then metrics report
            | req-13 |
            | req-14 |
            | req-15 |
        Then metrics report
            | req-16 |
            | req-17 |
            | req-18 |
        Then metrics report
            | req-19 |
            | req-20 |
            | req-21 |
        Then metrics report
            | req-22 |
            | req-23 |
            | req-24 |
        Then metrics report
            | req-25 |
            | req-26 |
            | req-27 |
        Then metrics report
            | req-28 |
            | req-29 |
            | req-30 |
        # Then avegrage metrics report was:
        #     | slots_idle        | 0 |
        #     | slots_processing  | 2 |
        #     | requests_buffered | 1 |
        # Then "req-1" response code is 200
        # Then "req-1" request landed in "llama-1"
        # Then "req-2" response code is 200
        # Then "req-2" request landed in "llama-2"
        # Then "req-3" response code is 200
        # Then "req-3" request landed in "llama-1"
        # Then "req-4" request landed in "llama-2"
        # Then "req-4" response code is 200
        # Then "req-5" request landed in "llama-1"
        # Then "req-5" request landed in "llama-2"
        # Then "req-6" response code is 200
        # Then "req-6" request landed in "llama-1"
        # Then "req-7" response code is 200
        # Then "req-7" request landed in "llama-1"
        # Then "req-8" response code is 200
        # Then "req-8" request landed in "llama-1"
        # Then "req-9" response code is 200
        # Then "req-9" request landed in "llama-1"
