Feature: Agent

    Scenario: Agent can register
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
      Given llamacpp-1 is running at 8080 with 4 slots
      Given llamacpp-2 is running at 8081 with 3 slots

      When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 must report that agent-1 is registered with 4 slots in 8080

      When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 must report that agent-2 is registered with 3 slots in 8081

    Scenario: Agent cannot register
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
      Given llamacpp-1 is running at 8080 with 4 slots
      Given llamacpp-2 is running at 8081 with 3 slots

      When agent-1 stops running and observing llamacpp-1, deregistered from balancer-1
      Then balancer-1 in 0.0.0.0:8070 must report that agent-1 does not exist

    Scenario: Agent reports error
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:8072
      Given llamacpp-1 is running at 8080 with 4 slots
      Given llamacpp-2 is running at 8081 with 3 slots

      When llamacpp-2 stops running
      Then balancer-1 in 0.0.0.0:8070 must report that agent-2 cannot fetch llama.cpp-2 in 0.0.0.0:8081
