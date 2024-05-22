# Installing Paddler on AWS EC2 Instances

## Preparations

Before starting this tutorial, you need to have a running EC2 instance with llama.cpp already installed.

To learn how to install llama.cpp on EC2 instance, follow the other tutorial: [Installing llama.cpp on AWS EC2 CUDA Instances](tutorial-installing-llamacpp-aws-cuda.md). 

This tutorial picks up from the above tutorial.

# Notes

1. create IAM role for EC2 so it can report custom metrics to cloudwatch
2. create aletrs in cloudwatch at 70%, 80%, 90% taken slots to setup scaling policy later
3. install cloudwatch agent on the EC2 instance
4. install Paddler Agent on the EC2 instance
5. install Paddler statsd reporter on the EC2 instance (so it can report stats to cloudwatch)
6. create a template from EC2 instance so it can be used in load balancer
7. setup loadbalancer from EC2 template
8. setup dynamic scaling policy from cloudwatch alerts
9. install paddler balancer in front of the llama instances
10. use paddler balancer
