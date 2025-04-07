use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod pb {
    pub mod config {
        include!("protobuf/config.rs");
    }
}

#[derive(Debug, Default)]
pub struct SharedState {
    config: pb::config::GatewayConfig,
}

#[derive(Debug, Clone)]
pub struct DpService {
    state: Arc<Mutex<SharedState>>,
}

impl Default for DpService {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(SharedState {
                config: pb::config::GatewayConfig {
                    generation: 1,
                    devices: vec![],
                    peerings: vec![],
                    vrfs: vec![],
                },
            })),
        }
    }
}

#[tonic::async_trait]
impl pb::config::config_service_server::ConfigService for DpService {
    async fn get_config(
        &self,
        _request: Request<pb::config::GetConfigRequest>,
    ) -> Result<Response<pb::config::GatewayConfig>, Status> {
        let config = self.state.lock().unwrap().config.clone();
        Ok(Response::new(config))
    }

    async fn get_config_generation(
        &self,
        _request: Request<pb::config::GetConfigGenerationRequest>,
    ) -> Result<Response<pb::config::GetConfigGenerationResponse>, Status> {
        let generation = self.state.lock().unwrap().config.generation;
        Ok(Response::new(pb::config::GetConfigGenerationResponse { generation }))
    }

    async fn update_config(
        &self,
        request: Request<pb::config::UpdateConfigRequest>,
    ) -> Result<Response<pb::config::UpdateConfigResponse>, Status> {
        let mut state = self.state.lock().unwrap();
        let new_config = request.into_inner().config.unwrap();
        state.config = new_config;

        Ok(Response::new(pb::config::UpdateConfigResponse {
            error: pb::config::Error::None.into(),
            message: "Config applied".into(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = DpService::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(pb::config::config_service_server::ConfigServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
