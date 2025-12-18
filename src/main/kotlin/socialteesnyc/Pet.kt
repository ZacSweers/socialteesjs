package socialteesnyc

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import org.apache.commons.text.StringEscapeUtils

/** Response from the Adoptapet pets_at_shelter endpoint. */
@Serializable
data class AdoptapetResponse(
  val pets: List<AdoptapetPet> = emptyList(),
)

/** Pet data from the Adoptapet API (pets_at_shelter endpoint). */
@Serializable
data class AdoptapetPet(
  @SerialName("pet_id") val petId: String,
  @SerialName("pet_name") val petName: String,
  val species: String? = null,
  @SerialName("primary_breed") val primaryBreed: String? = null,
  @SerialName("secondary_breed") val secondaryBreed: String? = null,
  val age: String? = null,
  val sex: String? = null,
  val size: String? = null,
  @SerialName("large_results_photo_url") val photoUrl: String? = null,
)

/** Response from the Adoptapet pet_details endpoint. */
@Serializable
data class PetDetailsResponse(
  val pet: PetDetails? = null,
)

/** Detailed pet data from pet_details endpoint. */
@Serializable
data class PetDetails(
  @SerialName("pet_id") val petId: Long,
  @SerialName("pet_name") val petName: String,
  @SerialName("pet_details_url") val petDetailsUrl: String? = null,
  val description: String? = null,
  val images: List<PetImage> = emptyList(),
)

/** Image data from pet_details. */
@Serializable
data class PetImage(
  @SerialName("original_url") val originalUrl: String? = null,
)

/** Simplified pet model for output JSON consumed by the website. */
@Serializable
data class Pet(
  val id: String,
  val name: String,
  val type: String,
  val breed: String?,
  val age: String?,
  val sex: String?,
  val size: String?,
  val url: String,
  val photoUrl: String?,
  val description: String?,
  @SerialName("short_description") val shortDescription: String?,
)

/** Wrapper for the output JSON. */
@Serializable
data class PetsData(
  val pets: List<Pet>,
  val updatedAt: String,
)

/**
 * Extract the Cloudinary image ID from the original_url and build a high-res URL.
 * Input: https://media.adoptapet.com/image/upload/.../1268757503
 * Output: https://media.adoptapet.com/image/upload/c_fill,w_800,h_600,g_auto/f_auto,q_auto/1268757503
 */
fun extractHighResImageUrl(originalUrl: String?): String? {
  if (originalUrl.isNullOrBlank()) return null
  // Extract the image ID (last path segment, no extension)
  val imageId = originalUrl.substringAfterLast("/").substringBefore(".")
  if (imageId.isBlank()) return originalUrl
  // Return optimized URL with reasonable size (800x600, 4:3 aspect)
  return "https://media.adoptapet.com/image/upload/c_fill,w_800,h_600,g_auto/f_auto,q_auto/$imageId"
}

/** Convert Adoptapet pet + details to our simplified model. */
fun AdoptapetPet.toPet(details: PetDetails?): Pet {
  // Get high-res photo from details, fall back to low-res from listing
  // Filter out "/null" placeholder URLs
  val highResPhoto = details?.images?.firstOrNull()?.originalUrl?.let { extractHighResImageUrl(it) }
  val finalPhotoUrl = (highResPhoto ?: photoUrl)?.takeUnless { it.contains("/null") }

  val type = when (species?.lowercase()) {
    "dog" -> "Dog"
    "cat" -> "Cat"
    else -> species ?: "Other"
  }

  val breed = listOfNotNull(primaryBreed, secondaryBreed)
    .filter { it.isNotBlank() }
    .joinToString(" / ")
    .ifBlank { null }

  val sex = when (this.sex?.lowercase()) {
    "m" -> "Male"
    "f" -> "Female"
    else -> this.sex
  }

  // Capitalize age
  val capitalizedAge = age?.replaceFirstChar { it.uppercase() }

  // Clean up description HTML
  val cleanDescription = details?.description
    ?.let { StringEscapeUtils.unescapeHtml4(it) }  // Decode HTML entities
    ?.replace(Regex("<[^>]*>"), " ")  // Strip HTML tags
    ?.replace(Regex("##\\d+##"), "")  // Strip reference codes
    ?.replace(Regex("\\s+"), " ")     // Collapse whitespace
    ?.trim()
    ?.takeIf { it.isNotBlank() }

  // Extract short description: text before "Please email" or truncate to 200 chars
  val shortDescription = cleanDescription?.let { desc ->
    val emailIndex = desc.indexOf("Please email", ignoreCase = true)
    if (emailIndex > 0) {
      desc.substring(0, emailIndex).trim()
    } else if (desc.length > 200) {
      desc.substring(0, 200).trim() + "..."
    } else {
      desc
    }
  }

  return Pet(
    id = petId,
    name = petName,
    type = type,
    breed = breed,
    age = capitalizedAge,
    sex = sex,
    size = size,
    url = details?.petDetailsUrl ?: "https://www.adoptapet.com/pet/$petId",
    photoUrl = finalPhotoUrl,
    description = cleanDescription,
    shortDescription = shortDescription,
  )
}
