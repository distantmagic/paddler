# Installing llama.cpp with AWS EC2 Image Builder via AWS GUI

This tutorial explains how to install [llama.cpp](https://github.com/ggerganov/llama.cpp) with [AWS EC2 Image Builder](https://aws.amazon.com/image-builder/) using the AWS GUI. It's an adjacent tutorial to [Installing llama.cpp with AWS EC2 Image Builder](tutorial-installing-llamacpp-aws-ec2-image-builder.md) that explains the same installation process using Terraform.

## Installation Steps

1. Create an IAM `imagebuilder` role. Go to the IAM Dashboard, click "Roles" from the left-hand menu, and select "AWS service" as the trusted entity type. Next, select "EC2" as the use case:

   ![screenshot-01](https://github.com/malzag/paddler/assets/12105347/9c841ee9-0f19-48fc-8386-4b5cb7507a4b)

   Next, assign the following policies:

   - `arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role`
   - `arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilderECRContainerBuilds`
   - `arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilder`
   - `arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore`

   Name your role (for example, "imagebuilder") and finish creating it. You should end up with permissions and trust relationships looking like this:

   ![screenshot-02](https://github.com/malzag/paddler/assets/12105347/cc6e56f1-91e0-472a-814d-6c9dc0c9ba81)
   ![screenshot-03](https://github.com/malzag/paddler/assets/12105347/97dee654-c146-4e68-b2a2-05a2a433b545)

2. Navigate to EC2 Image Builder service on AWS. From there, select "Components" from the menu. We'll need to add four components that will act as the building blocks in our Image Builder pipeline. You can refer to [the generic EC2 tutorial for more details](tutorial-installing-llamacpp-aws-cuda.md) for more information.
   
   Click "Create component". Next, for each component:

   - Choose "Build" as the component type
   - Select "Linux" as the image OS
   - Select "Ubuntu 22.04" as the compatible OS version

   Provide the following as component names and contents in YAML format:

   **Component name: apt_build_essential**
   ```yaml
    name: apt_build_essential
    description: "Component to install build essentials on Ubuntu"
    schemaVersion: '1.0'
    phases:
      - name: build
        steps:
          - name: InstallBuildEssential
            action: ExecuteBash
            inputs:
              commands:
                - sudo apt-get update
                - DEBIAN_FRONTEND=noninteractive sudo apt-get install -yq build-essential ccache
            onFailure: Abort
            timeoutSeconds: 180
   ```

   **Component name: apt_nvidia_driver_555**
   ```yaml
    name: apt_nvidia_driver_555
    description: "Component to install NVIDIA driver 555 on Ubuntu"
    schemaVersion: '1.0'
    phases:
      - name: build
        steps:
          - name: apt_nvidia_driver_555
            action: ExecuteBash
            inputs:
              commands:
                - sudo apt-get update
                - DEBIAN_FRONTEND=noninteractive sudo apt-get install -yq nvidia-driver-555
            onFailure: Abort
            timeoutSeconds: 180
          - name: reboot
            action: Reboot
   ```

   **Component name: cuda_toolkit_12**
   ```yaml
    name: cuda_toolkit_12
    description: "Component to install CUDA Toolkit 12 on Ubuntu"
    schemaVersion: '1.0'
    phases:
      - name: build
        steps:
          - name: apt_cuda_toolkit_12
            action: ExecuteBash
            inputs:
              commands:
                - wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.1-1_all.deb
                - sudo dpkg -i cuda-keyring_1.1-1_all.deb
                - sudo apt-get update
                - DEBIAN_FRONTEND=noninteractive sudo apt-get -yq install cuda-toolkit-12-5
            onFailure: Abort
            timeoutSeconds: 600
          - name: reboot
            action: Reboot
   ```

    **Component name: llamacpp_gpu_compute_75**
   ```yaml
    name: llamacpp_gpu_compute_75
    description: "Component to install and compile llama.cpp with CUDA compute capability 75 on Ubuntu"
    schemaVersion: '1.0'
    phases:
      - name: build
        steps:
          - name: compile
            action: ExecuteBash
            inputs:
              commands:
                - cd /opt
                - git clone https://github.com/ggerganov/llama.cpp.git
                - cd llama.cpp
                - |
                  CUDA_DOCKER_ARCH=compute_75 \
                  LD_LIBRARY_PATH="/usr/local/cuda-12/lib64:$LD_LIBRARY_PATH" \
                  LLAMA_CUDA=1 \
                  PATH="/usr/local/cuda-12/bin:$PATH" \
                  make -j
            onFailure: Abort
            timeoutSeconds: 1200
   ```        

   Once you're finished, you'll see all the created components you added on the list:
   
   ![screenshot-04](https://github.com/malzag/paddler/assets/12105347/c3d082a8-1971-471a-84a4-b806a14dd899)

3. Next, we'll create a new Infrastructure Configuration. Select it from the left-hand menu and click "Create". You'll need to use `g4dn.xlarge` instance type or any other instance type that supports CUDA. Name your configuration, select the IAM role you created in step 1, and select the instance, for example:

   ![screenshot-05](https://github.com/malzag/paddler/assets/12105347/9f5777b9-721e-4760-884b-e117b2bbc8a3)

4. Select Distribution settings in the left-hand menu to create a Distribution Configuration. It specifies how the AMI should be distributed (on what type of base AMI it will be published). Select Amazon Machine Image, name the configuration, and save:

   ![screenshot-06](https://github.com/malzag/paddler/assets/12105347/1f01e63d-db21-4bb4-906b-df4ea51e43b7)

6. Next, we'll add the Image Pipeline. It will use the Components, Infrastructure Configuration, and Distribution Configuration we prepared previously. Select "Imagie Pipeline" from the left-hand menu and click "Create". Name your image pipeline, and select the desired build schedule.

   As the second step, create a new recipe. Choose AMI, name the recipe:

   ![screenshot-07](https://github.com/malzag/paddler/assets/12105347/1d89b1ca-265b-4195-88e5-a965e124858f)
  
   Next, select the previously created components:
  
   ![screenshot-08](https://github.com/malzag/paddler/assets/12105347/c0fef492-dd04-40d6-b3d1-066c7baaf2d3)

6. The next step is to build the image. You should be able to run the pipeline:

   ![screenshot-09](https://github.com/malzag/paddler/assets/12105347/c1e54bcd-9f8f-44bb-a1e1-e6bde546fbc4)

7. Launch test EC2 Instance.

   When launching EC2 instance, the llama.cpp image we prepared should be available under `My AMIs` list:

   ![screenshot-10](https://github.com/malzag/paddler/assets/12105347/7e56bb7e-f458-4b4a-89c2-51dd35e656e9)



