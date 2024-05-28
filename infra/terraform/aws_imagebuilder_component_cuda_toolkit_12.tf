resource "aws_imagebuilder_component" "cuda_toolkit_12" {
  data = yamlencode({
    phases = [{
      name = "build"
      steps = [
        {
          action = "ExecuteBash"
          inputs = {
            commands = [
              "wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.1-1_all.deb",
              "sudo dpkg -i cuda-keyring_1.1-1_all.deb",
              "sudo apt-get update",
              "DEBIAN_FRONTEND=noninteractive sudo apt-get -yq install cuda-toolkit-12-5"
            ]
          }
          name           = "apt_cuda_toolkit_12"
          onFailure      = "Abort"
          timeoutSeconds = 600
        },
        {
          action = "Reboot"
          name   = "reboot"
        }
      ]
    }]
    schemaVersion = 1.0
  })
  name     = "cuda_toolkit_12"
  platform = "Linux"
  supported_os_versions = [
    "Ubuntu 22"
  ]
  version = "1.0.0"
}
