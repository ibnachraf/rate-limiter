use std::fs;

use axum::{
    body::{Body, to_bytes},
    extract::{Request, path},
};

use crate::engine::model::BodyAnalyzerError;

pub struct BodyAnalyzer {
    limit_body_size: usize, // this value must be calculated in advance by a ML model
}

impl BodyAnalyzer {
    pub fn analyze_open_api_description() -> () {
        let open_api_file = "src/resources/openapi.yml";
        let openapi_spec = fs::read_to_string(open_api_file)
            .expect(format!("Failed to read file {}", open_api_file).as_str());
        let serde_yaml = serde_yaml::from_str::<serde_yaml::Value>(&openapi_spec)
            .expect("Failed to parse openapi spec");

        let paths = &serde_yaml["paths"];
        let paths_mapping = paths
            .as_mapping()
            .expect("Error reading openApi mapping keys");
        let open_api_routes = paths_mapping.keys().for_each(|path| {
            let path_str = path.as_str().expect("Error reading path string");
            let path_desc = paths_mapping.get(path).expect("Failed to get path value");
            let verbs = path_desc
                .as_mapping()
                .expect(format!("Error reading verbs for {}", path_str).as_str())
                .keys()
                .map(|verb| verb.as_str().expect("Error reading verb string"))
                .collect::<Vec<&str>>()
                .join(", ");


            println!("Path: {}", path_str);
            println!("Verbs: {}", verbs);

            let verbs = path_desc
                .as_mapping()
                .expect(format!("Error reading verbs for {}", path_str).as_str())
                .keys().filter(|verb| verb.as_str().expect("Error casting to string").eq("post"))
                .for_each(|verb| {
                    path_desc["post"]["requestBody"]["content"]
                        .as_mapping()
                        .expect("Error reading content mapping")
                        .keys()
                        .for_each(|content_type| {
                            let content_type_str = content_type
                                .as_str()
                                .expect("Error reading content type string");
                            let schema_ref = &path_desc["post"]["requestBody"]["content"]
                                [content_type_str]["schema"]["$ref"];
                            let schema_ref_str = schema_ref
                                .as_str()
                                .expect("Error reading schema ref string");
                            println!(
                                "For verb POST, content type: {}, schema ref: {}",
                                content_type_str, schema_ref_str
                            );

                            let struct_ref = schema_ref_str.split("/").last().expect("Error splitting schema ref");

                        });
                    ()
                }
            );
            ()
        });
    }

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

    // Use openAPI templates to analyze body
    // use the generated model, map it to the routes and check if the body matches the model
    // use yaml_serde to parse openAPI files
    fn synchronize_body() -> () {}
}

#[cfg(test)]
mod tests {
    use crate::engine::body_analyzer::BodyAnalyzer;

    #[test]
    fn test_yaml_parsing() {
        BodyAnalyzer::analyze_open_api_description();
    }
}
