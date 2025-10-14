use std::collections::HashMap;

pub enum AuthorizationError {
    TooManyQueries,
    IpHeaderMissing,
}

pub enum QueryIp {
    Ip(Vec<u8>),
    Unknown,
}

#[derive(PartialEq, Eq, Hash)]
pub enum QueryParams {
    Ip,
}

pub struct UserQuery {
    pub header: HashMap<QueryParams, Vec<u8>>,
}