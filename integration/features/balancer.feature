Feature: Balancer

    Background:
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second
      Given llamacpp-1 is running at 8080 with 4 slots
      Given llamacpp-2 is running at 8081 with 3 slots
      
      When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080
      When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 3 slots at 0.0.0.0:8081

      @serial
      Scenario: Balancer can loadbalance
      
        When 1 request is proxied to balancer-1 in 127.0.0.1:8071
        Then balancer-1 must tell 1 slot is busy and 6 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2
        Then balancer-1 must return a successful response in 127.0.0.1:8071

      # @serial
      # Scenario: Balancer cannot loadbalance
      #   When 7 requests are proxied to balancer-1 in 127.0.0.1:8071 in balancer feature
      #   Then balancer-1 must tell 7 slots are busy and 0 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2 in balancer feature

      #   When 1 request is proxied to balancer-1 in 127.0.0.1:8071 in balancer feature
      #   Then balancer-1 must return an unsuccessful response in 127.0.0.1:8071 in balancer feature
      # @serial
  #     Scenario: Balancer reports metrics
  #       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in balancer feature
  #       Given statsd-1 is running at 0.0.0.0:9125, 0.0.0.0:9102 and receives metrics from balancer-1 in balancer feature
  #       Given prometheus-1 is running at 0.0.0.0:9090 and scrapes metrics from 0.0.0.0:9102 every 1 second in balancer feature
  #       Given llamacpp-1 is running at 8080 with 4 slots in balancer feature
  #       Given agent-1 is running and observing llamacpp-1 in 127.0.0.1:8080, and registered at balancer-1 in 127.0.0.1:8070 in balancer feature
  #       Given llamacpp-2 is running at 8081 with 3 slots in balancer feature
  #       Given agent-2 is running and observing llamacpp-2 in 127.0.0.1:8081, and registered at balancer-1 in 127.0.0.1:8070 in balancer feature

  #       When 1 requests are proxied to balancer-1 in 127.0.0.1:8071 in balancer feature
  #       Then prometheus-1 must tell 1 slot is processing at 0.0.0.0:9090 from 0.0.0.0:9102 in balancer feature
  #       Then prometheus-1 must tell 6 slots are idle at 0.0.0.0:9090 from 0.0.0.0:9102 in balancer feature
      