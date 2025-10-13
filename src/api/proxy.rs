use std::collections::HashMap;
use axum::http::Request;
use crate::engine::rate_limiter::RateLimiter;

enum AuthorizationError {
    TooManyQueries,
    IpHeaderMissing,
}

enum QueryIp {
    Ip(Vec<u8>),
    Unknown,
}

#[derive(PartialEq, Eq, Hash)]
enum QueryParams {
    Ip,
}

struct UserQuery {
    header: HashMap<QueryParams, Vec<u8>>,
}

struct Proxy {
    rate_limiter: RateLimiter,
}

impl Proxy {
    async fn proxy_handler<T>(&self, req: Request<T>) -> Result<(), AuthorizationError> {
        let user_query: UserQuery = self.map(req);
        self.check_user_authorization(user_query)
    }

    fn check_user_authorization(&self, user_query: UserQuery) -> Result<(), AuthorizationError> {
        let ip_opt: Option<&Vec<u8>> = user_query.header.get(&QueryParams::Ip);
        if let Some(ip) = ip_opt {
            self.check_user_rate_limit(&String::from_utf8(ip.clone()).unwrap())
        } else {
            Err(AuthorizationError::IpHeaderMissing)
        }
    }

    fn map<T>(&self, req: Request<T>) -> UserQuery {
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
