use htmd::HtmlToMarkdown;
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
    // Compatibility info
    pub good_with_cats: Option<u8>,
    pub good_with_dogs: Option<u8>,
    pub good_with_kids: Option<u8>,
    // Status info
    pub housetrained: Option<u8>,
    pub shots_current: Option<u8>,
    pub spayed_neutered: Option<u8>,
    pub special_needs: Option<u8>,
    pub declawed: Option<u8>,
    // Physical attributes
    pub color: Option<String>,
}

/// Image data from pet_details.
#[derive(Debug, Deserialize)]
pub struct PetImage {
    pub original_url: Option<String>,
}

/// Response from Cloudinary's fl_getinfo flag.
/// Example: https://media.adoptapet.com/image/upload/fl_getinfo/IMAGE_ID
#[derive(Debug, Deserialize)]
pub struct CloudinaryInfoResponse {
    pub input: CloudinaryAssetInfo,
}

/// Asset info from Cloudinary fl_getinfo response.
#[derive(Debug, Deserialize)]
pub struct CloudinaryAssetInfo {
    pub width: u32,
    pub height: u32,
}

/// Photo metadata with original dimensions and URLs.
#[derive(Debug, Serialize, Clone)]
pub struct PhotoMetadata {
    /// Original full-resolution image URL (from Cloudinary, no custom transformations)
    #[serde(rename = "originalUrl")]
    pub original_url: String,
    /// Width of original image in pixels
    pub width: u32,
    /// Height of original image in pixels
    pub height: u32,
    /// Aspect ratio of the original image (width / height)
    #[serde(rename = "aspectRatio")]
    pub aspect_ratio: f32,
}

/// A named attribute with display name. Only true attributes are included.
#[derive(Debug, Serialize)]
pub struct Attribute {
    pub key: String,
    pub display: String,
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
    /// All photos with metadata (dimensions, aspect ratio, URL)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub photos: Vec<PhotoMetadata>,
    /// Plain text description (HTML stripped)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Original HTML description (reference codes removed)
    #[serde(rename = "descriptionHtml", skip_serializing_if = "Option::is_none")]
    pub description_html: Option<String>,
    /// Markdown description (converted from HTML)
    #[serde(
        rename = "descriptionMarkdown",
        skip_serializing_if = "Option::is_none"
    )]
    pub description_markdown: Option<String>,
    #[serde(rename = "short_description", skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    /// Physical color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Compatibility and status attributes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
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

/// Extract the Cloudinary image ID from the original_url.
/// Input: https://media.adoptapet.com/image/upload/.../1268757503
/// Returns: 1268757503
pub fn extract_cloudinary_image_id(original_url: &str) -> Option<&str> {
    let url = original_url.trim();
    if url.is_empty() {
        return None;
    }

    // Extract the image ID (last path segment, no extension)
    let image_id = url.rsplit('/').next()?.split('.').next()?;

    if image_id.is_empty() {
        None
    } else {
        Some(image_id)
    }
}

/// Build a Cloudinary fl_getinfo URL to fetch image metadata.
/// Input: https://media.adoptapet.com/image/upload/.../1268757503
/// Output: https://media.adoptapet.com/image/upload/fl_getinfo/1268757503
pub fn build_cloudinary_info_url(original_url: &str) -> Option<String> {
    let image_id = extract_cloudinary_image_id(original_url)?;
    Some(format!(
        "https://media.adoptapet.com/image/upload/fl_getinfo/{}",
        image_id
    ))
}

/// Build the original Cloudinary URL (no transformations, just format optimization).
/// Input: https://media.adoptapet.com/image/upload/.../1268757503
/// Output: https://media.adoptapet.com/image/upload/f_auto,q_auto/1268757503
pub fn build_cloudinary_original_url(original_url: &str) -> Option<String> {
    let image_id = extract_cloudinary_image_id(original_url)?;
    Some(format!(
        "https://media.adoptapet.com/image/upload/f_auto,q_auto/{}",
        image_id
    ))
}

