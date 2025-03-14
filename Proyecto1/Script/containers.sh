#!/bin/bash

SCRIPT_LOG="script_containers.log"

generate_random_name() {
    cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 10 | head -n 1
}

# Tipos de contenedores
CONTAINER_TYPES=("cpu" "ram" "io" "disk")

echo "---------------------------------------------------------------------" >> $SCRIPT_LOG
# Generar 10 contenedores
for i in {1..10}; do
    # Seleccionar un tipo de contenedor aleatorio
    RANDOM_TYPE=${CONTAINER_TYPES[$RANDOM % ${#CONTAINER_TYPES[@]}]}

    # Nombre aleatorio para el contenedor
    CONTAINER_NAME=$(generate_random_name)

    case $RANDOM_TYPE in
        "cpu")
            docker run -d --cpus="0.2" --memory="50m" --name $CONTAINER_NAME containerstack/alpine-stress stress --cpu 1
            ;;
        "ram")
            docker run -d --cpus="0.2" --memory="50m" --name $CONTAINER_NAME containerstack/alpine-stress stress --vm 1 --vm-bytes 50MB
            ;;
        "io")
            docker run -d --cpus="0.2" --memory="50m" --name $CONTAINER_NAME containerstack/alpine-stress stress --io 2
            ;;
        "disk")
            docker run -d --cpus="0.2" --memory="50m" --name $CONTAINER_NAME containerstack/alpine-stress stress --hdd 2
            ;;
    esac

    # Registrar las creaciones del container
    echo "$(date): Contenedor $CONTAINER_NAME de tipo $RANDOM_TYPE creado." >> $SCRIPT_LOG
done
