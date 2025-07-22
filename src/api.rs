use actix_web::{
    web, App, HttpResponse, HttpServer, Result as ActixResult, middleware::Logger,
};
use dyn_plug_core::{PluginManager, PluginError};
use log::{info, error, warn, debug};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    let start_time = Instant::now();
    info!("API: Listing all plugins");
    
    let manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("API: Failed to acquire plugin manager lock: {} (category: lock_error)", e);
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
    
    let duration = start_time.elapsed();
    info!("API: Found {} plugins in {}ms (category: list_success)", 
          plugin_infos.len(), duration.as_millis());
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(plugin_infos)))
}

/// POST /plugins/{name}/execute - Execute a plugin
pub async fn execute_plugin(
    path: web::Path<String>,
    payload: web::Json<ExecuteRequest>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let start_time = Instant::now();
    let plugin_name = path.into_inner();
    let input = &payload.input;
    
    info!("API: Executing plugin '{}' with input length: {}", plugin_name, input.len());
    debug!("API: Plugin '{}' input content: '{}'", plugin_name, 
           if input.len() > 100 { 
               format!("{}...", &input[..100]) 
           } else { 
               input.to_string() 
           });
    
    let manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("API: Failed to acquire plugin manager lock: {} (category: lock_error)", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.execute_plugin(&plugin_name, input) {
        Ok(result) => {
            let api_duration = start_time.elapsed();
            if result.success {
                info!("API: Plugin '{}' executed successfully in {}ms (API overhead: {}ms, category: execute_success)", 
                      plugin_name, result.duration_ms, api_duration.as_millis().saturating_sub(result.duration_ms as u128));
                let execution_result = ExecutionResult {
                    plugin: plugin_name,
                    output: result.output,
                    duration_ms: result.duration_ms,
                };
                Ok(HttpResponse::Ok().json(ApiResponse::success(execution_result)))
            } else {
                warn!("API: Plugin '{}' execution failed in {}ms: {} (category: execute_failed)", 
                      plugin_name, result.duration_ms, result.output);
                Ok(HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error(format!("Plugin execution failed: {}", result.output))))
            }
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found (category: not_found)", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(PluginError::PluginDisabled { .. }) => {
            warn!("API: Plugin '{}' is disabled (category: plugin_disabled)", plugin_name);
            Ok(HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' is disabled", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to execute plugin '{}': {} (category: {})", plugin_name, e, e.category());
            
            let status_code = match &e {
                PluginError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
                PluginError::PluginDisabled { .. } => actix_web::http::StatusCode::BAD_REQUEST,
                PluginError::TimeoutError { .. } => actix_web::http::StatusCode::REQUEST_TIMEOUT,
                PluginError::ResourceExhausted { .. } => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Ok(HttpResponse::build(status_code)
                .json(ApiResponse::<()>::error(e.user_friendly_message())))
        }
    }
}

/// PUT /plugins/{name}/enable - Enable a plugin
pub async fn enable_plugin(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let start_time = Instant::now();
    let plugin_name = path.into_inner();
    
    info!("API: Enabling plugin '{}'", plugin_name);
    
    let mut manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("API: Failed to acquire plugin manager lock: {} (category: lock_error)", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.enable_plugin(&plugin_name) {
        Ok(()) => {
            let duration = start_time.elapsed();
            info!("API: Plugin '{}' enabled successfully in {}ms (category: enable_success)", 
                  plugin_name, duration.as_millis());
            Ok(HttpResponse::Ok()
                .json(ApiResponse::success(format!("Plugin '{}' enabled successfully", plugin_name))))
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found (category: not_found)", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to enable plugin '{}': {} (category: {})", plugin_name, e, e.category());
            
            let status_code = match &e {
                PluginError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
                PluginError::ConfigError { .. } => actix_web::http::StatusCode::BAD_REQUEST,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Ok(HttpResponse::build(status_code)
                .json(ApiResponse::<()>::error(e.user_friendly_message())))
        }
    }
}

/// PUT /plugins/{name}/disable - Disable a plugin
pub async fn disable_plugin(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let start_time = Instant::now();
    let plugin_name = path.into_inner();
    
    info!("API: Disabling plugin '{}'", plugin_name);
    
    let mut manager = match data.plugin_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            error!("API: Failed to acquire plugin manager lock: {} (category: lock_error)", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error".to_string())));
        }
    };
    
    match manager.disable_plugin(&plugin_name) {
        Ok(()) => {
            let duration = start_time.elapsed();
            info!("API: Plugin '{}' disabled successfully in {}ms (category: disable_success)", 
                  plugin_name, duration.as_millis());
            Ok(HttpResponse::Ok()
                .json(ApiResponse::success(format!("Plugin '{}' disabled successfully", plugin_name))))
        }
        Err(PluginError::NotFound { .. }) => {
            warn!("API: Plugin '{}' not found (category: not_found)", plugin_name);
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error(format!("Plugin '{}' not found", plugin_name))))
        }
        Err(e) => {
            error!("API: Failed to disable plugin '{}': {} (category: {})", plugin_name, e, e.category());
            
            let status_code = match &e {
                PluginError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
                PluginError::ConfigError { .. } => actix_web::http::StatusCode::BAD_REQUEST,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Ok(HttpResponse::build(status_code)
                .json(ApiResponse::<()>::error(e.user_friendly_message())))
        }
    }
}

/// GET /health - Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    debug!("API: Health check requested (category: health_check)");
    
    #[derive(Serialize)]
    struct HealthStatus {
        status: String,
        timestamp: String,
        version: String,
        uptime_ms: u64,
    }
    
    // Simple uptime tracking (could be enhanced with actual process start time)
    static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let start_time = START_TIME.get_or_init(|| Instant::now());
    let uptime = start_time.elapsed();
    
    let health = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_ms: uptime.as_millis() as u64,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(health)))
}



/// Start the HTTP API server with graceful shutdown support
pub async fn start_server(
    plugin_manager: PluginManager,
    host: &str,
    port: u16,
    mut shutdown_signal: tokio::sync::mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTTP API server with graceful shutdown on {}:{}", host, port);
    
    let plugin_manager = Arc::new(Mutex::new(plugin_manager));
    
    // Create the HTTP server
    let server = HttpServer::new(move || {
        let app_state = AppState { plugin_manager: plugin_manager.clone() };
        
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .wrap(actix_web::middleware::DefaultHeaders::new()
                .add(("X-Service", "DynPlug Plugin System")))
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
    .bind(format!("{}:{}", host, port))
    .map_err(|e| {
        error!("Failed to bind server to {}:{}: {}", host, port, e);
        e
    })?;
    
    // Start the server and handle graceful shutdown
    let server_handle = server.run();
    
    tokio::select! {
        result = server_handle => {
            match result {
                Ok(()) => {
                    info!("HTTP server completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("HTTP server error: {}", e);
                    Err(Box::new(e) as Box<dyn std::error::Error>)
                }
            }
        }
        _ = shutdown_signal.recv() => {
            info!("Shutdown signal received, stopping HTTP server gracefully...");
            
            // Give the server a moment to finish current requests
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            info!("HTTP server shutdown completed");
            Ok(())
        }
    }
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