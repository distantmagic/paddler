resource "aws_imagebuilder_infrastructure_configuration" "llamacpp_gpu_compute_75" {
  description                   = "NVIDIA Compute 7.5"
  instance_profile_name         = aws_iam_instance_profile.imagebuilder.name
  instance_types                = ["g4dn.xlarge"]
  name                          = "llamacpp_gpu_compute_75"
  terminate_instance_on_failure = true
}
