// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

package dataplane_test

import (
	context "context"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"go.githedgehog.com/gateway-proto/pkg/dataplane"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
)

func TestDataplaneClient(t *testing.T) {
	l, err := net.Listen("tcp", ":0")
	require.NoError(t, err, "failed to listen on port")
	defer l.Close()

	server := grpc.NewServer()
	dataplane.RegisterConfigServiceServer(server, dataplane.NewMockConfigServiceServer())
	defer server.Stop()

	go func() {
		err := server.Serve(l)
		require.NoError(t, err, "failed to serve gRPC server")
	}()

	conn, err := grpc.NewClient(l.Addr().String(),
		grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err, "failed to create gRPC client")
	defer conn.Close()

	client := dataplane.NewConfigServiceClient(conn)

	ctx, cancel := context.WithTimeout(t.Context(), 30*time.Second)
	defer cancel()

	{
		resp, err := client.UpdateConfig(ctx, &dataplane.UpdateConfigRequest{
			Config: &dataplane.GatewayConfig{
				Generation: 42,
			},
		}, grpc.WaitForReady(true))
		require.NoError(t, err, "failed to update config")
		require.Equal(t, codes.OK, status.Code(err), "unexpected error code for update config")
		require.Equal(t, dataplane.Error_ERROR_NONE, resp.Error, "unexpected response for update config")
	}

	{
		resp, err := client.GetConfig(ctx, &dataplane.GetConfigRequest{}, grpc.WaitForReady(true))
		require.NoError(t, err, "failed to get config")
		require.Equal(t, codes.OK, status.Code(err), "unexpected error code for get config")
		require.Equal(t, uint64(42), resp.Generation, "unexpected response for get config")
	}

	{
		resp, err := client.GetConfigGeneration(ctx, &dataplane.GetConfigGenerationRequest{}, grpc.WaitForReady(true))
		require.NoError(t, err, "failed to get config generation")
		require.Equal(t, codes.OK, status.Code(err), "unexpected error code for get config generation")
		require.Equal(t, uint64(42), resp.Generation, "unexpected response for get config generation")
	}
}
