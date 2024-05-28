resource "aws_iam_instance_profile" "imagebuilder" {
  name = "imagebuilder"
  role = aws_iam_role.imagebuilder_role.name
}