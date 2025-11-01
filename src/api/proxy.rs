use crate::api::model::AuthorizationError;

pub trait Proxy<T> {
    async fn proxy_handler(&self, req: T) -> Result<(), AuthorizationError>;
}