package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"

	"github.com/gorilla/mux"

	"database/sql"

	_ "github.com/lib/pq"
)

type Measurement struct {
	Lng      int    `json:"lng"`
	Lpg      int    `json:"lpg"`
	Co       int    `json:"co"`
	Datetime string `json:"datetime,omitempty"`
}

func storeMeasurement(msmt Measurement) Measurement {
	db, err := connectToDb()
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	defer db.Close()
	var result Measurement
	err = db.QueryRow(`
		INSERT INTO misc_measurements(lpg, lng, co)
        VALUES($1, $2, $3)
        RETURNING lng, lpg, co, datetime::TEXT
	`, msmt.Lng, msmt.Lpg, msmt.Co).
		Scan(&result.Lng, &result.Lpg, &result.Co, &result.Datetime)
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	return result
}

func listMeasurements() []Measurement {
	db, err := connectToDb()
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	defer db.Close()
	rows, err := db.Query(`
		SELECT lpg, lng, co, datetime::TEXT AS datetime
		FROM misc_measurements
		ORDER BY datetime DESC
	`)
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	var msmts []Measurement
	for rows.Next() {
		var msmt Measurement
		err = rows.Scan(&msmt.Lpg, &msmt.Lng, &msmt.Co, &msmt.Datetime)
		if err != nil {
			fmt.Println(err)
			panic(err)
		}
		msmts = append(msmts, msmt)
	}
	return msmts
}

func connectToDb() (*sql.DB, error) {
	host, isPresent := os.LookupEnv("AUTOCLEARSKIES_DB_HOST")
	if !isPresent {
		host = "localhost"
	}
	port, isPresent := os.LookupEnv("AUTOCLEARSKIES_DB_PORT")
	if !isPresent {
		port = "5432"
	}
	user, isPresent := os.LookupEnv("AUTOCLEARSKIES_DB_USER")
	if !isPresent {
		user = "postgres"
	}
	pass, isPresent := os.LookupEnv("AUTOCLEARSKIES_DB_PASS")
	if !isPresent {
		pass = "test"
	}
	connStr := fmt.Sprintf("postgres://%s:%s@%s:%s/autoclearskiesdb?sslmode=disable", user, pass, host, port)
	db, err := sql.Open("postgres", connStr)
	if err != nil {
		fmt.Println(err)
		panic(err)
	}
	return db, err
}

func main() {
	r := mux.NewRouter()

	r.HandleFunc("/measurements", func(w http.ResponseWriter, r *http.Request) {
		results := listMeasurements()
		json.NewEncoder(w).Encode(results)
	}).Methods("GET")
	r.HandleFunc("/measurements/record", func(w http.ResponseWriter, r *http.Request) {
		var msmt Measurement
		json.NewDecoder(r.Body).Decode(&msmt)
		result := storeMeasurement(msmt)
		json.NewEncoder(w).Encode(result)
	}).Methods("POST")

	http.ListenAndServe(":8080", r)
}
