from fastapi import FastAPI
from datetime import datetime
import socket

app = FastAPI()

@app.get("/")
def home():
    return {
        "status": "authorized",
        "message": "Stealth dashboard online",
        "time": str(datetime.now()),
        "hostname": socket.gethostname()
    }

@app.get("/health")
def health():
    return {
        "status": "healthy"
    }
