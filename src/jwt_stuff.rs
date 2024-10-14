use actix_web_grants::authorities::AuthDetails;
use futures::future::LocalBoxFuture;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    collections::HashSet,
    future::{ready, Ready},
    sync::Arc,
};
use utoipa::{
    openapi::{
        security::{Http, HttpAuthScheme, SecurityScheme},
        RefOr, Response,
    },
    Modify,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorBadRequest,
    http::header::{self, HeaderValue},
    Error, HttpMessage,
};

#[derive(Deserialize)]
struct TokenData {
    pub grants: HashSet<String>,
}

pub struct JwtGrantsMiddleware {
    decoding_key: Arc<DecodingKey>,
    validation: Arc<Validation>,
    #[allow(clippy::type_complexity)]
    err_handler: Option<Arc<dyn Fn(JwtDecodeErrors) -> Error + Send + Sync>>,
}

impl JwtGrantsMiddleware {
    pub fn new(decoding_key: DecodingKey, validation: Validation) -> Self {
        Self {
            decoding_key: Arc::new(decoding_key),
            validation: Arc::new(validation),
            err_handler: None,
        }
    }

    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(JwtDecodeErrors) -> Error + Send + Sync + 'static,
    {
        self.err_handler = Some(Arc::new(f));
        self
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for JwtGrantsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtGrantsService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtGrantsService {
            service,
            decoding_key: self.decoding_key.clone(),
            validation: self.validation.clone(),
            err_handler: self.err_handler.clone(),
        }))
    }
}

pub struct JwtGrantsService<S> {
    service: S,
    decoding_key: Arc<DecodingKey>,
    validation: Arc<Validation>,
    #[allow(clippy::type_complexity)]
    err_handler: Option<Arc<dyn Fn(JwtDecodeErrors) -> Error + Send + Sync>>,
}

pub enum JwtDecodeErrors {
    InvalidAuthHeader,
    InvalidJWTHeader,
    InvalidJWTToken(jsonwebtoken::errors::Error),
}

impl JwtDecodeErrors {
    pub fn to_error_string(&self) -> String {
        match self {
            JwtDecodeErrors::InvalidAuthHeader => {
                "Invalid authorization header - header contains invalid ASCII characters".into()
            }
            JwtDecodeErrors::InvalidJWTHeader => "Invalid authorization header - header need to have this format 'Bearer HEADER.PAYLOAD.SIGNATURE' where all three parts need to be base64 encoded and separated by a dot".into(),
            JwtDecodeErrors::InvalidJWTToken(e) => format!("Invalid JWT token - an error occurred when decoding token: {}", e),
        }
    }
}

fn decode_jwt<T: DeserializeOwned>(
    header_value: &HeaderValue,
    decoding_key: &DecodingKey,
    validation: &Validation,
) -> Result<T, JwtDecodeErrors> {
    let Ok(header_value) = header_value.to_str() else {
        return Err(JwtDecodeErrors::InvalidAuthHeader);
    };
    if !header_value.starts_with("Bearer ") {
        return Err(JwtDecodeErrors::InvalidJWTHeader);
    }
    match jsonwebtoken::decode::<T>(&header_value[7..], decoding_key, validation) {
        Ok(data) => Ok(data.claims),
        Err(e) => Err(JwtDecodeErrors::InvalidJWTToken(e)),
    }
}

impl<S, B> Service<ServiceRequest> for JwtGrantsService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header_value = req.headers().get(header::AUTHORIZATION).cloned();

        if let Some(auth_header_value) = auth_header_value {
            let claims = decode_jwt(&auth_header_value, &self.decoding_key, &self.validation);
            match claims {
                Ok(TokenData { grants }) => {
                    req.extensions_mut().insert(AuthDetails {
                        authorities: Arc::new(grants),
                    });
                }
                Err(e) => {
                    return Box::pin(ready(Ok(req
                        .error_response({
                            if let Some(err_handler) = self.err_handler.clone() {
                                (err_handler)(e)
                            } else {
                                ErrorBadRequest(e.to_error_string())
                            }
                        })
                        .map_into_right_body())));
                }
            }
        };

        let fut = self.service.call(req);
        Box::pin(async move { Ok(fut.await?.map_into_left_body()) })
    }
}

pub struct JwtGrantsAddon;

impl Modify for JwtGrantsAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt_grants",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );

        for (_, path) in &mut openapi.paths.paths {
            for (_, operation) in &mut path.operations {
                let Some(securities) = &mut operation.security else {
                    continue;
                };

                let contains_self = securities
                    .iter()
                    .any(|security| security.value.contains_key("jwt_grants"));

                if contains_self {
                    let responses = &mut operation.responses.responses;
                    responses.insert(
                        "400".into(),
                        RefOr::T(Response::new(
                            "Malformed authorization header or invalid jwt token",
                        )),
                    );
                    responses.insert(
                        "401".into(),
                        RefOr::T(Response::new("Insufficient permissions")),
                    );
                }
            }
        }
    }
}
