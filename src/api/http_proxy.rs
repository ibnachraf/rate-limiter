use crate::api::model::{
    AuthorizationError, CallError, DownstreamError, QueryIp, QueryParams, UserQuery, Verb,
};
use crate::api::proxy::Proxy;
use crate::engine::rate_limiter::RateLimiter;
use axum::body::Body;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, Uri};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Clone)]
pub struct HttpProxy {
    pub rate_limiter: RateLimiter,
    pub client: reqwest::Client,
    pub original_url: String,
}

impl<T> Proxy<Request<T>> for HttpProxy {
    async fn proxy_handler(&self, req: Request<T>) -> Result<(), CallError> {
        let user_query: UserQuery = self.map(&req);
        println!("User query: {:?}", user_query);

        if self.check_user_authorization(&user_query).is_err() {
            println!("Authorization error");
            return Err(CallError::Authorization(AuthorizationError::TooManyQueries));
        }

        let proxy_reqwest = self
            .client
            .request(user_query.verb.to_method(), &self.original_url)
            //must manage the body here
            .headers(self.into_header_map(&user_query))
            .build();
        // TODO: fix the request building, why use client?
        let request = proxy_reqwest.expect("Oups! building request failed");

        let proxy_res = self.client.execute(request).await;
        match proxy_res {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("Downstream error: {:?}", err);
                Err(CallError::Downstream(DownstreamError::DownstreamError {
                    response: Response::new(Body::from(format!("Downstream error: {}", err))),
                }))
            }
        }
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
            original_url: "https://www.google.com".to_string(),
        };

        assert!(
            http_proxy
                .proxy_handler(generate_request(create_body()))
                .await
                .is_ok()
        );
        assert!(
            http_proxy
                .proxy_handler(generate_request(create_body()))
                .await
                .is_ok()
        );
        assert!(
            http_proxy
                .proxy_handler(generate_request(create_body()))
                .await
                .is_err()
        );
    }

    fn create_body() -> Body {
        let body_json = "{\"key\": \"value\"}".to_string();
        Body::new(body_json)
    }
    
    fn generate_request(body: Body) -> Request<Body> {
        Request::get("https://www.google.com")
            .header("x-forwarded-for", "1.0.0.0")
            .body(body)
            .unwrap()
    }
}
