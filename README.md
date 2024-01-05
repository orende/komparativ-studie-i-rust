# Komparativ studie i Rust

Detta repo innehåller likvärdig kod i fyra olika språk som skapar en HTTP
server med två routes (GET /measurements och POST /measurements/record) samt
en postgresql-klient. 

För att skapa en databas som matchar SQL-frågorna kan du använda följande
SQL-skript:

```sql
-- The below script assumes you have created a database named autoclearskiesdb and
-- have switched to it before executing the sql statements.

CREATE ROLE autoclearskies_user WITH PASSWORD 'blab' LOGIN;
GRANT CONNECT ON DATABASE autoclearskiesdb TO autoclearskies_user;

CREATE TABLE misc_measurements(
id SERIAL PRIMARY KEY, 
lpg INT, 
lng INT, 
co INT, 
datetime TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW());

GRANT SELECT,INSERT,UPDATE ON misc_measurements TO autoclearskies_user;
```

Koden kommer att försöka hitta konfiguration för databasen i dina lokala environment variables med följande namn:

* AUTOCLEARSKIES_DB_HOST
* AUTOCLEARSKIES_DB_PORT
* AUTOCLEARSKIES_DB_USER
* AUTOCLEARSKIES_DB_PASS

Om dem ej är definierade kommer hårdkodade defaultvärden att användas istället.
