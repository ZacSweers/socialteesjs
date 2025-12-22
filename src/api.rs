use anyhow::Result;
use reqwest::Client;

use crate::models::{AdoptapetPet, AdoptapetResponse, PetDetails, PetDetailsResponse};

const BASE_URL: &str = "https://api.adoptapet.com/search";

/// Client for the Adoptapet API.
pub struct AdoptapetApi {
    client: Client,
    api_key: String,
}

impl AdoptapetApi {
    /// Create a new API client.
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    /// Fetch all pets at a shelter.
    pub async fn get_pets_at_shelter(&self, shelter_id: &str) -> Result<Vec<AdoptapetPet>> {
        let response: AdoptapetResponse = self
            .client
            .get(format!("{}/pets_at_shelter", BASE_URL))
            .query(&[
                ("key", self.api_key.as_str()),
                ("shelter_id", shelter_id),
                ("start_number", "1"),
                ("end_number", "500"),
                ("output", "json"),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(response.pets)
    }

    /// Fetch details for a specific pet.
    pub async fn get_pet_details(&self, pet_id: &str) -> Option<PetDetails> {
        let response: PetDetailsResponse = self
            .client
            .get(format!("{}/pet_details", BASE_URL))
            .query(&[
                ("key", self.api_key.as_str()),
                ("pet_id", pet_id),
                ("output", "json"),
            ])
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        response.pet
    }
}
