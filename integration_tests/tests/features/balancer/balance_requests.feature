Feature: Balance llama.cpp requests

    Background:
        Given balancer is running

    @serial
    Scenario: There are no agents attached
        When request "foo" is sent to "/chat/completions"
        Then "foo" response code is 504

    @serial
    Scenario: There is one agent attached
        Given llama.cpp server "llama-1" is running (has 3 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Then dashboard report:
            |  agent  | slots_idle | slots_processing |  error |
            | agent-1 |     3      |        0         |  None  |
        When request "foo" is sent to "/chat/completions"
        Then "foo" response code is 200
        Then "foo" request landed in "llama-1"

    @serial
    Scenario: There are multiple agents attached
        Given llama.cpp server "llama-1" is running (has 4 slots)
        Given llama.cpp server "llama-2" is running (has 4 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
        Then "req-1" response code is 200
        Then "req-1" request landed in "llama-1"
        Then "req-2" response code is 200
        Then "req-2" request landed in "llama-2"

    @serial
    Scenario: More requests than slots available
        Given llama.cpp server "llama-1" is running (has 1 slot)
        Given llama.cpp server "llama-2" is running (has 1 slot)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
        Then "req-1" response code is 200
        Then "req-1" request landed in "llama-1"
        Then "req-2" response code is 200
        Then "req-2" request landed in "llama-2"
        Then "req-3" response code is 504
