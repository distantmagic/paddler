resource "aws_imagebuilder_image" "llamacpp_gpu_compute_75" {
  distribution_configuration_arn   = aws_imagebuilder_distribution_configuration.llamacpp_gpu_compute_75.arn
  image_recipe_arn                 = aws_imagebuilder_image_recipe.llamacpp_gpu_compute_75.arn
  infrastructure_configuration_arn = aws_imagebuilder_infrastructure_configuration.llamacpp_gpu_compute_75.arn
}
