// this code must be generated
// it must map the struct corresponding to each route

use axum::{
    body::Body,
    http::{Request, method},
};
use base64::{Engine as _, alphabet, engine::general_purpose};

use crate::api::model::Verb;

#[derive(Debug)]
pub enum MapperError {
    UnknownRoute,
}
// example of generated structs
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
}

// ATTENTION: this may lead to an error if the body content is already consumed
pub fn can_map_route_to_model(path: &str, verb: &Verb, body: &[u8]) -> Result<bool, MapperError> {
    //base64(POST /pets) = cG9zdCAvcGV0cw==

    let a = general_purpose::STANDARD.encode(format!("{} {}", verb.as_ref().to_lowercase(), path));
    let b64 = a.as_str();
    match b64 {
        "cG9zdCAvcGV0cw==" => {
            // POST /pets
            // map to CreatePetsRequest
            Ok(serde_json::from_slice::<Pet>(body).is_ok())
        }
        _ => {
            // unknown route
            print!("unknown route: {} {}", verb.as_ref(), path);
            Err(MapperError::UnknownRoute)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_map_route_to_model() {
        let path = "/pets";
        let verb = Verb::POST;
        let body = r#"{
            "id": 1,
            "name": "doggie",
            "tag": "dog"
        }"#;
        let result = can_map_route_to_model(path, &verb, body.as_bytes());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
