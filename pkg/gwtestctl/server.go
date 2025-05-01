// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

package gwtestctl

import (
	"context"
	"fmt"
	"log/slog"
	"net"
	"os"
	"strings"

	"go.githedgehog.com/gateway-proto/pkg/dataplane"
	"google.golang.org/grpc"
)

const (
	tcpScheme  = "tcp"
	unixScheme = "unix"
)

func DoFakeServer(ctx context.Context, target string) error {
	scheme, target, err := parseTarget(target)
	if err != nil {
		return fmt.Errorf("parsing target: %w", err)
	}

	if scheme == unixScheme {
		if err := os.RemoveAll(target); err != nil && !os.IsNotExist(err) {
			return fmt.Errorf("removing existing unix socket: %w", err)
		}
	}

	l, err := net.Listen(scheme, target)
	if err != nil {
		return fmt.Errorf("listening: %w", err)
	}

	slog.Info("Starting fake server", "target", l.Addr().String())

	server := grpc.NewServer()
	dataplane.RegisterConfigServiceServer(server, dataplane.NewMockConfigServiceServer(true))

	if err := server.Serve(l); err != nil {
		return fmt.Errorf("serving gRPC: %w", err)
	}

	return nil
}

func parseTarget(target string) (string, string, error) {
	switch {
	case strings.HasPrefix(target, tcpScheme+"://"):
		return tcpScheme, strings.TrimPrefix(target, tcpScheme+"://"), nil
	case strings.HasPrefix(target, unixScheme+"://"):
		return unixScheme, strings.TrimPrefix(target, unixScheme+"://"), nil
	default:
		return "", "", fmt.Errorf("invalid target scheme: %s", target)
	}
}
