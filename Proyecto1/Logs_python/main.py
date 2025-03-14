from fastapi import FastAPI
import os
import json
from typing import List
from models.models import LogContainer, MySystem


app = FastAPI()

def load_json_file(file_path):
    with open(file_path, 'r') as file:
        return json.load(file)

@app.get("/")
def read_root():
    return {"Servidor": "OK"}

@app.post("/allLogs")
def get_logs(logs_cont: List[LogContainer]):
    logs_file = 'logs/logs.json'
    if os.path.exists(logs_file):
        
        # with open(logs_file, 'r') as file:
        #     existing_logs = json.load(file)
        os.remove(logs_file)
        existing_logs = []
    else:
        existing_logs = []
   
    new_logs = [log.dict() for log in logs_cont]
    existing_logs.extend(new_logs)
    with open(logs_file, 'w') as file:
        json.dump(existing_logs, file, indent=4)

    return {"received": True}


@app.post("/system")
def update_system(systemF: MySystem):
    sys_file = 'logs/logs_sys.json'
    with open(sys_file, 'w') as file:
        json.dump(systemF.dict(), file, indent=4)

    return {"received": True}


@app.get("/containers")
def graphs_all_logs():
    containers_data = load_json_file('logs/logs.json')

    logs_for_grafana = []
    for log in containers_data:
        logs_for_grafana.append({
            "time": log["timestamp"],  
            "container_id": log["container_id"], 
            "nombre": log["nombre"],
            "memoria": log["memoria"], 
            "cpu": log["cpu"]
        })
    return logs_for_grafana


@app.get("/dataSystem")
def graphs_system():
    system_info = load_json_file('logs/logs_sys.json')
    return system_info
