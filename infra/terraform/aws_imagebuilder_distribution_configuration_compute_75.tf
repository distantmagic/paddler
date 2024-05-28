resource "aws_imagebuilder_distribution_configuration" "llamacpp_gpu_compute_75" {
  name = "llamacpp_gpu_compute_75"

  distribution {
    region = data.aws_region.current.name
    ami_distribution_configuration {
      name = "llamacpp-gpu-compute_75-{{ imagebuilder:buildDate }}"
    }
  }
}
