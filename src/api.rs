use anyhow::Result;
use futures::future::join_all;
use reqwest::Client;

use crate::models::{
    build_cloudinary_info_url, build_cloudinary_original_url, AdoptapetPet, AdoptapetResponse,
    CloudinaryInfoResponse, PetDetails, PetDetailsResponse, PhotoMetadata,
};

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

    /// Fetch image metadata from Cloudinary using fl_getinfo.
    /// Returns PhotoMetadata with original dimensions and aspect ratio.
    pub async fn get_image_metadata(&self, original_url: &str) -> Option<PhotoMetadata> {
        let info_url = build_cloudinary_info_url(original_url)?;
        let original_url = build_cloudinary_original_url(original_url)?;

        let response: CloudinaryInfoResponse = self
            .client
            .get(&info_url)
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        let width = response.input.width;
        let height = response.input.height;
        let aspect_ratio = width as f32 / height as f32;

        Some(PhotoMetadata {
            original_url,
            width,
            height,
            aspect_ratio,
        })
    }

    /// Fetch image metadata for multiple URLs in parallel.
    /// Returns a Vec of PhotoMetadata for images that were successfully fetched.
    pub async fn get_all_image_metadata(&self, original_urls: Vec<&str>) -> Vec<PhotoMetadata> {
        let futures: Vec<_> = original_urls
            .into_iter()
            .map(|url| self.get_image_metadata(url))
            .collect();

        join_all(futures).await.into_iter().flatten().collect()
    }
}
