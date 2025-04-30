package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"deploygo/client"

	"google.golang.org/grpc"
)

type ClimateData struct {
	Description string `json:"description"`
	Country     string `json:"country"`
	Weather     string `json:"weather"`
}

func handler(w http.ResponseWriter, r *http.Request) {
	var data ClimateData
	err := json.NewDecoder(r.Body).Decode(&data)
	if err != nil {
		http.Error(w, "Error en JSON", http.StatusBadRequest)
		return
	}

	log.Printf("✅ Recibido desde Rust: %+v", data)

	// conexion al cliente gRPC que esta esuchando en el puerto 50051
	conn, err := grpc.Dial("localhost:50051", grpc.WithInsecure(), grpc.WithBlock(), grpc.WithTimeout(2*time.Second)) //Cliente gRPC
	if err != nil {
		http.Error(w, "No se pudo conectar al servidor gRPC", http.StatusInternalServerError)
		return
	}
	defer conn.Close()

	cliente := client.NewPublicadorClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	msg := &client.Mensaje{Contenido: data.Description + " - " + data.Country + " - " + data.Weather} //Llama a Mensaje en publisher.pb.go

	_, err = cliente.PublicarRabbit(ctx, msg)
	if err != nil {
		log.Println("❌ Error al llamar a PublicarRabbit:", err)
	}

	_, err = cliente.PublicarKafka(ctx, msg)
	if err != nil {
		log.Println("❌ Error al llamar a PublicarKafka:", err)
	}

	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Datos recibidos y funciones gRPC simuladas."))
}

func main() {
	go client.StartGRPCServer() //Llama al run_grpc_server.go

	http.HandleFunc("/input", handler)
	log.Println("API Go escuchando en http://localhost:8081/input")
	// log.Fatal(http.ListenAndServe(":8081", nil)) //Local host
	log.Fatal(http.ListenAndServe("0.0.0.0:8081", nil))
}
