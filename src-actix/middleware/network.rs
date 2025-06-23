use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpResponse,
};
use crate::configuration::configuration_data::Configuration;
use log::warn;
use std::future::{ready, Ready, Future};
use std::net::IpAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};

// Middleware for handling network configuration (CORS and IP filtering)
pub struct NetworkMiddleware;

impl<S, B> Transform<S, ServiceRequest> for NetworkMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = NetworkMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(NetworkMiddlewareService { service }))
    }
}

pub struct NetworkMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for NetworkMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = Configuration::get();
        
        // Check if the request is from an authorized host
        if !config.authorized_hosts.is_empty() {
            let connection_info = req.connection_info().clone();
            let client_ip = connection_info.peer_addr().unwrap_or("unknown");
            let client_ip = client_ip.split(':').next().unwrap_or(client_ip);
            
            let is_authorized = config.authorized_hosts.iter().any(|host| {
                // Check for an exact match (for hostnames like "localhost")
                if host == client_ip {
                    return true;
                }
                
                // Try to parse as IP address for more precise comparison
                if let (Ok(host_ip), Ok(client_addr)) = (
                    IpAddr::from_str(host),
                    IpAddr::from_str(client_ip)
                ) {
                    return host_ip == client_addr;
                }
                
                false
            });
            
            if !is_authorized {
                warn!("Unauthorized access attempt from IP: {}", client_ip);
                let response = req.into_response(
                    HttpResponse::Forbidden()
                        .content_type("application/json")
                        .body(format!("{{\"error\":\"Access denied. Your IP {} is not in the authorized hosts list.\"}}", client_ip))
                );
                return Box::pin(ready(Ok(response.map_into_right_body())));
            }
        }
        
        // Process the request
        let fut = self.service.call(req);
        
        // Add CORS headers if enabled
        Box::pin(async move {
            let mut res = fut.await?;
            
            if config.cors_enabled {
                // If CORS is enabled, add permissive CORS headers
                res.headers_mut().insert(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    header::HeaderValue::from_static("*"),
                );
                res.headers_mut().insert(
                    header::ACCESS_CONTROL_ALLOW_METHODS,
                    header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
                );
                res.headers_mut().insert(
                    header::ACCESS_CONTROL_ALLOW_HEADERS,
                    header::HeaderValue::from_static("Content-Type, Authorization, Accept"),
                );
                res.headers_mut().insert(
                    header::ACCESS_CONTROL_MAX_AGE,
                    header::HeaderValue::from_static("3600"),
                );
            }
            
            Ok(res.map_into_left_body())
        })
    }
}