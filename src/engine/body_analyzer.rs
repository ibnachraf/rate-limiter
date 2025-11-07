use axum::{
    body::{Body, to_bytes},
    extract::Request,
};

use crate::engine::model::BodyAnalyzerError;

pub struct BodyAnalyzer {
    limit_body_size: usize, // this value must be calculated in advance by a ML model
}

impl BodyAnalyzer {
    pub async fn analyze_body_size<T>(
        &self,
        req: Request<Body>,
    ) -> Result<usize, BodyAnalyzerError> {
        let (parts, body) = req.into_parts();
        match parts
            .headers
            .iter()
            .find(|header| header.0.as_str().eq("Content-Length"))
        {
            Some(content_length_header) => {
                let size = content_length_header
                    .1
                    .to_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                if size > self.limit_body_size as u64 {
                    return Err(BodyAnalyzerError::BodySizeExceeded);
                }
            }
            None => println!("Content-Length header not found"),
        }
        // must add equart type
        match to_bytes(body, self.limit_body_size).await {
            Ok(bytes) => {
                return Ok(bytes.len());
            }
            Err(_err) => {
                return Err(BodyAnalyzerError::BodySizeExceeded);
            }
        }
    }

    fn body_variation() -> () {
        // it must detect req/res body evolution
        // developers can add or remove bodies, so the analyzer must be intelligent to detect it
        // how?: 1 - ML model that detect body changes over time
        //       2 - take all the body parts/fields create a structure automatically and test against it
    }

    fn synchronize_body() -> () {
        // the backend calls the software and gets a key
        // using an algorithm 
    }
}
