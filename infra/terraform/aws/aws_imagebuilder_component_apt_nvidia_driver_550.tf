resource "aws_imagebuilder_component" "apt_nvidia_driver_550" {
  data = yamlencode({
    phases = [{
      name = "build"
      steps = [
        {
          action = "ExecuteBash"
          inputs = {
            commands = [
              "sudo apt-get update",
              "DEBIAN_FRONTEND=noninteractive sudo apt-get install -yq nvidia-driver-550",
            ]
          }
          name           = "apt_nvidia_driver_550"
          onFailure      = "Abort"
          timeoutSeconds = 180
        },
        {
          action = "Reboot"
          name   = "reboot"
        }
      ]
    }]
    schemaVersion = 1.0
  })
  name     = "apt_nvidia_driver_550"
  platform = "Linux"
  supported_os_versions = [
    "Ubuntu 22"
  ]
  version = "1.0.0"
}
