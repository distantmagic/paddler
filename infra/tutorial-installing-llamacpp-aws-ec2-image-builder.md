# Installing llama.cpp on AWS EC2 Image Builder

This tutorial explains how to install [llama.cpp](https://github.com/ggerganov/llama.cpp) on [AWS EC2 Image Builder](https://aws.amazon.com/image-builder/).

By putting [llama.cpp](https://github.com/ggerganov/llama.cpp) in EC2 Image Builder pipeline, you can automatically build custom AMIs with [llama.cpp](https://github.com/ggerganov/llama.cpp) pre-installed.

You can also use that AMI as a base and add your foundational model on top of it. Thanks to that you can scale up or down your [llama.cpp](https://github.com/ggerganov/llama.cpp) groups quickly.

We will essentially repackage [the base EC2 tutorial](tutorial-installing-llamacpp-aws-cuda.md) as a set of Image Builder Components and Workflow.

We will be using [Terraform](https://www.terraform.io/)/[OpenTofu](https://opentofu.org/) to automate the infrastructure setup.

