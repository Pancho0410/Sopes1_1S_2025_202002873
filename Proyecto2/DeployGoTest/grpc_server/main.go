package main

import (
	"log"
	"net"

	"deploygo/client"

	"google.golang.org/grpc"
)

func main() { //SERVIDOR GRCP
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("❌ Error al escuchar: %v", err)
	}

	grpcServer := grpc.NewServer()
	client.RegisterPublicadorServer(grpcServer, client.NewServer()) //aca cambia a las funciones de server.go en client

	log.Println("Servidor gRPC escuchando en puerto 50051...")
	if err := grpcServer.Serve(lis); err != nil { //Levanta el servidor gRPC
		log.Fatalf("❌ Error al servir: %v", err)
	}
}
