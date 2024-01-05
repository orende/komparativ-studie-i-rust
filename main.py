import json
import os
from dataclasses import dataclass, is_dataclass, asdict
import traceback

from flask import Flask, request, abort
import psycopg2
from psycopg2.extras import DictCursor
from gevent.pywsgi import WSGIServer


app = Flask(__name__)


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if is_dataclass(o):
            return asdict(o)
        return super().default(o)


@dataclass
class Measurement:
    lpg: int
    lng: int
    co: int
    datetime: str


def storeMeasurementInDb(msmt):
    with connectToDb() as conn:
        with conn.cursor() as cursor:
            values = {
                'lpg': msmt['lpg'],
                'lng': msmt['lng'],
                'co': msmt['co']
            }
            cursor.execute("""
                INSERT INTO misc_measurements(lpg, lng, co)
                VALUES(%(lpg)s, %(lng)s, %(co)s) 
                RETURNING lng, lpg, co, datetime::TEXT
            """, values)

            result = cursor.fetchone()
            lpg = result[0]
            lng = result[1]
            co = result[2]
            datetime = result[3]

            return Measurement(lpg, lng, co, datetime)


def retrieveMeasurementsFromDb():
    with connectToDb() as conn:
        with conn.cursor() as cursor:
            cursor.execute("""
                SELECT id, lpg, lng, co, datetime::TEXT AS datetime
                FROM misc_measurements
                ORDER BY datetime DESC
            """)
            results = cursor.fetchall()
            output = []
            for result in results:
                output += [Measurement(result['lpg'], result['lng'], result['co'], result['datetime'])]
            return output


def connectToDb():
    addr = os.getenv('AUTOCLEARSKIES_DB_HOST', '0.0.0.0')
    portnum = os.getenv('AUTOCLEARSKIES_DB_PORT', '5432')
    username = os.getenv('AUTOCLEARSKIES_DB_USER', 'postgres')
    passw = os.getenv('AUTOCLEARSKIES_DB_PASS', 'test')
    db_name = "autoclearskiesdb"
    conn = psycopg2.connect(database=db_name, host=addr,
                            user=username, password=passw, port=portnum,
                            cursor_factory=DictCursor)
    conn.autocommit = True
    return conn


@app.route("/measurements/record", methods=['POST'])
def recordMeasurement():
    try:
        result = storeMeasurementInDb(request.json)
        return json.dumps(result, cls=EnhancedJSONEncoder)
    except Exception:
        traceback.print_exc()
        abort(500)


@app.route("/measurements")
def listMeasurements():
    try:
        measurements = retrieveMeasurementsFromDb()
        return json.dumps(measurements, cls=EnhancedJSONEncoder)
    except Exception:
        traceback.print_exc()
        abort(500)


def main():
    http_server = WSGIServer(('', 3030), app)
    http_server.serve_forever()


if __name__ == '__main__':
    main()
