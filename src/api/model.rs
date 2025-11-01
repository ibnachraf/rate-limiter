use reqwest::Method;
use std::collections::HashMap;
use std::fmt;

pub enum Verb {
    GET,
    PATCH,
    POST,
    PUT,
    DELETE,
}

impl Verb {
    pub fn to_method(&self) -> Method {
        match self {
            Verb::GET => Method::GET,
            Verb::PATCH => Method::PATCH,
            Verb::POST => Method::POST,
            Verb::PUT => Method::PUT,
            Verb::DELETE => Method::DELETE,
        }
    }   
}

#[derive(Debug)]
pub enum AppError {
    Authorization(AuthorizationError),
    Technical(TechnicalError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authorization(e) => write!(f, "Authorization error"),
            Self::Technical(e) => write!(f, "Technical error:" ),
        }
    }
}

#[derive(Debug)]
pub enum AuthorizationError {
    TooManyQueries,
    IpHeaderMissing,
}

#[derive(Debug)]
pub enum TechnicalError {
    NotSupportedMethod,
}

pub enum QueryIp {
    Ip(Vec<u8>),
    Unknown,
}

#[derive(PartialEq, Eq, Hash)]
pub enum QueryParams {
    Ip,
}

impl QueryParams {
    pub fn to_header_name_str(&self) -> String {
        match self {
            QueryParams::Ip => "x-forwarded-for".to_string(),
        }
    }
}

pub struct UserQuery {
    pub header: HashMap<QueryParams, Vec<u8>>,
    pub verb: Verb,
    pub uri: String,
}
