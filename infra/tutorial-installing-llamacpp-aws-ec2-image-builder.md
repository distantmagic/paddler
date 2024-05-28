# Installing llama.cpp with AWS EC2 Image Builder

This tutorial explains how to install [llama.cpp](https://github.com/ggerganov/llama.cpp) with [AWS EC2 Image Builder](https://aws.amazon.com/image-builder/).

By putting [llama.cpp](https://github.com/ggerganov/llama.cpp) in EC2 Image Builder pipeline, you can automatically build custom AMIs with [llama.cpp](https://github.com/ggerganov/llama.cpp) pre-installed.

You can also use that AMI as a base and add your foundational model on top of it. Thanks to that, you can quickly scale up or down your [llama.cpp](https://github.com/ggerganov/llama.cpp) groups.

We will repackage [the base EC2 tutorial](tutorial-installing-llamacpp-aws-cuda.md) as a set of Image Builder Components and Workflow.

You can complete the tutorial steps either manually or by automating the setup with [Terraform](https://www.terraform.io/)/[OpenTofu](https://opentofu.org/).

## Installation Steps

1. Create IAM `imagebuilder` role. It needs to have the following policies attached:
    * `arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role`
    * `arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilderECRContainerBuilds`
    * `arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilder`
    * `arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore`

    That role will be used by Image Builder to set up the instances. It needs some EC2 permissions to spin up a build instance and take a snapshot of it.
    (see: [source file](terraform/aws/aws_iam_role_imagebuilder_role.tf))

    You should end up with a role looking like this:

    ![image](https://github.com/distantmagic/paddler/assets/1286785/570e57e5-9049-4907-b7de-98a1e3a2de93)

3. Add necessary components (you can refer to [the generic EC2 tutorial for more details](tutorial-installing-llamacpp-aws-cuda.md)):
    1. llama.cpp build dependencies. It needs to install `build-essentials` and `ccache` (see: [source file](terraform/aws/aws_imagebuilder_component_apt_build_essential.tf))
    2. CUDA toolkit (see: [source file](terraform/aws/aws_imagebuilder_component_cuda_toolkit_12.tf))
    3. NVIDIA driver (see: [source file](terraform/aws/aws_imagebuilder_component_apt_nvidia_driver_555.tf))
    4. llama.cpp itself (see: [source file](terraform/aws/aws_imagebuilder_component_llamacpp_gpu_compute_75.tf))

    Those components are the building blocks in our Image Builder pipeline.

    ![image](https://github.com/distantmagic/paddler/assets/1286785/0f1a34e3-b950-4f1e-a555-862922835e22)

4. Add Infrastructure Configuration

    It should use `g4dn.xlarge` or any other instance type that supports CUDA (in case you want to use llama.cpp with cuda). In general, you should use exactly the same instance type you plan to use later in the production environment.

    It should have the `imagebuilder` role attached, which we created previously.

    (see: [source file](terraform/aws/aws_imagebuilder_infrastructure_configuration_llamacpp_gpu_compute_75.tf))

    ![image](https://github.com/distantmagic/paddler/assets/1286785/d61f2eec-d753-46ef-9dde-0996f5e481bb)

6. Add Distribution Configuration

    It specifies how the AMI should be distributed (on what type of base AMI it will be published)

    (see: [source file](terraform/aws/aws_imagebuilder_distribution_configuration_compute_75.tf))

    ![image](https://github.com/distantmagic/paddler/assets/1286785/6de3085b-1ce5-4359-a77f-b603cdfec383)

7. Add Image Pipeline

    The pipeline should use the Components, Infrastructure Configuration, and Distribution Configuration we prepared previously.

    (see: [source file](terraform/aws/aws_imagebuilder_image_pipeline_llamacpp_gpu_compute_75.tf))

    ![image](https://github.com/distantmagic/paddler/assets/1286785/a7ceaddd-5d0a-4aa2-9304-43e7c80d5f41)

    The recipe should use all the components we prepared beforehand:

    ![image](https://github.com/distantmagic/paddler/assets/1286785/81b3627f-ba2b-4404-a35d-0174ca861ef9)

8. Build the image

    Now you should be able to run the pipeline and prepare the image:

    ![image](https://github.com/distantmagic/paddler/assets/1286785/94716142-00bf-431c-89d5-9a9b15a38cbf)

9. Launch test EC2 Instance

    When launching EC2 instance, the llama.cpp image we prepared should be available under `My AMIs` list:

    ![image](https://github.com/distantmagic/paddler/assets/1286785/1e571f84-61d8-461d-aef6-a373f6a0020b)

## Summary

Check out `infra/terraform/aws` if you have any issues. Feel free to open an issue if you find a bug in the tutorial or have ideas on how to improve it.

Contributions are always welcomed!
