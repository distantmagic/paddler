Feature: Supervisor

    Background:
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second
      Given llamacpp-1 is running at 8080 with 4 slots
      When agent-1 is running and observing llamacpp-1 in 127.0.0.1:8080, and registered at balancer-1 in 127.0.0.1:8070
      Given llamacpp-2 is running at 8081 with 3 slots
      When agent-2 is running and observing llamacpp-2 in 127.0.0.1:8081, and registered at balancer-1 in 127.0.0.1:8070

      @serial
      Scenario: Supervisor can restart llamacpp after being killed
        When llamacpp-2 from supervisor-2 is killed
        When llamacpp-1 from supervisor-1 is killed
        When 2 requests are proxied to balancer-1 in 127.0.0.1:8071
        Then balancer-1 must tell 2 slots are busy and 6 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2
        Then balancer-1 must return a successful response in 127.0.0.1:8071

      # @serial
      # Scenario: Supervisor can start llamacpp and change its arguments
      #   When 1 request is proxied to supervisor-1 in 0.0.0.0:8087 to change slots to 2 and port to 8080 in supervisor feature
      #   Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 2 slots at 0.0.0.0:8080 in supervisor feature

      #   When 1 request is proxied to supervisor-2 in 0.0.0.0:8088 to change slots to 4 and port to 8082 in supervisor feature
      #   Then "balancer-1" in "0.0.0.0:8070" must report that "agent-2" cannot fetch "llamacpp-2" in "0.0.0.0:8081" in supervisor feature
  