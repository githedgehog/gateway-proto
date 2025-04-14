use tonic::{transport::Server, Request, Response, Status};

// Include the generated protobuf code
pub mod config {
   include!("protobuf/config.rs");
}

use config::{
    config_service_server::{ConfigService, ConfigServiceServer},
    Error, GatewayConfig, GetConfigGenerationRequest, GetConfigGenerationResponse,
    GetConfigRequest, UpdateConfigRequest, UpdateConfigResponse,
};

// Our implementation of the gRPC service
#[derive(Default)]
pub struct ConfigServiceImpl {
    // In a real implementation, this would be a shared state with proper synchronization
    config: std::sync::Mutex<GatewayConfig>,
}

#[tonic::async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config(
        &self,
        _request: Request<GetConfigRequest>,
    ) -> Result<Response<GatewayConfig>, Status> {
        println!("Got a request for config");
        
        // Clone the current config to return it
        let config = self.config.lock().unwrap().clone();
        Ok(Response::new(config))
    }

    async fn get_config_generation(
        &self,
        _request: Request<GetConfigGenerationRequest>,
    ) -> Result<Response<GetConfigGenerationResponse>, Status> {
        println!("Got a request for config generation");
        
        let generation = self.config.lock().unwrap().generation;
        Ok(Response::new(GetConfigGenerationResponse { generation }))
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        println!("Got a request to update config");
        
        let update_request = request.into_inner();
        let new_config = update_request.config.unwrap();
        
        // Apply the new config
        // In a real implementation, this would involve more complex logic
        match apply_config(&new_config) {
            Ok(_) => {
                // Update our stored config
                let mut config = self.config.lock().unwrap();
                *config = new_config;
                
                Ok(Response::new(UpdateConfigResponse {
                    error: Error::None as i32,
                    message: "Config updated successfully".to_string(),
                }))
            }
            Err(e) => Ok(Response::new(UpdateConfigResponse {
                error: Error::ApplyFailed as i32,
                message: format!("Failed to apply config: {}", e),
            })),
        }
    }
}

// Placeholder functions for validation and application logic
fn validate_config(_config: &GatewayConfig) -> bool {
    // Implement your validation logic here
    true
}

fn apply_config(_config: &GatewayConfig) -> Result<(), String> {
    // Implement your application logic here
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = ConfigServiceImpl::default();
    
    println!("ConfigService server listening on {}", addr);
    
    Server::builder()
        .add_service(ConfigServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}