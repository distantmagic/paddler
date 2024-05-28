resource "aws_imagebuilder_component" "apt_build_essential" {
  data = yamlencode({
    phases = [{
      name = "build"
      steps = [{
        action = "ExecuteBash"
        inputs = {
          commands = [
            "sudo apt-get update",
            "DEBIAN_FRONTEND=noninteractive sudo apt-get install -yq build-essential ccache",
          ]
        }
        name           = "apt_build_essential"
        onFailure      = "Abort"
        timeoutSeconds = 180
      }]
    }]
    schemaVersion = 1.0
  })
  name     = "apt_build_essential"
  platform = "Linux"
  supported_os_versions = [
    "Ubuntu 22"
  ]
  version = "1.0.0"
}
