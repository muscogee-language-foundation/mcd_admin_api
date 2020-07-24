extern crate dotenv;

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web::HttpResponse;
use dotenv::dotenv;
use futures::future::{ok, Either, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::env;
use std::task::{Context, Poll};

use crate::models::Claims;

pub struct Authorization;

impl<S, B> Transform<S> for Authorization
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthorizationMiddleware { service })
    }
}
pub struct AuthorizationMiddleware<S> {
    service: S,
}

impl<S, B> Service for AuthorizationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let path = req.path();

        if path == "/login" {
            Either::Left(self.service.call(req))
        } else {
            dotenv().ok();
            let secret = env::var("SECRET").expect("SECRET should be set");
            let auth = req.headers().get("Authorization");
            match auth {
                Some(_) => {
                    let split: Vec<&str> =
                        auth.unwrap().to_str().unwrap().split("Bearer").collect();
                    let token = split[1].trim();
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::new(Algorithm::HS256),
                    ) {
                        Ok(_token) => Either::Left(self.service.call(req)),
                        Err(_e) => Either::Right(ok(req.into_response(
                            HttpResponse::InternalServerError().finish().into_body(),
                        ))),
                    }
                }
                None => Either::Right(ok(
                    req.into_response(HttpResponse::InternalServerError().finish().into_body())
                )),
            }
        }
    }
}
