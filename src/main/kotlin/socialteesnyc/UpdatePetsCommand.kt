package socialteesnyc

import com.github.ajalt.clikt.command.SuspendingCliktCommand
import com.github.ajalt.clikt.command.main
import com.github.ajalt.clikt.core.Context
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.required
import com.github.ajalt.clikt.parameters.types.path
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.HttpRequestRetry
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.serialization.kotlinx.json.json
import java.nio.file.Path
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.serialization.json.Json
import okio.FileSystem
import okio.Path.Companion.toOkioPath

class UpdatePetsCommand : SuspendingCliktCommand() {
  override fun help(context: Context): String {
    return "Fetch pets from Adoptapet API and write to JSON file"
  }

  private val apiKey by
  option("--api-key", envvar = "ADOPTAPET_API_KEY", help = "Adoptapet API key").required()

  private val shelterId by
  option("--shelter-id", envvar = "SHELTER_ID", help = "Shelter ID").default("83349")

  private val outputFile by
  option("-o", "--output", help = "Output JSON file path").path().default(Path.of("data/pets.json"))

  private val json = Json {
    prettyPrint = true
    ignoreUnknownKeys = true
  }

  private val client =
    HttpClient(OkHttp) {
      install(HttpRequestRetry) {
        retryOnExceptionOrServerErrors(maxRetries = 2)
        exponentialDelay()
      }
      install(ContentNegotiation) { json(json) }
    }

  override suspend fun run() {
    echo("Fetching pets from Adoptapet for shelter $shelterId...")

    val api = AdoptapetApi.create(client, apiKey)
    val adoptapetPets = api.getPetsAtShelter(shelterId)
    echo("Fetched ${adoptapetPets.size} pets from listing")

    // Fetch details for each pet in parallel to get high-res images
    echo("Fetching pet details for high-res images...")
    val petsWithDetails =
      coroutineScope {
        adoptapetPets.map { pet -> async { pet to api.getPetDetails(pet.petId) } }.awaitAll()
      }

    val pets = petsWithDetails.map { (pet, details) -> pet.toPet(details) }
    val petsWithPhotos = pets.count { pet -> (pet.photoUrl != null).also { if (!it) echo(pet.name + " had no photo") } }
    echo("${pets.size} pets total, $petsWithPhotos with photos")

    val dogs = pets.count { it.type == "Dog" }
    val cats = pets.count { it.type == "Cat" }
    val other = pets.size - dogs - cats
    echo("Breakdown: $dogs dogs, $cats cats, $other other")

    val timestamp = DateTimeFormatter.ISO_INSTANT.withZone(ZoneOffset.UTC).format(Instant.now())

    val data = PetsData(pets = pets, updatedAt = timestamp)
    val jsonOutput = json.encodeToString(PetsData.serializer(), data)

    val outputPath = outputFile.toOkioPath()
    outputPath.parent?.let { parent ->
      if (!FileSystem.SYSTEM.exists(parent)) {
        FileSystem.SYSTEM.createDirectories(parent)
      }
    }
    FileSystem.SYSTEM.write(outputPath) { writeUtf8(jsonOutput) }

    echo("Wrote ${pets.size} pets to $outputFile")
    client.close()
  }
}

suspend fun main(args: Array<String>) = UpdatePetsCommand().main(args)
