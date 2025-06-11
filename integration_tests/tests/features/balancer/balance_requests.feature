Feature: Balance llama.cpp requests

    Background:
        Given balancer is running

    @serial
    Scenario: There are no agents attached
        When request "foo" is sent to "/chat/completions"
        Then "foo" response code is 502

    @serial
    Scenario: There is one agent attached
        Given llama.cpp server "llama-1" is running at :8071 (has 4 slots)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is healthy
        When request "foo" is sent to "/chat/completions"
        Then "foo" response code is 200

