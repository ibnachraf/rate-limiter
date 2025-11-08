use openapi_model_generator::{generate_models, parse_openapi};
use std::fs;

fn main() {
    let open_api_file = "src/resources/openapi.yml";
    let openapi_spec = fs::read_to_string(open_api_file)
        .expect(format!("Failed to read file {}", open_api_file).as_str());
    let openapi: openapiv3::OpenAPI = serde_yaml::from_str(&openapi_spec).expect("Failed to parse openapi spec");

    // Generate models
let (models, requests, responses) = parse_openapi(&openapi)
    .expect("Failed to parse OpenAPI spec");
let generated_code = generate_models(&models, &requests, &responses).expect("Failed to generate models");

// creatre the output directory if it doesn't exist
fs::create_dir_all("src/generated").expect("Failed to create output directory");
// Write the generated code to a file
fs::write("src/generated/generated_models.rs", generated_code)
    .expect("Unable to write generated models to file");
}
