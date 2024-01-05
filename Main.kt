package org.example

import io.ktor.serialization.kotlinx.json.*
import io.ktor.server.application.*
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.plugins.contentnegotiation.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.ktor.util.logging.*
import kotlinx.serialization.Serializable
import org.apache.commons.dbcp2.BasicDataSource
import org.jdbi.v3.core.HandleCallback
import org.jdbi.v3.core.Jdbi
import org.jdbi.v3.core.kotlin.KotlinPlugin
import org.jdbi.v3.core.statement.UnableToExecuteStatementException
import org.jdbi.v3.postgres.PostgresPlugin

internal val LOGGER = KtorSimpleLogger("com.example.ErrorLogger")

@Serializable
data class Measurement(val lng: Int, val lpg: Int, val co: Int, val datetime: String? = null)

fun connectToDb(): Jdbi {
    val host = System.getenv("AUTOCLEARSKIES_DB_HOST") ?: "localhost"
    val port = System.getenv("AUTOCLEARSKIES_DB_PORT") ?: "5432"
    val user = System.getenv("AUTOCLEARSKIES_DB_USER") ?: "postgres"
    val pass = System.getenv("AUTOCLEARSKIES_DB_PASS") ?: "blab"
    val url = "jdbc:postgresql://$host:$port/autoclearskiesdb?user=$user&password=$pass&ssl=false"
    val jdbi = Jdbi.create(BasicDataSource().apply { this.url = url })
        .installPlugin(PostgresPlugin())
        .installPlugin(KotlinPlugin())
    return jdbi!!
}

fun storeMeasurement(msmt: Measurement, jdbiConnection: Jdbi): Measurement {
    try {
        return jdbiConnection.withHandle(HandleCallback { handle ->
            try {
                val sql = """
                    INSERT INTO misc_measurements(lpg, lng, co)
                    VALUES(:lpg, :lng, :co) 
                    RETURNING lng, lpg, co, datetime::TEXT
                """.trimMargin()
                val result = handle.createUpdate(sql)
                    .bind("lpg", msmt.lpg)
                    .bind("lng", msmt.lng)
                    .bind("co", msmt.co)
                    .executeAndReturnGeneratedKeys("lng", "lpg", "co", "datetime")
                    .mapTo(Measurement::class.java)
                    .findOnly()
                return@HandleCallback result
            } catch (e: UnableToExecuteStatementException) {
                throw RuntimeException(e.message, e)
            }
        })!!
    } catch (e: Exception) {
        LOGGER.error("Error storing measurement")
        throw RuntimeException(e)
    }
}

fun listMeasurements(jdbiConnection: Jdbi): List<Measurement> {
    try {
        return jdbiConnection.withHandle(HandleCallback<List<Measurement>, Exception> { handle ->
            val sql = """
                SELECT id, lpg, lng, co, datetime::TEXT AS datetime
                FROM misc_measurements
                ORDER BY datetime DESC
            """.trimMargin()
            val results = handle.createQuery(sql)
                .mapTo(Measurement::class.java)
                .list()
            return@HandleCallback results
        })
    } catch (e: Exception) {
        LOGGER.error("Error listing measurements")
        throw RuntimeException(e)
    }
}

fun main() {
    val conn = connectToDb()

    embeddedServer(Netty, port = 3030) {
        routing {
            get ("/measurements") {
                val results = listMeasurements(conn)
                call.respond(results)
            }
            post ("/measurements/record") {
                val msmt = call.receive<Measurement>()
                val result = storeMeasurement(msmt, conn)
                call.respond(result)
            }
        }
        install(ContentNegotiation) {
            json()
        }
    }.start(wait = true)
}
