testing 4

# Paddler Docker Compose Example

This example demonstrates how to set up a complete Paddler cluster using Docker Compose.

## What's Included

This example sets up:

* **Two llama.cpp server instances** (`llm1` and `llm2`) - Running the llama.cpp server with slots enabled
* **Paddler load balancer** - Distributes requests across the llama.cpp instances with management dashboard enabled
* **Two Paddler agents** - Monitor each llama.cpp instance and report their status to the load balancer

## How to Run

1. **Navigate to the example directory:**

   ```bash
   cd example/
   ```

2. **Start the cluster:**

   ```bash
   docker compose up
   ```

3. **Access the services:**
   * **OpenAI API endpoint:** <http://localhost:8080>
   * **Management dashboard:** <http://localhost:8085/dashboard>

4. **Stop the cluster:**

   ```bash
   docker compose down
   ```

   To remove all data including cached models:

   ```bash
   docker compose down -v
   ```

## Configuration

### Environment Variables

The llama.cpp instances are configured via the `llm.env` file. You can customize:

## GPU Support

To use GPU acceleration with CUDA, follow these steps:

### 1. Install NVIDIA Container Toolkit

Install the NVIDIA Container Toolkit on your host system by following the official installation guide:
<https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html>

### 2. Update Docker Image Tag

Change the llama.cpp image tag in `compose.yaml` from:

```yaml
image: ghcr.io/ggml-org/llama.cpp:server-<version>
```

To the CUDA version:

```yaml
image: ghcr.io/ggml-org/llama.cpp:server-cuda-<version>
```

> **Note:** Replace `<version>` with the specific llama.cpp version you want to use.
> Check the [llama.cpp container packages](https://github.com/ggml-org/llama.cpp/pkgs/container/llama.cpp) for the latest available versions.

### 3. Configure GPU Assignment

To specify which GPU each instance should use, add the `device_ids` configuration:

#### Single GPU Setup

```yaml
services:
  llm1:
    <<: *llm-common
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

#### Multi-GPU Setup (Two GPUs)

For systems with multiple GPUs, you can assign specific GPU device IDs:

```yaml
services:
  llm1:
    <<: *llm-common
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              device_ids: ['0']
              capabilities: [gpu]
  
  llm2:
    <<: *llm-common
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              device_ids: ['1']
              capabilities: [gpu]
```

This configuration assigns GPU 0 to `llm1` and GPU 1 to `llm2`, allowing you to distribute the workload across multiple GPUs.

### 4. Verify GPU Usage

After starting the containers, you can verify GPU usage with:

```bash
nvidia-smi
```

You should see the Docker containers listed in the GPU processes.
