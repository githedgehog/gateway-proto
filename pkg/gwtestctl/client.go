// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

package gwtestctl

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"time"

	"go.githedgehog.com/gateway-proto/pkg/dataplane"
	"go.githedgehog.com/gateway-proto/pkg/protoyaml"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
)

func DoGetConfig(ctx context.Context, target string, wait bool) error {
	if !wait {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, 5*time.Second)
		defer cancel()
	}

	slog.Info("Getting config", "target", target)

	client, close, err := getClient(target)
	if err != nil {
		return fmt.Errorf("getting client: %w", err)
	}
	defer close()

	resp, err := client.GetConfig(ctx, &dataplane.GetConfigRequest{}, withOpts(wait)...)
	slog.Info("Response", "status", status.Code(err))
	if err != nil {
		slog.Error("Response", "error", err)

		return fmt.Errorf("getting config: %w", err)
	}

	slog.Info("Response", "generation", resp.Generation)

	data, err := protoyaml.MarshalYAML(resp)
	if err != nil {
		return fmt.Errorf("marshalling config to YAML: %w", err)
	}

	fmt.Println(string(data))

	return nil
}

func DoGetConfigGeneration(ctx context.Context, target string, wait bool) error {
	if !wait {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, 5*time.Second)
		defer cancel()
	}

	slog.Info("Getting config generation", "target", target)

	client, close, err := getClient(target)
	if err != nil {
		return fmt.Errorf("getting client: %w", err)
	}
	defer close()

	resp, err := client.GetConfigGeneration(ctx, &dataplane.GetConfigGenerationRequest{}, withOpts(wait)...)
	slog.Info("Response", "status", status.Code(err))
	if err != nil {
		slog.Error("Response", "error", err)

		return fmt.Errorf("getting config generation: %w", err)
	}

	fmt.Println(resp.Generation)

	return nil
}

func DoUpdateConfig(ctx context.Context, target, configFile string, wait bool) error {
	if !wait {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, 5*time.Second)
		defer cancel()
	}

	slog.Info("Updating config", "target", target, "file", configFile)

	data, err := os.ReadFile(configFile)
	if err != nil {
		return fmt.Errorf("reading config file: %w", err)
	}

	var config dataplane.GatewayConfig
	if err := protoyaml.UnmarshalYAML(data, &config); err != nil {
		return fmt.Errorf("unmarshalling config from YAML: %w", err)
	}

	client, close, err := getClient(target)
	if err != nil {
		return fmt.Errorf("getting client: %w", err)
	}
	defer close()

	resp, err := client.UpdateConfig(ctx, &dataplane.UpdateConfigRequest{
		Config: &config,
	}, withOpts(wait)...)
	slog.Info("Response", "status", status.Code(err))
	if err != nil {
		slog.Error("Response", "error", err)

		return fmt.Errorf("updating config: %w", err)
	}

	slog.Info("Response", "message", resp.Message, "error", resp.Error)

	if resp.Error != dataplane.Error_ERROR_NONE {
		return fmt.Errorf("updating config returned error: %s", resp.Error)
	}

	return nil
}

func getClient(target string) (dataplane.ConfigServiceClient, func() error, error) {
	scheme, target, err := parseTarget(target)
	if err != nil {
		return nil, nil, fmt.Errorf("parsing target: %w", err)
	}

	if scheme == unixScheme {
		target = fmt.Sprintf("unix-abstract:%s", target)
	}

	conn, err := grpc.NewClient(target,
		grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, nil, fmt.Errorf("creating gRPC client: %w", err)
	}

	return dataplane.NewConfigServiceClient(conn), conn.Close, nil
}

func withOpts(wait bool) []grpc.CallOption {
	var opts []grpc.CallOption
	if wait {
		opts = append(opts, grpc.WaitForReady(true))
	}

	return opts
}
