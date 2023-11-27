use std::future::{ready, Ready};
use std::path::Path;
use std::rc::Rc;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, HttpResponse};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use crate::RepoDir;

pub struct UnwrapRepo;

impl<S, B> Transform<S, ServiceRequest> for UnwrapRepo
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = UnwrapRepoMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UnwrapRepoMiddleware {
            service: Rc::new(service)
        }))
    }
}

pub struct UnwrapRepoMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for UnwrapRepoMiddleware<S>
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
        let service = Rc::clone(&self.service);

        let dir = Path::new(req.app_data::<RepoDir>().unwrap().path).join(req.match_info().query("repo"));

        Box::pin(async move {
            let repo = match gix::open(dir) {
                Ok(repo) => repo,
                Err(err) => {
                    return Ok(req.into_response(
                        HttpResponse::NotFound().body(err.to_string())
                    ).map_into_right_body());
                }
            };

            println!("{:?}", repo);
            println!("{:?}", req);

            req.extensions_mut().insert();
            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}