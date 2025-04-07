# Feature: Balancer

#     Scenario: Balancer can loadbalance
#       Given balancer-1 is running at 0.0.0.0:8070
#       Given llamacpp-1 is running at 0.0.0.0:8080 with 4 slots
#       Given agent-1 is running and observing llamacpp-1, and registered at balancer-1
#       Given llamacpp-2 is running at 0.0.0.0:8081 with 3 slots
#       Given agent-2 is running and observing llamacpp-2, and registered at balancer-1

#       # When first request is proxied to balancer-1
#       # Then balancer-1 must tell 1 slot is busy
#       # When second request is proxied to balancer-1
#       # Then balancer-1 must tell 2 slots are busy
#       # Then balancer-1 should return a successful response

#       When 7 requests are proxied to balancer-1
#       Then balancer-1 must tell 7 slots are busy
#       When 1 request is proxied to balancer-1
#       Then balancer-1 should return an unsuccessful response
      