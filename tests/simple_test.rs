// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;
use tonic::{Request, Response, Status};

use gateway_config::{
    ConfigService, ConfigServiceClient, ConfigServiceServer, GetConfigGenerationRequest,
    GetConfigGenerationResponse,
};

struct SimpleConfigService {
    generation: i64,
}

impl SimpleConfigService {
    fn new(generation: i64) -> Self {
        Self { generation }
    }
}

#[tonic::async_trait]
impl ConfigService for SimpleConfigService {
    async fn get_config_generation(
        &self,
        _request: Request<GetConfigGenerationRequest>,
    ) -> Result<Response<GetConfigGenerationResponse>, Status> {
        println!("Server received get_config_generation request");
        Ok(Response::new(GetConfigGenerationResponse {
            generation: self.generation,
        }))
    }

    async fn get_config(
        &self,
        _request: Request<gateway_config::GetConfigRequest>,
    ) -> Result<Response<gateway_config::GatewayConfig>, Status> {
        Err(Status::unimplemented(
            "get_config not implemented in this test",
        ))
    }

    async fn update_config(
        &self,
        _request: Request<gateway_config::UpdateConfigRequest>,
    ) -> Result<Response<gateway_config::UpdateConfigResponse>, Status> {
        Err(Status::unimplemented(
            "update_config not implemented in this test",
        ))
    }
}

#[tokio::test]
async fn test_simple_generation_request() {
    let service = SimpleConfigService::new(228);
    let server = ConfigServiceServer::new(service);
    let addr: SocketAddr = "[::1]:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    println!("Server will listen on: {}", server_addr);

    let server_uri = if server_addr.is_ipv6() {
        format!("http://[{}]:{}", server_addr.ip(), server_addr.port())
    } else {
        format!("http://{}:{}", server_addr.ip(), server_addr.port())
    };

    println!("Server URI: {}", server_uri);

    tokio::spawn(async move {
        println!("Starting gRPC server...");

        tonic::transport::Server::builder()
            .add_service(server)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    println!("Connecting client to: {}", server_uri);

    let channel = tonic::transport::Channel::from_shared(server_uri)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = ConfigServiceClient::new(channel);

    println!("Sending request...");
    let request = Request::new(GetConfigGenerationRequest {});
    let response = client.get_config_generation(request).await.unwrap();
    let result = response.into_inner();

    println!("Received response with generation: {}", result.generation);
    assert_eq!(result.generation, 228);
}
