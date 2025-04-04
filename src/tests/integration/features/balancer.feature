Feature: Balancer

    Scenario: Balancer can loadbalance
      Given balancer-1 is running at 0.0.0.0:8070
      Given llamacpp-1 is running at 0.0.0.0:8080 with 4 slots
      Given agent-1 is running and observing llamacpp-1, and registered at balancer-1
      Given llamacpp-2 is running at 0.0.0.0:8081 with 3 slots
      Given agent-2 is running and observing llamacpp-2, and registered at balancer-1

      When first request is proxied to balancer
      Then balancer must tell 1 slot is busy
      When second request is proxied to balancer
      Then balancer must tell 2 slots are busy
      Then balancer should return a successful response

      # When first request is proxied to balancer
      # Then balancer must tell 1 slot is busy
      # When second request is proxied to balancer
      # Then balancer must tell 2 slots are busy
      # Then balancer should return a successful response
      