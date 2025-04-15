from locust import HttpUser, task, between, constant
import random
import json
class ClimateUser(HttpUser):
    # wait_time = between(1, 5)
    wait_time = constant(5)

    @task
    def send_climate_data(self):
        weather_types = ["Rainy", "Cloudy", "Sunny"]
        countries = ["GT", "MX", "US", "FR", "JP"]
        descripctions = [ "El clima esta genial", "El clima esta horrible", "El clima esta normal", "No es de mis favoritos"] 

        payload = {
            "description": random.choice(descripctions),
            "country": random.choice(countries),
            "weather": random.choice(weather_types)
        }

        self.client.post("/input", json=payload)
