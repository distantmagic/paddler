Feature: Agent

    Scenario: Agent can register
      Given llamacpp-1 is running at 0.0.0.0:8080 with 4 slots
      Given llamacpp-2 is running at 0.0.0.0:8081 with 3 slots
      Given balancer-1 is running at 0.0.0.0:8070

      When agent-1 is running and observing llamacpp-1, and registered at balancer-1
      Then balancer-1 should report that agent-1 is registered with 4 slots

      When agent-2 is running and observing llamacpp-2, and registered at balancer-1
      Then balancer-1 should report that agent-2 is registered with 3 slots

    Scenario: Agent cannot register
      When agent-1 stops running and observing llamacpp-1, deregistered from balancer-1
      Then balancer-1 should report that agent-1 does not exist

    Scenario: Agent reports error
      When llamacpp-2 stops running
      Then balancer-1 should report that agent-1 cannot fetch llama.cpp-2
