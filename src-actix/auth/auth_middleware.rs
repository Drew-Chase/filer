use crate::auth::auth_data::User;
use actix_web::dev::{forward_ready, Service};
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::Error;
use futures::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use crate::auth::auth_endpoint::TOKEN_COOKIE_KEY;

pub struct Authentication;

impl Authentication {
    pub fn new() -> Self {
        Authentication
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let headers = req.headers().clone();
            let connection_info = req.connection_info().clone();

            // Check for X-Authentication header
            if let Some(auth_header) = headers.get("X-Authentication") {
                if let Some(auth_name) = headers.get("X-Username") {
                    if let Ok(token) = auth_header.to_str() {
                        if let Ok(username) = auth_name.to_str() {
                            if let Ok(user_result) = User::get_by_username(username).await {
                                if let Some(user) = user_result {
                                    if let Some(ip_address) = connection_info.realip_remote_addr() {
                                        let host = connection_info.host().to_owned();
                                        if let Ok(is_valid) = user.authenticate_with_session_token(
                                            ip_address, &host, token,
                                        ) {
                                            if is_valid {
                                                return service.call(req).await;
                                            }
                                        }
                                    }
                                }
                                return Err(ErrorUnauthorized(
                                    "Missing or invalid authentication token",
                                ));
                            }
                        }
                    }
                }
            }

            if let Some(token_cookie) = &req.cookie(TOKEN_COOKIE_KEY) {
                let token = token_cookie.value().to_string();
                let ip = connection_info.peer_addr().unwrap_or("unknown");
                let host = connection_info.host().to_string();

                if let Ok(users) = User::list().await {
                    for user in users {
                        if let Ok(is_valid) =
                            user.authenticate_with_session_token(ip, &host, &token)
                        {
                            if is_valid {
                                return service.call(req).await;
                            }
                        }
                    }
                }
            }

            Err(ErrorUnauthorized("Missing or invalid authentication token"))
        })
    }
}
