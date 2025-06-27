Feature: Balancer supports CORS headers

    @serial
    Scenario: CORS headers are present in the response
        Given balancer allows CORS host "http://example.com"
        Given balancer is running
        Given llama.cpp server "llama-1" is running (has 1 slot)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given request "foo" has header "Origin" with value "http://example.com"
        When request "foo" is sent to management endpoint "/api/v1/agents"
        Then "foo" response code is 200
        And "foo" response header "Access-Control-Allow-Origin" is "http://example.com"

    @serial
    Scenario: CORS headers are missing when not allowed
        Given balancer is running
        Given llama.cpp server "llama-1" is running (has 1 slot)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given request "foo" has header "Origin" with value "http://example_2.com"
        When request "foo" is sent to management endpoint "/api/v1/agents"
        Then "foo" response code is 200
        And "foo" response header "Access-Control-Allow-Origin" is not present
