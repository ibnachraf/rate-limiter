use crate::api::model::{AuthorizationError, QueryIp, QueryParams, UserQuery, Verb};
use crate::api::proxy::Proxy;
use crate::engine::rate_limiter::RateLimiter;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Uri};
use reqwest::Client;
use std::collections::HashMap;

struct HttpProxy {
    rate_limiter: RateLimiter,
    client: reqwest::Client,
}

impl<T> Proxy<&Request<T>> for HttpProxy {
    async fn proxy_handler(&self, req: &Request<T>) -> Result<(), AuthorizationError> {
        let user_query: UserQuery = self.map(req);
        match self.check_user_authorization(&user_query) {
            Ok(_) => (),
            Err(err) => return Err(err),
        };

        let proxy_reqwest = self
            .client
            .request(user_query.verb.to_method(), &user_query.uri)
            //must manage the body here
            .headers(self.into_header_map(&user_query))
            .build();

        let request = proxy_reqwest.expect("Oups! building request failed");

        self.client.execute(request).await;

        Ok(())
    }
}

impl HttpProxy {
    fn into_header_map(&self, user_query: &UserQuery) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        for (k, v) in &user_query.header {
            let header_name = HeaderName::from_bytes(k.to_header_name_str().as_bytes())
                .expect("oups! in header name");
            let header_value =
                HeaderValue::from_bytes(v.as_slice()).expect("Oups! in header value");
            header_map.append(header_name, header_value);
        }
        return header_map;
    }

    fn check_user_authorization(&self, user_query: &UserQuery) -> Result<(), AuthorizationError> {
        let ip_opt: Option<&Vec<u8>> = user_query.header.get(&QueryParams::Ip);
        if let Some(ip) = ip_opt {
            return self.check_user_rate_limit(&String::from_utf8(ip.clone()).unwrap());
        } else {
            Err(AuthorizationError::IpHeaderMissing)
        }
    }

    fn map<T>(&self, req: &Request<T>) -> UserQuery {
        let query_param_map: HashMap<QueryParams, Vec<u8>> = self.extract_headers(req);
        let verb: Verb = self.extract_verb(req);
        let uri: String = self.extract_uri(req);

        UserQuery {
            header: query_param_map,
            verb: verb,
            uri: uri,
        }
    }

    fn extract_headers<T>(&self, req: &Request<T>) -> HashMap<QueryParams, Vec<u8>> {
        let mut query_param_map: HashMap<QueryParams, Vec<u8>> = HashMap::new();

        //Ip
        let ip: QueryIp = req
            .headers()
            .get(QueryParams::Ip.to_header_name_str())
            .map(|header| QueryIp::Ip(header.as_bytes().to_vec()))
            .unwrap_or(QueryIp::Unknown);

        if let QueryIp::Ip(ip) = ip {
            query_param_map.insert(QueryParams::Ip, ip);
        }

        query_param_map
    }

    fn extract_verb<T>(&self, req: &Request<T>) -> Verb {
        match req.method().as_str() {
            "GET" => Verb::GET,
            "POST" => Verb::POST,
            "PATCH" => Verb::PATCH,
            "PUT" => Verb::PUT,
            "DELETE" => Verb::DELETE,
            verb => panic!("Verb not supported {}", verb),
        }
    }

    fn extract_uri<T>(&self, req: &Request<T>) -> String {
        req.uri().to_owned().to_string()
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
    use axum::body::Body;
    use axum::http::Request;

    use crate::api::http_proxy::HttpProxy;

    #[tokio::test]
    async fn test_simple_proxy_handler() {
        let mut rate_limiter = RateLimiter::new(2, 1);
        let client = reqwest::Client::new();
        let http_proxy = HttpProxy {
            rate_limiter,
            client,
        };

        let body_json = "{\"key\": \"value\"}".to_string();

        let body = Body::new(body_json);

        let request: Request<Body> = Request::get("https://www.google.com")
            .header("x-forwarded-for", "1.0.0.0")
            .body(body)
            .unwrap();

        assert!(http_proxy.proxy_handler(&request).await.is_ok());
        assert!(http_proxy.proxy_handler(&request).await.is_ok());
        assert!(http_proxy.proxy_handler(&request).await.is_err());
    }
}
