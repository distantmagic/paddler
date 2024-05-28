resource "aws_iam_role" "imagebuilder_role" {
  name = "imagebuilder-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
        Action = "sts:AssumeRole"
      }
    ]
  })
}

# https://docs.aws.amazon.com/imagebuilder/latest/userguide/image-builder-setting-up.html

resource "aws_iam_role_policy_attachment" "aws_ec2_container_service_for_ec2_role" {
  role       = aws_iam_role.imagebuilder_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role"
}

resource "aws_iam_role_policy_attachment" "ec2_instance_profile_for_imagebuilder_ecr_container_builds" {
  role       = aws_iam_role.imagebuilder_role.name
  policy_arn = "arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilderECRContainerBuilds"
}

resource "aws_iam_role_policy_attachment" "ec2_instance_profile_for_imagebuilder" {
  role       = aws_iam_role.imagebuilder_role.name
  policy_arn = "arn:aws:iam::aws:policy/EC2InstanceProfileForImageBuilder"
}

resource "aws_iam_role_policy_attachment" "ssm_managed_instance_core" {
  role       = aws_iam_role.imagebuilder_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}
