#!/bin/bash

OUTPUT_FILE="/tmp/docker_stats.json"

while true; do
  # Información del sistema
  TOTAL_RAM=$(free -m | awk '/Mem:/ {print $2}')
  FREE_RAM=$(free -m | awk '/Mem:/ {print $4}')
  USED_RAM=$(free -m | awk '/Mem:/ {print $3}')
  CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')

  # Obtener los datos de los contenedores
  CONTAINER_STATS=$(docker stats --no-stream --format "{{ json . }}" | jq -s '
    map({
      ID: .ID,
      Nombre: .Name,
      Memoria: (.MemPerc | rtrimstr("%") | tonumber),
      CPU: (.CPUPerc | rtrimstr("%") | tonumber),
      Disco: ( .BlockIO | split(" /") | .[0] | if . == "0B" then "0" else rtrimstr("kB") end | tonumber),
      IO: ( .NetIO | split(" /") | .[0] | if . == "0B" then "0" else rtrimstr("kB") end | tonumber)
    })')

  CONTAINERS_WITH_PIDS=()

  # Agregar PID de cada contenedor
  for container in $(echo "$CONTAINER_STATS" | jq -c '.[]'); do
    # Obtener el ID del contenedor
    ID=$(echo "$container" | jq -r '.ID')

    # Obtener el PID usando docker inspect
    PID=$(docker inspect --format '{{.State.Pid}}' "$ID")

    # Agregar el PID al objeto del contenedor
    CONTAINERS_WITH_PIDS+=("$(echo "$container" | jq --arg pid "$PID" '. + {PID: $pid}')")
  done

  # Combinar la información general del sistema con los datos de los contenedores con PID
  jq -n --argjson containers "$(echo "${CONTAINERS_WITH_PIDS[@]}" | jq -s '.')" \
    --arg total_ram "$TOTAL_RAM" \
    --arg free_ram "$FREE_RAM" \
    --arg used_ram "$USED_RAM" \
    --arg cpu_usage "$CPU_USAGE" \
    '{
      sistema: {
        ram_total: ($total_ram | tonumber),
        ram_libre: ($free_ram | tonumber),
        ram_usada: ($used_ram | tonumber),
        cpu_usado: ($cpu_usage | tonumber)
      },
      containers: $containers
    }' > "$OUTPUT_FILE"

  #informacion guardada"
  echo "Ok"

  sleep 5
done

