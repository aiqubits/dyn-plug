use actix_web::{
    web, App, HttpResponse, HttpServer, Result as ActixResult, middleware::Logger,
};
use dyn_plug_core::{PluginManager, PluginError};
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// API response wrapper for consistent response format
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Plugin execution request payload
#[derive(Deserialize, Serialize)]
pub struct ExecuteRequest {
    #[serde(default)]
    pub input: String,
}

/// Plugin execution result
#[derive(Serialize)]
pub struct ExecutionResult {
    pub plugin: String,
    pub output: String,
    pub duration_ms: u64,
}

/// Plugin information for API responses
#[derive(Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub loaded: bool,
}

/// Application state containing the plugin manager
pub struct AppState {
    pub plugin_manager: Arc<Mutex<PluginManager>>,
}

/// GET /plugins - List all plugins with their status
pub async fn list_plugins(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("API: Listing all plugins");
    
    let manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to acquire plugin manager lock: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    let plugins = manager.list_plugins();
    let plugin_infos: Vec<PluginInfo> = plugins
        .into_iter()
        .map(|p| PluginInfo {
            name: p.name,
            version: p.version,
            description: p.description,
            enabled: p.enabled && p.config_enabled,
            loaded: p.enabled,
        })
        .collect();
    
    info!("API: Found {} plugins", plugin_infos.len());
    Ok(HttpResponse::Ok().json(ApiResponse::success(plugin_infos)))
}

/// POST /plugins/{name}/execute - Execute a plugin
pub async fn execute_plugin(
    path: web::Path<String>,
    payload: web::Json<ExecuteRequest>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let plugin_name = path.into_inner();
    let input = &payload.input;
    
    info!("API: Executing plugin '{}' with input: '{}'", plugin_name, input);
    
    let manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to acquire plugin manager lock: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.execute_plugin(&plugin_name, input) {
        Ok(result) => {
            if result.success {
                info!("API: Plugin '{}' executed successfully in {}ms", plugin_name, result.duration_ms);
                let execution_result = ExecutionResult {
                    plugin: plugin_name,
                    output: result.output,
                    duration_ms: result.duration_ms,
                };
                Ok(HttpResponse::Ok().json(ApiResponse::success(execution_result)))
            } else {
                warn!("API: Plugin '{}' execution failed: {}", plugin_name, result.output);
                Ok(HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error(format!("Plugin execution failed: {}", result.output))))
            }
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(PluginError::PluginDisabled { .. }) => {
            warn!("API: Plugin '{}' is disabled", plugin_name);
            Ok(HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' is disabled", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to execute plugin '{}': {}", plugin_name, e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("Failed to execute plugin: {}", e))))
        }
    }
}

/// PUT /plugins/{name}/enable - Enable a plugin
pub async fn enable_plugin(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let plugin_name = path.into_inner();
    
    info!("API: Enabling plugin '{}'", plugin_name);
    
    let mut manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to acquire plugin manager lock: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.enable_plugin(&plugin_name) {
        Ok(()) => {
            info!("API: Plugin '{}' enabled successfully", plugin_name);
            Ok(HttpResponse::Ok()
                .json(ApiResponse::success(format!("Plugin '{}' enabled successfully", plugin_name))))
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to enable plugin '{}': {}", plugin_name, e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("Failed to enable plugin: {}", e))))
        }
    }
}

/// PUT /plugins/{name}/disable - Disable a plugin
pub async fn disable_plugin(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let plugin_name = path.into_inner();
    
    info!("API: Disabling plugin '{}'", plugin_name);
    
    let mut manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to acquire plugin manager lock: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.disable_plugin(&plugin_name) {
        Ok(()) => {
            info!("API: Plugin '{}' disabled successfully", plugin_name);
            Ok(HttpResponse::Ok()
                .json(ApiResponse::success(format!("Plugin '{}' disabled successfully", plugin_name))))
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to disable plugin '{}': {}", plugin_name, e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(format!("Failed to disable plugin: {}", e))))
        }
    }
}

/// GET /health - Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    info!("API: Health check requested");
    
    #[derive(Serialize)]
    struct HealthStatus {
        status: String,
        timestamp: String,
        version: String,
    }
    
    let health = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(health)))
}



/// Start the HTTP API server
pub async fn start_server(
    plugin_manager: PluginManager,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTTP API server on {}:{}", host, port);
    
    let plugin_manager = Arc::new(Mutex::new(plugin_manager));
    
    HttpServer::new(move || {
        let app_state = AppState { plugin_manager: plugin_manager.clone() };
        
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/plugins", web::get().to(list_plugins))
                    .route("/plugins/{name}/execute", web::post().to(execute_plugin))
                    .route("/plugins/{name}/enable", web::put().to(enable_plugin))
                    .route("/plugins/{name}/disable", web::put().to(disable_plugin))
                    .route("/health", web::get().to(health_check))
            )
            // Also expose health endpoint at root level
            .route("/health", web::get().to(health_check))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use dyn_plug_core::PluginManager;
    
    fn create_test_app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >
    > {
        let manager = PluginManager::new().expect("Failed to create plugin manager");
        let app_state = AppState { plugin_manager: Arc::new(Mutex::new(manager)) };
        
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .route("/plugins", web::get().to(list_plugins))
                    .route("/plugins/{name}/execute", web::post().to(execute_plugin))
                    .route("/plugins/{name}/enable", web::put().to(enable_plugin))
                    .route("/plugins/{name}/disable", web::put().to(disable_plugin))
                    .route("/health", web::get().to(health_check))
            )
            .route("/health", web::get().to(health_check))
    }
    
    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(create_test_app()).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
    
    #[actix_web::test]
    async fn test_list_plugins_endpoint() {
        let app = test::init_service(create_test_app()).await;
        let req = test::TestRequest::get().uri("/api/v1/plugins").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
    
    #[actix_web::test]
    async fn test_execute_nonexistent_plugin() {
        let app = test::init_service(create_test_app()).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/plugins/nonexistent/execute")
            .set_json(&ExecuteRequest {
                input: "test".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }
}