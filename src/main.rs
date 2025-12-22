mod api;
mod models;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use futures::future::join_all;

use api::AdoptapetApi;
use models::{Pet, PetsData};

/// Fetch pets from Adoptapet API and write to JSON file.
#[derive(Parser, Debug)]
#[command(name = "update-pets")]
#[command(about = "Fetch pets from Adoptapet API and write to JSON file")]
struct Args {
    /// Adoptapet API key
    #[arg(long, env = "ADOPTAPET_API_KEY")]
    api_key: String,

    /// Shelter ID
    #[arg(long, env = "SHELTER_ID", default_value = "83349")]
    shelter_id: String,

    /// Output JSON file path
    #[arg(short, long, default_value = "data/pets.json")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!(
        "Fetching pets from Adoptapet for shelter {}...",
        args.shelter_id
    );

    let api = AdoptapetApi::new(args.api_key);

    // Fetch all pets at the shelter
    let adoptapet_pets = api.get_pets_at_shelter(&args.shelter_id).await?;
    println!("Fetched {} pets from listing", adoptapet_pets.size());

    // Fetch details for each pet in parallel to get high-res images
    println!("Fetching pet details for high-res images...");
    let detail_futures: Vec<_> = adoptapet_pets
        .into_iter()
        .map(|pet| {
            let pet_id = pet.pet_id.clone();
            let api = &api;
            async move {
                let details = api.get_pet_details(&pet_id).await;
                (pet, details)
            }
        })
        .collect();

    let pets_with_details = join_all(detail_futures).await;

    // Convert to our output format
    let pets: Vec<Pet> = pets_with_details
        .into_iter()
        .map(|(pet, details)| pet.into_pet(details.as_ref()))
        .collect();

    // Count pets with photos
    let mut pets_without_photos = Vec::new();
    for pet in &pets {
        if pet.photo_url.is_none() {
            pets_without_photos.push(&pet.name);
        }
    }
    let pets_with_photos = pets.len() - pets_without_photos.len();

    for name in &pets_without_photos {
        println!("{} had no photo", name);
    }
    println!(
        "{} pets total, {} with photos",
        pets.len(),
        pets_with_photos
    );

    // Count by type
    let dogs = pets.iter().filter(|p| p.pet_type == "Dog").count();
    let cats = pets.iter().filter(|p| p.pet_type == "Cat").count();
    let other = pets.len() - dogs - cats;
    println!("Breakdown: {} dogs, {} cats, {} other", dogs, cats, other);

    // Create output data with timestamp
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let data = PetsData {
        pets,
        updated_at: timestamp,
    };

    // Serialize to pretty JSON
    let json_output = serde_json::to_string_pretty(&data)?;

    // Ensure output directory exists
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to file
    fs::write(&args.output, json_output)?;

    println!("Wrote {} pets to {:?}", data.pets.len(), args.output);

    Ok(())
}

// Extension trait to match Kotlin's size() method name
trait VecExt {
    fn size(&self) -> usize;
}

impl<T> VecExt for Vec<T> {
    fn size(&self) -> usize {
        self.len()
    }
}
