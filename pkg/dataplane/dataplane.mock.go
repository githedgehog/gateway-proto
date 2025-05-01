// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

package dataplane

import (
	"context"
)

type MockConfigServiceServer struct {
	UnimplementedConfigServiceServer
	cfg  *GatewayConfig
	logF func(msg string, args ...any)
}

func NewMockConfigServiceServer(logF func(msg string, args ...any)) *MockConfigServiceServer {
	return &MockConfigServiceServer{
		cfg: &GatewayConfig{
			Generation: 0,
		},
		logF: logF,
	}
}

func (m *MockConfigServiceServer) GetObservedConfig() *GatewayConfig {
	return m.cfg
}

func (m *MockConfigServiceServer) log(msg string, args ...any) {
	if m.logF != nil {
		m.logF(msg, args...)
	}
}

var _ ConfigServiceServer = &MockConfigServiceServer{}

func (m *MockConfigServiceServer) GetConfig(context.Context, *GetConfigRequest) (*GatewayConfig, error) {
	m.log("GetConfig called", "gen", m.cfg.Generation)

	return m.cfg, nil
}

func (m *MockConfigServiceServer) GetConfigGeneration(context.Context, *GetConfigGenerationRequest) (*GetConfigGenerationResponse, error) {
	m.log("GetConfigGeneration called", "gen", m.cfg.Generation)

	return &GetConfigGenerationResponse{
		Generation: m.cfg.Generation,
	}, nil
}

func (m *MockConfigServiceServer) UpdateConfig(_ context.Context, req *UpdateConfigRequest) (*UpdateConfigResponse, error) {
	m.log("UpdateConfig called", "gen", m.cfg.Generation)

	m.cfg = req.Config

	return &UpdateConfigResponse{
		Error: Error_ERROR_NONE,
	}, nil
}
