package socialteesnyc

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter

interface AdoptapetApi {
  suspend fun getPetsAtShelter(shelterId: String): List<AdoptapetPet>
  suspend fun getPetDetails(petId: String): PetDetails?

  companion object {
    private const val BASE_URL = "https://api.adoptapet.com/search"

    fun create(client: HttpClient, apiKey: String): AdoptapetApi {
      return object : AdoptapetApi {
        override suspend fun getPetsAtShelter(shelterId: String): List<AdoptapetPet> {
          val response: AdoptapetResponse = client.get("$BASE_URL/pets_at_shelter") {
            parameter("key", apiKey)
            parameter("shelter_id", shelterId)
            parameter("start_number", 1)
            parameter("end_number", 500)
            parameter("output", "json")
          }.body()
          return response.pets
        }

        override suspend fun getPetDetails(petId: String): PetDetails? {
          return try {
            val response: PetDetailsResponse = client.get("$BASE_URL/pet_details") {
              parameter("key", apiKey)
              parameter("pet_id", petId)
              parameter("output", "json")
            }.body()
            response.pet
          } catch (e: Exception) {
            null
          }
        }
      }
    }
  }
}
