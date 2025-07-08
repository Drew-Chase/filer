use crate::middleware::cors::is_authorized_host;
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use log::warn;
use std::future::{Future, Ready, ready};
use std::pin::Pin;

pub struct AuthorizedHostsCheck;

impl<S, B> Transform<S, ServiceRequest> for AuthorizedHostsCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthorizedHostsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizedHostsMiddleware { service }))
    }
}

pub struct AuthorizedHostsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthorizedHostsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse<EitherBody<B>>, Error>>>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check if the request is from an authorized host
        if is_authorized_host(req.request()) {
            // If authorized, proceed with the request
            let fut = self.service.call(req);
            Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
        } else {
            // If not authorized, return a 403 Forbidden response
            let connection_info = req.connection_info().clone();
            let client_ip = connection_info.host();
            warn!("Unauthorized access attempt from IP: {}", client_ip);

            let response = req.into_response(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied. Your IP is not in the authorized hosts list."
            })));

            Box::pin(ready(Ok(response.map_into_right_body())))
        }
    }
}
