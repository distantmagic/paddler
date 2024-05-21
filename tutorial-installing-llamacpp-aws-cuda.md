# Installation on AWS EC2 CUDA Instances

This tutorial was tested on `g4dn.xlarge` instance with `Ubuntu 24.04` operating 
system. This tutorial was specifically written for an installation on a `Ubuntu 24.04` machine.

## Installation Steps

1. Start an EC2 instance of any class with a GPU with CUDA support.  
    
    If you want to compile llama.cpp on this instance, you will need at least 4GB for CUDA drivers and enough space for your LLM of choice. I recommed at least 30GB. Perform the following steps of this tutorial on the instance you started.
   
2. Install NVIDIA Drivers
  ```shell
  sudo apt update
  ```

  ```shell
  sudo apt install nvidia-driver-550-server nvidia-headless-550-server nvidia-utils-550-server
  ```

3. Install CUDA Toolkit. Download it and follow instructions from
  https://developer.nvidia.com/cuda-downloads  

    At the time of writing this tutorial, the highest available Ubuntu version supported is 22.04. But do not fear! :) We'll get it to work with some small workarounds (see the [Potential Errors](#potential-errors) section)

4. Compile llama.cpp.
  Follow the official tutorial for the remaining steps. However, use `make LLAMA_CUDA=1` to compile the llama.cpp:
  https://github.com/ggerganov/llama.cpp/discussions/4225

## Potential Errors

### libtinfo5 is not installable

```
Some packages could not be installed. This may mean that you have
requested an impossible situation or if you are using the unstable
distribution that some required packages have not yet been created
or been moved out of Incoming.
The following information may help to resolve the situation:

The following packages have unmet dependencies:
 nsight-systems-2023.4.4 : Depends: libtinfo5 but it is not installable
E: Unable to correct problems, you have held broken packages.
```

It was removed from Ubuntu 24.04. One way to solve that is to add a repository
from Ubuntu 22.04 to your `/etc/sources.list.d` directory and install it from 
there.

You might consider 
[APT pinning](https://help.ubuntu.com/community/PinningHowto) to pin that 
specific version of the library, although it might not be necessary.

### CUDA Architecture Must Be Explicitly Provided

```
ERROR: For CUDA versions < 11.7 a target CUDA architecture must be explicitly 
provided via environment variable CUDA_DOCKER_ARCH, e.g. by running 
"export CUDA_DOCKER_ARCH=compute_XX" on Unix-like systems, where XX is the 
minimum compute capability that the code needs to run on. A list with compute 
capabilities can be found here: https://developer.nvidia.com/cuda-gpus
```

You need to check the mentioned page (https://developer.nvidia.com/cuda-gpus)
and pick the appropriate version for your instance's GPU. `g4dn` instances 
use T4 GPU, which would be `compute_75`.

```shell
CUDA_DOCKER_ARCH=compute_75 LLAMA_CUDA=1 make -j batched-bench
```

### NVCC not found

```
/bin/sh: 1: nvcc: not found
```

You need to add CUDA path to your shell environmental variables.

For example, with Bash and CUDA 12:

```
export PATH="/usr/local/cuda-12/bin:$PATH"
export LD_LIBRARY_PATH="/usr/local/cuda-12/lib64:$LD_LIBRARY_PATH"
```

### cannot find -lcuda

```
/usr/bin/ld: cannot find -lcuda: No such file or directory
```

That means your Nvidia drivers are not installed. Install NVIDIA Drivers first.

### Cannot communicate with NVIDIA driver

```
NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver. Make sure that the latest NVIDIA driver is installed and running.
```

If you installed the drivers, reboot the instance.
