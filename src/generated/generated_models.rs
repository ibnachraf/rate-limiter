use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

/// Pet
#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
}


/// Error
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub code: i64,
    pub message: String,
}


/// CreatePetsRequest
#[derive(Debug, Serialize)]
pub struct CreatePetsRequest {
    pub content_type: String,
    pub body: Pet,
}

/// A paged array of pets
#[derive(Debug, Deserialize)]
pub struct ListPetsResponse200 {
    pub body: Pets,
}

/// Expected response to a valid request
#[derive(Debug, Deserialize)]
pub struct ShowPetByIdResponse200 {
    pub body: Pet,
}

