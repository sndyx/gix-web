use std::future::{ready, Ready};
use std::path::PathBuf;
use std::rc::Rc;

use actix_web::{Error, HttpMessage, HttpResponse, web};
use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use gix::Repository;
use futures_util::future::LocalBoxFuture;

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

        Box::pin(async move {
            let repo = match req.app_data::<web::Data<Repository>>() {
                Some(repo) => repo.get_ref().clone(),
                None => {
                    let path = req.app_data::<web::Data<PathBuf>>().unwrap().join(req.match_info().query("repo"));
                    match gix::open(path) {
                        Ok(repo) => repo,
                        Err(err) => {
                            return Ok(req.into_response(
                                HttpResponse::NotFound().body(err.to_string())
                            ).map_into_right_body());
                        }
                    }
                }
            };

            req.extensions_mut().insert(repo);
            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}