plugins {
  alias(libs.plugins.kotlin.jvm)
  alias(libs.plugins.kotlin.serialization)
  application
}

application {
  mainClass = "socialteesnyc.UpdatePetsCommandKt"
}

kotlin {
  jvmToolchain(21)
}

dependencies {
  implementation(libs.apache.commons.text)
  implementation(libs.clikt)
  implementation(libs.kotlinx.coroutines)
  implementation(libs.kotlinx.serialization.json)
  implementation(libs.ktor.client.core)
  implementation(libs.ktor.client.okhttp)
  implementation(libs.ktor.client.contentNegotiation)
  implementation(libs.ktor.serialization.json)
  implementation(libs.okio)
  runtimeOnly(libs.slf4j.nop)
}
