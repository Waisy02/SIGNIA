
import requests

class SigniaClient:
    def __init__(self, base_url: str):
        self.base_url = base_url

    def compile(self, payload: dict):
        r = requests.post(f"{self.base_url}/v1/compile", json=payload)
        r.raise_for_status()
        return r.json()

    def verify(self, payload: dict):
        r = requests.post(f"{self.base_url}/v1/verify", json=payload)
        r.raise_for_status()
        return r.json()
