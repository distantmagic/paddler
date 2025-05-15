Feature: Agent

    Background:
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second
      Given llamacpp-1 is running at 8080 with 4 slots
      Given llamacpp-2 is running at 8081 with 3 slots
      
      When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080

      When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 3 slots at 0.0.0.0:8081
        
      @serial
      Scenario: Agent cannot register
        When agent-1 stops running and observing llamacpp-1, unregistered from balancer-1
        Then balancer-1 in 0.0.0.0:8070 must report that agent-1 does not exist
      
      @serial
      Scenario: Agent reports error
        When llamacpp-2 stops running
        Then balancer-1 in 0.0.0.0:8070 must report that agent-2 cannot fetch llamacpp-2 in 0.0.0.0:8081