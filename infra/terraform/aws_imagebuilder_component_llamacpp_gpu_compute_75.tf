resource "aws_imagebuilder_component" "llamacpp_gpu_compute_75" {
  data = yamlencode({
    phases = [{
      name = "build"
      steps = [{
        action = "ExecuteBash"
        inputs = {
          commands = [
            "cd /opt",
            "git clone https://github.com/ggerganov/llama.cpp.git",
            "cd llama.cpp",
            <<COMPILE
            CUDA_DOCKER_ARCH=compute_75 \
            LD_LIBRARY_PATH="/usr/local/cuda-12/lib64:$LD_LIBRARY_PATH" \
            LLAMA_CUDA=1 \
            PATH="/usr/local/cuda-12/bin:$PATH" \
              make -j
            COMPILE
          ]
        }
        name           = "compile"
        onFailure      = "Abort"
        timeoutSeconds = 1200
      }]
    }]
    schemaVersion = 1.0
  })
  name     = "llamacpp_gpu_compute_75"
  platform = "Linux"
  supported_os_versions = [
    "Ubuntu 22"
  ]
  version = "1.0.0"
}
