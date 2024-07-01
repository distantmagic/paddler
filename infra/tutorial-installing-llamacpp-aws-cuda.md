# Installation on AWS EC2 CUDA Instances

This tutorial was tested on `g4dn.xlarge` instance with `Ubuntu 22.04` operating 
system. This tutorial was written explicitly to perform the installation on a `Ubuntu 22.04` machine.

## Installation Steps

1. Start an EC2 instance of any class with a GPU with CUDA support.  
    
    If you want to compile llama.cpp on this instance, you will need at least 4GB for CUDA drivers and enough space for your LLM of choice. I recommend at least 30GB. Perform the following steps of this tutorial on the instance you started.

2. Install build dependencies:
    ```shell
    sudo apt update
    ```
    ```shell
    sudo apt install build-essential ccache
    ```
   
3. Install CUDA Toolkit (only the Base Installer). Download it and follow instructions from
  https://developer.nvidia.com/cuda-downloads  

    At the time of writing this tutorial, the highest available supported version of the Ubuntu version was 22.04. But do not fear! :) We'll get it to work with some minor workarounds (see the [Potential Errors](#potential-errors) section)

4. Install NVIDIA Drivers:
    ```shell
    sudo apt install nvidia-driver-555
    ```

5. Compile llama.cpp:
    ```shell
    git clone https://github.com/ggerganov/llama.cpp.git
    ```
    ```shell
    cd llama.cpp
    ```
    ```shell
    GGML_CUDA=1 make -j
    ```
5. Benchmark llama.cpp (optional):

    Follow the official tutorial if you intend to run the benchmark. However, keep using `GGML_CUDA=1 make` to compile the llama.cpp (do *not* use `LLAMA_CUBLAS=1`):
  https://github.com/ggerganov/llama.cpp/discussions/4225

    Instead of performing a model quantization yourself, you can download quantized models from Hugging Face. For example, `Mistral Instruct` you can download from https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/tree/main

## Potential Errors

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

For example:

```shell
CUDA_DOCKER_ARCH=compute_75 GGML_CUDA=1 make -j
```

### Failed to initialize CUDA

```
ggml_cuda_init: failed to initialize CUDA: unknown error
```

Sometimes can be solved with `sudo modprobe nvidia_uvm`. 

You can also create a Systemd unit that loads the module on boot:

```ini
[Unit]
After=nvidia-persistenced.service

[Service]
Type=oneshot
ExecStart=/usr/sbin/modprobe nvidia_uvm

[Install]
WantedBy=multi-user.target
```

### NVCC not found

```
/bin/sh: 1: nvcc: not found
```

You need to add CUDA path to your shell environmental variables.

For example, with Bash and CUDA 12:

```shell
export PATH="/usr/local/cuda-12/bin:$PATH"
```
```shell
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

### Failed to decode the batch

```
failed to decode the batch, n_batch = 0, ret = -1
main: llama_decode() failed
```

There are two potential causes of this issue.

#### Option 1: Install NVIDIA drivers

Make sure you have installed the CUDA Toolkit and NVIDIA drivers. If you do, restart your server and try again. Most likely, NVIDIA kernel modules are not loaded.

```shell
sudo reboot
```

#### Option 2: Use different benchmarking parameters

For example, with `Mistral Instruct 7B` what worked for me is:

```shell
./llama-batched-bench -m ../mistral-7b-instruct-v0.2.Q4_K_M.gguf 2048 2048 512 0 999 128,256,512 128,256 1,2,4,8,16,32
```
