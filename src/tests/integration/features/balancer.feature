Feature: Balancer

    # Scenario: Balancer can loadbalance
    #   Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
    #   Given statsd-1 is running at 0.0.0.0 in 8072, 8073 and receives metrics from balancer-1
    #   Given llamacpp-1 is running at 8080 with 4 slots
    #   Given agent-1 is running and observing llamacpp-1 in 127.0.0.1:8080, and registered at balancer-1 in 127.0.0.1:8070
    #   Given llamacpp-2 is running at 8081 with 3 slots
    #   Given agent-2 is running and observing llamacpp-2 in 127.0.0.1:8081, and registered at balancer-1 in 127.0.0.1:8070

    #   When first request is proxied to balancer-1 in 127.0.0.1:8071
    #   Then balancer-1 must tell 1 slot is busy and 6 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2

    #   When second request is proxied to balancer-1 in 127.0.0.1:8071
    #   Then balancer-1 must tell 2 slots are busy and 5 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2
    #   Then balancer-1 should return a successful response in 127.0.0.1:8071

    # Scenario: Balancer cannot loadbalance
    #   Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
    #   Given statsd-1 is running at 0.0.0.0 in 8072, 8073 and receives metrics from balancer-1
    #   Given llamacpp-1 is running at 8080 with 4 slots
    #   Given agent-1 is running and observing llamacpp-1 in 127.0.0.1:8080, and registered at balancer-1 in 127.0.0.1:8070
    #   Given llamacpp-2 is running at 8081 with 3 slots
    #   Given agent-2 is running and observing llamacpp-2 in 127.0.0.1:8081, and registered at balancer-1 in 127.0.0.1:8070

    #   When 7 requests are proxied to balancer-1 in 127.0.0.1:8071
    #   Then balancer-1 must tell 7 slots are busy and 0 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2

    #   When 1 request is proxied to balancer-1 in 127.0.0.1:8071
    #   Then balancer-1 should return an unsuccessful response in 127.0.0.1:8071

    Scenario: Balancer report metrics
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
      Given statsd-1 is running at 0.0.0.0 in 8072, 8073 and receives metrics from balancer-1
      Given llamacpp-1 is running at 8080 with 4 slots
      Given agent-1 is running and observing llamacpp-1 in 127.0.0.1:8080, and registered at balancer-1 in 127.0.0.1:8070
      Given llamacpp-2 is running at 8081 with 3 slots
      Given agent-2 is running and observing llamacpp-2 in 127.0.0.1:8081, and registered at balancer-1 in 127.0.0.1:8070

      When 5 requests are proxied to balancer-1 in 127.0.0.1:8071
      Then statsd-1 must tell 5 slots are busy and 2 slots are idle at 0.0.0.0 in 8073
      