impl AdoptapetPet {
    /// Get all valid original image URLs from pet details.
    /// Filters out "/null" placeholder URLs.
    pub fn get_original_image_urls<'a>(&'a self, details: Option<&'a PetDetails>) -> Vec<&'a str> {
        details
            .map(|d| {
                d.images
                    .iter()
                    .filter_map(|img| img.original_url.as_deref())
                    .filter(|url| !url.contains("/null"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Convert Adoptapet pet + details + photo metadata to our simplified model.
    /// Consumes self to avoid cloning strings.
    pub fn into_pet(self, details: Option<&PetDetails>, photos: Vec<PhotoMetadata>) -> Pet {
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

        // Combine primary and secondary breed, excluding "Unknown Type" entries
        let breed = [&self.primary_breed, &self.secondary_breed]
            .iter()
            .filter_map(|b| b.as_ref())
            .filter(|b| !b.trim().is_empty() && !b.contains("Unknown Type"))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" / ");
        let breed = if breed.is_empty() { None } else { Some(breed) };

        // Convert sex abbreviation to full word, None if missing
        let sex = match self.sex.as_deref().map(|s| s.to_lowercase()).as_deref() {
            Some("m") => Some("Male".to_string()),
            Some("f") => Some("Female".to_string()),
            Some(other) if !other.is_empty() => Some(other.to_string()),
            _ => None,
        };

        // Capitalize age, None if missing
        let age = self.age.as_ref().map(|a| capitalize_first(a));

        // Size is None if missing
        let size = self.size.clone();

        // Process description in multiple formats
        let raw_description = details.and_then(|d| d.description.as_ref());
        let description = raw_description.map(|desc| clean_html_description(desc));
        let description_html = raw_description.map(|desc| sanitize_html_description(desc));
        let description_markdown = raw_description.and_then(|desc| html_to_markdown(desc));

        // Extract short description: text before "Please email" or truncate to 200 chars
        let short_description = description.as_ref().map(|desc| {
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

        // Build attributes and get color
        let attributes = build_attributes(details);
        let color = details.and_then(|d| d.color.clone());

        Pet {
            id: self.pet_id,
            name: self.pet_name,
            pet_type,
            breed,
            age,
            sex,
            size,
            url,
            photo_url: final_photo_url,
            photos,
            description,
            description_html,
            description_markdown,
            short_description,
            color,
            attributes,
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

/// Remove reference codes like ##123## from text.
fn strip_reference_codes(text: &str) -> String {
    let ref_code_re = Regex::new(r"##\d+##").unwrap();
    ref_code_re.replace_all(text, "").trim().to_string()
}

/// Sanitize HTML description: remove reference codes but keep HTML structure.
fn sanitize_html_description(html: &str) -> String {
    strip_reference_codes(html)
}

/// Convert HTML description to Markdown.
fn html_to_markdown(html: &str) -> Option<String> {
    let sanitized = strip_reference_codes(html);
    let converter = HtmlToMarkdown::new();
    converter.convert(&sanitized).ok().map(|s| {
        // Normalize to double newlines between paragraphs for conventional markdown spacing
        // First collapse any existing multiple newlines, then make all single newlines double
        let multi_newline_re = Regex::new(r"\n{2,}").unwrap();
        let normalized = multi_newline_re.replace_all(s.trim(), "\n");
        normalized.replace('\n', "\n\n")
    })
}

/// Clean HTML from description text (plain text output).
fn clean_html_description(html: &str) -> String {
    // Decode HTML entities
    let decoded = htmlescape::decode_html(html).unwrap_or_else(|_| html.to_string());

    // Strip HTML tags
    let html_tag_re = Regex::new(r"<[^>]*>").unwrap();
    let no_tags = html_tag_re.replace_all(&decoded, " ");

    // Strip reference codes like ##123##
    let no_refs = strip_reference_codes(&no_tags);

    // Collapse whitespace
    let whitespace_re = Regex::new(r"\s+").unwrap();
    let collapsed = whitespace_re.replace_all(&no_refs, " ");

    collapsed.trim().to_string()
}

/// Helper to convert API's 0/1 to boolean, treating 1 as true.
fn api_bool(value: Option<u8>) -> Option<bool> {
    value.map(|v| v == 1)
}

/// Build attributes list from pet details. Only includes true attributes.
fn build_attributes(details: Option<&PetDetails>) -> Vec<Attribute> {
    let Some(d) = details else {
        return Vec::new();
    };

    let mut attrs = Vec::new();

    if api_bool(d.good_with_cats) == Some(true) {
        attrs.push(Attribute {
            key: "good_with_cats".to_string(),
            display: "Good with cats".to_string(),
        });
    }
    if api_bool(d.good_with_dogs) == Some(true) {
        attrs.push(Attribute {
            key: "good_with_dogs".to_string(),
            display: "Good with dogs".to_string(),
        });
    }
    if api_bool(d.good_with_kids) == Some(true) {
        attrs.push(Attribute {
            key: "good_with_kids".to_string(),
            display: "Good with kids".to_string(),
        });
    }
    if api_bool(d.housetrained) == Some(true) {
        attrs.push(Attribute {
            key: "housetrained".to_string(),
            display: "Housetrained".to_string(),
        });
    }
    if api_bool(d.shots_current) == Some(true) {
        attrs.push(Attribute {
            key: "shots_current".to_string(),
            display: "Shots current".to_string(),
        });
    }
    if api_bool(d.spayed_neutered) == Some(true) {
        attrs.push(Attribute {
            key: "spayed_neutered".to_string(),
            display: "Spayed/Neutered".to_string(),
        });
    }
    if api_bool(d.special_needs) == Some(true) {
        attrs.push(Attribute {
            key: "special_needs".to_string(),
            display: "Special needs".to_string(),
        });
    }
    if api_bool(d.declawed) == Some(true) {
        attrs.push(Attribute {
            key: "declawed".to_string(),
            display: "Declawed".to_string(),
        });
    }

    attrs
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

    /// Test that the Pet struct serializes with all JSON keys expected by socialtees-custom.js.
    /// Keys are defined in the JS file between @api-keys-start and @api-keys-end markers.
    #[test]
    fn test_pet_json_keys_match_frontend_expectations() {
        // Read the JS file at compile time and parse the required keys
        const JS_SOURCE: &str = include_str!("../socialtees-custom.js");

        let start_marker = "@api-keys-start";
        let end_marker = "@api-keys-end";

        let start_idx = JS_SOURCE
            .find(start_marker)
            .expect("Missing @api-keys-start marker in socialtees-custom.js");
        let end_idx = JS_SOURCE
            .find(end_marker)
            .expect("Missing @api-keys-end marker in socialtees-custom.js");

        let keys_section = &JS_SOURCE[start_idx + start_marker.len()..end_idx];

        // Parse keys from lines like "* - keyName"
        let required_keys: Vec<&str> = keys_section
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim().trim_start_matches('*').trim();
                if trimmed.starts_with("- ") {
                    Some(trimmed.trim_start_matches("- ").trim())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !required_keys.is_empty(),
            "No API keys found between markers in socialtees-custom.js"
        );

        // Create a Pet with all fields populated
        let pet = Pet {
            id: "123".to_string(),
            name: "Buddy".to_string(),
            pet_type: "Dog".to_string(),
            breed: Some("Labrador".to_string()),
            age: Some("Adult".to_string()),
            sex: Some("Male".to_string()),
            size: Some("Large".to_string()),
            url: "https://example.com/pet/123".to_string(),
            photo_url: Some("https://example.com/photo.jpg".to_string()),
            photos: vec![],
            description: Some("A friendly dog".to_string()),
            description_html: Some("<p>A friendly dog</p>".to_string()),
            description_markdown: Some("A friendly dog".to_string()),
            short_description: Some("A friendly dog".to_string()),
            color: Some("Brown".to_string()),
            attributes: vec![],
        };

        let json = serde_json::to_value(&pet).expect("Failed to serialize Pet");
        let obj = json
            .as_object()
            .expect("Pet should serialize to JSON object");

        for key in &required_keys {
            assert!(
                obj.contains_key(*key),
                "Missing required JSON key '{}' expected by socialtees-custom.js. \
                 If you renamed or removed this key, update the @api-keys section in socialtees-custom.js",
                key
            );
        }
    }
}
