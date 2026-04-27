use std::time::Duration;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tonic::transport::Channel;
use tonic::Status;
use crate::circuit_breaker::CircuitBreaker;

/// 简单的重试策略
#[derive(Clone)]
pub struct SimpleRetryPolicy {
    max_retries: usize,
}

impl SimpleRetryPolicy {
    pub fn new(max_retries: usize) -> Self {
        Self { max_retries }
    }
}

impl<Req, Res, E> tower::retry::Policy<Req, Res, E> for SimpleRetryPolicy 
where 
    Req: Clone, 
    E: From<Status> + Into<Status> + Clone
{
    type Future = std::future::Ready<Self>;

    fn retry(&self, _req: &Req, result: Result<&Res, &E>) -> Option<Self::Future> {
        match result {
            Err(e) => {
                let status: Status = e.clone().into();
                if status.code() == tonic::Code::Unavailable && self.max_retries > 0 {
                    return Some(std::future::ready(SimpleRetryPolicy {
                        max_retries: self.max_retries - 1,
                    }));
                }
            }
            _ => {}
        }
        None
    }

    fn clone_request(&self, req: &Req) -> Option<Req> {
        Some(req.clone())
    }
}

/// 弹性网关构造器
pub struct ResilientClientBuilder {
    breaker: Arc<CircuitBreaker>,
}

impl ResilientClientBuilder {
    pub fn new() -> Self {
        Self {
            breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        }
    }

    pub fn build(&self, channel: Channel) -> impl tower::Service<http::Request<tonic::body::BoxBody>, Response = http::Response<tonic::body::BoxBody>, Error = tower::BoxError> + Clone {
        ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::from_secs(3)))
            .layer(ConcurrencyLimitLayer::new(100))
            // 注意：在高阶实现中，CircuitBreaker 会被封装为 Layer
            // 这里为了演示逻辑，可以在外层调用时 check breaker.allow_request()
            .service(channel)
    }
    
    pub fn get_breaker(&self) -> Arc<CircuitBreaker> {
        self.breaker.clone()
    }
}
