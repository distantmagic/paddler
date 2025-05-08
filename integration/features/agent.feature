# Feature: Agent

#     Scenario: Agent can register
#       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in agent feature
#       Given llamacpp-1 is running at 8080 with 4 slots in agent feature
#       Given llamacpp-2 is running at 8081 with 3 slots in agent feature

#       When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080 in agent feature

#       When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 3 slots at 0.0.0.0:8081 in agent feature

#     Scenario: Agent cannot register
#       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in agent feature
#       Given llamacpp-1 is running at 8080 with 4 slots in agent feature
#       Given llamacpp-2 is running at 8081 with 3 slots in agent feature

#       When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080 in agent feature

#       When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 3 slots at 0.0.0.0:8081 in agent feature

#       When agent-1 stops running and observing llamacpp-1, unregistered from balancer-1 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 does not exist in agent feature

#     Scenario: Agent reports error
#       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in agent feature
#       Given llamacpp-1 is running at 8080 with 4 slots in agent feature
#       Given llamacpp-2 is running at 8081 with 3 slots in agent feature

#       When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080 in agent feature

#       When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070 in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 3 slots at 0.0.0.0:8081 in agent feature

#       When llamacpp-2 stops running in agent feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 cannot fetch llamacpp-2 in 0.0.0.0:8081 in agent feature