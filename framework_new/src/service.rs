use std::task::{Context, Poll};

use futures::Future;

pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&self, req: Request) -> Self::Future;
}

impl<'a, S, Request> Service<Request> for &'a mut S
where
    S: Service<Request> + 'a,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        (**self).poll_ready(cx)
    }

    fn call(&self, request: Request) -> S::Future {
        (**self).call(request)
    }
}

impl<S, Request> Service<Request> for Box<S>
where
    S: Service<Request> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        (**self).poll_ready(cx)
    }

    fn call(&self, request: Request) -> S::Future {
        (**self).call(request)
    }
}
