from pydantic import BaseModel # type: ignore
from typing import List

class LogContainer(BaseModel):
    pid: int
    container_id: str
    nombre: str
    memoria: float
    cpu: float
    # action: str
    timestamp: str

class MySystem(BaseModel):
    ram_total: int
    ram_libre: int
    ram_usada: int
    cpu_usado: int