use crate::api::model::{AuthorizationError, QueryIp, QueryParams, UserQuery};
use crate::api::proxy::Proxy;
use crate::engine::rate_limiter::RateLimiter;
use axum::http::Request;
use std::collections::HashMap;

struct HttpProxy {
    rate_limiter: RateLimiter,
}

impl<T> Proxy<&Request<T>> for HttpProxy {
    fn proxy_handler(&self, req: &Request<T>) -> Result<(), AuthorizationError> {
        let user_query: UserQuery = self.map(req);
        self.check_user_authorization(user_query)
    }
}

impl HttpProxy {
    fn check_user_authorization(&self, user_query: UserQuery) -> Result<(), AuthorizationError> {
        let ip_opt: Option<&Vec<u8>> = user_query.header.get(&QueryParams::Ip);
        if let Some(ip) = ip_opt {
            self.check_user_rate_limit(&String::from_utf8(ip.clone()).unwrap())
        } else {
            Err(AuthorizationError::IpHeaderMissing)
        }
    }

    fn map<T>(&self, req: &Request<T>) -> UserQuery {
        let e: QueryIp = req
            .headers()
            .get("X-forwarded-for")
            .map(|header| QueryIp::Ip(header.as_bytes().to_vec()))
            .unwrap_or(QueryIp::Unknown);

        let mut query_param_map: HashMap<QueryParams, Vec<u8>> = HashMap::new();

        if let QueryIp::Ip(ip) = e {
            query_param_map.insert(QueryParams::Ip, ip);
        }

        UserQuery {
            header: query_param_map,
        }
    }

    fn check_user_rate_limit(&self, ip: &String) -> Result<(), AuthorizationError> {
        if self.rate_limiter.is_authorized(ip) {
            Ok(())
        } else {
            Err(AuthorizationError::TooManyQueries)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::proxy::Proxy;
    use crate::engine::rate_limiter::RateLimiter;
    use axum::http::Request;

    use crate::api::http_proxy::HttpProxy;

    fn test_simple_proxy_handler() {
        let mut rate_limiter = RateLimiter::new(2, 1);
        let http_proxy = HttpProxy { rate_limiter };

        let request: Request<()> = Request::get("www.google.com")
            .header("x-forwarded-for", "1.0.0.0")
            .body(())
            .unwrap();

        assert!(http_proxy.proxy_handler(&request).is_ok());
        assert!(http_proxy.proxy_handler(&request).is_ok());
        assert!(http_proxy.proxy_handler(&request).is_err());
    }
}
