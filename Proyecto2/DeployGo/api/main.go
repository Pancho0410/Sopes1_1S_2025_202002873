package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"deploygo/client" // Ajusta si tu módulo tiene otro nombre

	"google.golang.org/grpc"
)

type ClimateData struct {
	Description string `json:"description"`
	Country     string `json:"country"`
	Weather     string `json:"weather"`
}

func handler(w http.ResponseWriter, r *http.Request) {
	var data ClimateData
	if err := json.NewDecoder(r.Body).Decode(&data); err != nil {
		http.Error(w, "Error en el formato JSON", http.StatusBadRequest)
		return
	}

	log.Printf("Recibido: %+v", data)

	// conexion al cliente gRPC que esta esuchando en el puerto 50051
	grcpServer := "localhost:50051"
	// grcpServer := "grpc-server:50051"
	conn, err := grpc.Dial(grcpServer, grpc.WithInsecure(), grpc.WithBlock(), grpc.WithTimeout(2*time.Second)) //Cliente gRPC
	if err != nil {
		http.Error(w, "No se pudo conectar al servidor gRPC", http.StatusInternalServerError)
		log.Println("❌ Error gRPC:", err)
		return
	}
	defer conn.Close()

	cliente := client.NewPublicadorClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	msg := &client.Mensaje{
		Contenido: data.Description + " - " + data.Country + " - " + data.Weather, //Llama a Mensaje en publisher.pb.go
	}

	// Llamadas gRPC
	if _, err := cliente.PublicarRabbit(ctx, msg); err != nil {
		log.Println("❌ Error al publicar en Rabbit:", err)
	}
	if _, err := cliente.PublicarKafka(ctx, msg); err != nil {
		log.Println("❌ Error al publicar en Kafka:", err)
	}

	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Datos enviados al servidor gRPC."))
}

func main() {
	http.HandleFunc("/input", handler)
	log.Println("API REST GO escuchando en http://localhost:8081/input")
	// log.Fatal(http.ListenAndServe(":8081", nil)) //Local host
	log.Fatal(http.ListenAndServe("0.0.0.0:8081", nil))
}
