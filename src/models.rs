use regex::Regex;
use serde::{Deserialize, Serialize};

/// Response from the Adoptapet pets_at_shelter endpoint.
#[derive(Debug, Deserialize)]
pub struct AdoptapetResponse {
    #[serde(default)]
    pub pets: Vec<AdoptapetPet>,
}

/// Pet data from the Adoptapet API (pets_at_shelter endpoint).
#[derive(Debug, Deserialize, Clone)]
pub struct AdoptapetPet {
    pub pet_id: String,
    pub pet_name: String,
    pub species: Option<String>,
    pub primary_breed: Option<String>,
    pub secondary_breed: Option<String>,
    pub age: Option<String>,
    pub sex: Option<String>,
    pub size: Option<String>,
    pub large_results_photo_url: Option<String>,
}

/// Response from the Adoptapet pet_details endpoint.
#[derive(Debug, Deserialize)]
pub struct PetDetailsResponse {
    pub pet: Option<PetDetails>,
}

/// Detailed pet data from pet_details endpoint.
#[derive(Debug, Deserialize)]
pub struct PetDetails {
    pub pet_details_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub images: Vec<PetImage>,
}

/// Image data from pet_details.
#[derive(Debug, Deserialize)]
pub struct PetImage {
    pub original_url: Option<String>,
}

/// Simplified pet model for output JSON consumed by the website.
#[derive(Debug, Serialize)]
pub struct Pet {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<String>,
    pub sex: Option<String>,
    pub size: Option<String>,
    pub url: String,
    #[serde(rename = "photoUrl")]
    pub photo_url: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "short_description")]
    pub short_description: Option<String>,
}

/// Wrapper for the output JSON.
#[derive(Debug, Serialize)]
pub struct PetsData {
    pub pets: Vec<Pet>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Extract the Cloudinary image ID from the original_url and build a high-res URL.
/// Input: https://media.adoptapet.com/image/upload/.../1268757503
/// Output: https://media.adoptapet.com/image/upload/c_fill,w_800,h_600,g_auto/f_auto,q_auto/1268757503
pub fn extract_high_res_image_url(original_url: Option<&str>) -> Option<String> {
    let url = original_url?.trim();
    if url.is_empty() {
        return None;
    }

    // Extract the image ID (last path segment, no extension)
    let image_id = url.rsplit('/').next()?.split('.').next()?;

    if image_id.is_empty() {
        return Some(url.to_string());
    }

    // Return optimized URL with reasonable size (800x600, 4:3 aspect)
    Some(format!(
        "https://media.adoptapet.com/image/upload/c_fill,w_800,h_600,g_auto/f_auto,q_auto/{}",
        image_id
    ))
}

impl AdoptapetPet {
    /// Convert Adoptapet pet + details to our simplified model.
    /// Consumes self to avoid cloning strings.
    pub fn into_pet(self, details: Option<&PetDetails>) -> Pet {
        // Get high-res photo from details, fall back to low-res from listing
        // Filter out "/null" placeholder URLs
        let high_res_photo = details
            .and_then(|d| d.images.first())
            .and_then(|img| extract_high_res_image_url(img.original_url.as_deref()));

        let final_photo_url = high_res_photo
            .or(self.large_results_photo_url)
            .filter(|url| !url.contains("/null"));

        let pet_type = match self.species.as_deref().map(|s| s.to_lowercase()).as_deref() {
            Some("dog") => "Dog".to_string(),
            Some("cat") => "Cat".to_string(),
            Some(other) => capitalize_first(other),
            None => "Other".to_string(),
        };

        // Combine primary and secondary breed
        let breed = [&self.primary_breed, &self.secondary_breed]
            .iter()
            .filter_map(|b| b.as_ref())
            .filter(|b| !b.trim().is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" / ");
        let breed = if breed.is_empty() { None } else { Some(breed) };

        // Convert sex abbreviation to full word
        let sex = match self.sex.as_deref().map(|s| s.to_lowercase()).as_deref() {
            Some("m") => Some("Male".to_string()),
            Some("f") => Some("Female".to_string()),
            other => other.map(|s| s.to_string()),
        };

        // Capitalize age
        let age = self.age.as_ref().map(|a| capitalize_first(a));

        // Clean up description HTML
        let clean_description = details
            .and_then(|d| d.description.as_ref())
            .map(|desc| clean_html_description(desc));

        // Extract short description: text before "Please email" or truncate to 200 chars
        let short_description = clean_description.as_ref().map(|desc| {
            if let Some(idx) = desc.to_lowercase().find("please email") {
                if idx > 0 {
                    return desc[..idx].trim().to_string();
                }
            }
            if desc.len() > 200 {
                format!("{}...", desc[..200].trim())
            } else {
                desc.clone()
            }
        });

        let url = details
            .and_then(|d| d.pet_details_url.clone())
            .unwrap_or_else(|| format!("https://www.adoptapet.com/pet/{}", self.pet_id));

        Pet {
            id: self.pet_id,
            name: self.pet_name,
            pet_type,
            breed,
            age,
            sex,
            size: self.size,
            url,
            photo_url: final_photo_url,
            description: clean_description,
            short_description,
        }
    }
}

/// Capitalize the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Clean HTML from description text.
fn clean_html_description(html: &str) -> String {
    // Decode HTML entities
    let decoded = htmlescape::decode_html(html).unwrap_or_else(|_| html.to_string());

    // Strip HTML tags
    let html_tag_re = Regex::new(r"<[^>]*>").unwrap();
    let no_tags = html_tag_re.replace_all(&decoded, " ");

    // Strip reference codes like ##123##
    let ref_code_re = Regex::new(r"##\d+##").unwrap();
    let no_refs = ref_code_re.replace_all(&no_tags, "");

    // Collapse whitespace
    let whitespace_re = Regex::new(r"\s+").unwrap();
    let collapsed = whitespace_re.replace_all(&no_refs, " ");

    collapsed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_high_res_url() {
        let url = "https://media.adoptapet.com/image/upload/v123/1268757503";
        let result = extract_high_res_image_url(Some(url));
        assert_eq!(
            result,
            Some("https://media.adoptapet.com/image/upload/c_fill,w_800,h_600,g_auto/f_auto,q_auto/1268757503".to_string())
        );
    }

    #[test]
    fn test_extract_high_res_url_none() {
        assert_eq!(extract_high_res_image_url(None), None);
        assert_eq!(extract_high_res_image_url(Some("")), None);
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("adult"), "Adult");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("PUPPY"), "PUPPY");
    }
}
