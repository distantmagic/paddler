resource "aws_imagebuilder_image_recipe" "llamacpp_gpu_compute_75" {
  name         = "llamacpp_gpu_compute_75"
  version      = "1.0.0"
  parent_image = "arn:${data.aws_partition.current.partition}:imagebuilder:${data.aws_region.current.name}:aws:image/ubuntu-server-22-lts-x86/x.x.x"

  component {
    component_arn = aws_imagebuilder_component.apt_build_essential.arn
  }

  component {
    component_arn = aws_imagebuilder_component.cuda_toolkit_12.arn
  }

  component {
    component_arn = aws_imagebuilder_component.apt_nvidia_driver_555.arn
  }

  component {
    component_arn = aws_imagebuilder_component.llamacpp_gpu_compute_75.arn
  }

  block_device_mapping {
    device_name = "/dev/sda1"

    ebs {
      delete_on_termination = true
      volume_size           = 30
      volume_type           = "gp2"
    }
  }
}
