package netcfg

import (
	"context"
	"io"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/feature/ec2/imds"
	"github.com/hashicorp/go-hclog"
)

type HostResolver struct {
	Logger hclog.Logger

	awsConfig *aws.Config
}

func (self *HostResolver) ResolveHost(ctx context.Context, host string) (string, error) {
	switch host {
	case "aws:metadata:local-ipv4":
		return self.resolveAwsLocalIpv4(ctx)
	default:
		return host, nil
	}
}

func (self *HostResolver) loadAwsConfig(ctx context.Context) (*aws.Config, error) {
	if self.awsConfig != nil {
		return self.awsConfig, nil
	}

	cfg, err := config.LoadDefaultConfig(ctx)

	if err != nil {
		return nil, err
	}

	self.awsConfig = &cfg

	return self.awsConfig, nil
}

func (self *HostResolver) resolveAwsLocalIpv4(ctx context.Context) (string, error) {
	cfg, err := self.loadAwsConfig(ctx)

	if err != nil {
		return "", err
	}

	imdsClient := imds.NewFromConfig(*cfg)

	localip, err := imdsClient.GetMetadata(ctx, &imds.GetMetadataInput{
		Path: "local-ipv4",
	})

	if err != nil {
		return "", err
	}

	actualContent, err := io.ReadAll(localip.Content)

	if err != nil {
		return "", err
	}

	self.Logger.Debug(
		"resolved aws:metadata:local-ipv4",
		"local-ipv4", string(actualContent),
	)

	return string(actualContent), nil
}
