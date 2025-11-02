use crate::api::model::{AuthorizationError, CallError};

pub trait Proxy<T> {
    async fn proxy_handler(&self, req: T) -> Result<(), CallError>;
}
