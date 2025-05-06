# Feature: Supervisor

#     Scenario: Supervisor can restart llamacpp after being killed
#       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in supervisor feature
#       Given supervisor-1 is running at 0.0.0.0:8087 with file configuration stored on supervisor-1.toml and starts llamacpp-1 at 8080 with 4 slots running in supervisor feature
#       Given supervisor-2 is running at 0.0.0.0:8088 with file configuration stored on supervisor-2.toml and starts llamacpp-2 at 8081 with 4 slots running in supervisor feature

#       When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070 in supervisor feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080 in supervisor feature

#       When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070 in supervisor feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 4 slots at 0.0.0.0:8081 in supervisor feature

#       When llamacpp-2 from supervisor-2 is killed in supervisor feature
#       When llamacpp-1 from supervisor-1 is killed in supervisor feature
#       When 2 requests are proxied to balancer-1 in 127.0.0.1:8071 in supervisor feature
#       Then balancer-1 must tell 2 slots are busy and 6 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2 in supervisor feature
#       Then balancer-1 must return a successful response in 127.0.0.1:8071 in supervisor feature

#     Scenario: Supervisor can start llamacpp and change its arguments
#       Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second in supervisor feature
#       Given supervisor-1 is running at 0.0.0.0:8087 with file configuration stored on supervisor-1.toml and starts llamacpp-1 at 8080 with 4 slots running in supervisor feature
#       Given supervisor-2 is running at 0.0.0.0:8088 with file configuration stored on supervisor-2.toml and starts llamacpp-2 at 8081 with 4 slots running in supervisor feature

#       When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070 in supervisor feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080 in supervisor feature

#       When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070 in supervisor feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 4 slots at 0.0.0.0:8081 in supervisor feature

#       When 1 request is proxied to supervisor-1 in 0.0.0.0:8087 to change slots to 2 and port to 8080 in supervisor feature
#       Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 2 slots at 0.0.0.0:8080 in supervisor feature

#       When 1 request is proxied to supervisor-2 in 0.0.0.0:8088 to change slots to 4 and port to 8082 in supervisor feature
#       Then "balancer-1" in "0.0.0.0:8070" must report that "agent-2" cannot fetch "llamacpp-2" in "0.0.0.0:8081" in supervisor feature
  