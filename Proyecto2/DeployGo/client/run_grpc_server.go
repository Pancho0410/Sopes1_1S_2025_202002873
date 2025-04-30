package client

import (
	"log"
	"net"

	"google.golang.org/grpc"
)

func StartGRPCServer() { //Server gRPC
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("Error al escuchar: %v", err)
	}

	grpcServer := grpc.NewServer()
	RegisterPublicadorServer(grpcServer, NewServer()) //Llama a publicador_grpc.pb.go, NewServer() devuelve una instancia de esta estructura

	log.Println("Servidor gRPC escuchando en puerto 50051")
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Error al servir: %v", err)
	}
}
