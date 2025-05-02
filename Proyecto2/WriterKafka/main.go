package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"time"

	"writerkafka/client"

	"github.com/segmentio/kafka-go"
	"google.golang.org/grpc"
)

const (
	// kafkaBroker = "localhost:9092" //Broker Kafka
	// kafkaBroker = "kafka:9092" //Broker Kafka
	kafkaBroker = "my-cluster-kafka-bootstrap:9092" //Broker Kafka
	kafkaTopic  = "tweet"                           // Nombre del topic de Kafka
)

type server struct {
	client.UnimplementedPublicadorServer // Aquí usamos el nombre correcto del servicio
}

// Implementa el método PublicarKafka de la interfaz del servidor
func (s *server) PublicarKafka(ctx context.Context, req *client.Mensaje) (*client.Respuesta, error) {
	// Imprimir el mensaje recibido
	fmt.Println("Mensaje Recibido:", req.GetContenido())

	// Publicar el mensaje en Kafka
	err := publishToKafka(req.GetContenido())
	if err != nil {
		return nil, fmt.Errorf("error al publicar en Kafka KafkaWriter: %v", err)
	}

	// Responder al cliente
	fmt.Println("Mensaje publicado con éxito")
	return &client.Respuesta{Estado: "Mensaje publicado con éxito"}, nil
}

// Función para publicar un mensaje en Kafka
func publishToKafka(message string) error {
	// Crear una nueva conexión a Kafka
	conn, err := kafka.DialLeader(context.Background(), "tcp", kafkaBroker, kafkaTopic, 0)
	if err != nil {
		fmt.Printf("error al conectar a Kafka: %s\n", err)
		return fmt.Errorf("error al conectar a Kafka: %v", err)
	}

	// Enviar el mensaje al topic Kafka

	convertBytes, err := json.Marshal(message)
	if err != nil {
		fmt.Printf("error al convertir mensaje a JSON: %s\n", err)
		return fmt.Errorf("error al convertir mensaje a JSON: %v", err)
	}
	conn.SetWriteDeadline(time.Now().Add(10 * time.Second)) // Establecer un tiempo de espera para la escritura
	// Escribir el mensaje en Kafka
	_, err = conn.WriteMessages(kafka.Message{
		Value: convertBytes,
		Key:   []byte(time.Now().Format("20060102150405")),
		// Offset: -1,
	})
	if err != nil {
		fmt.Printf("error al escribir mensaje en Kafka: %s\n", err)
		return fmt.Errorf("error al escribir mensaje en Kafka: %v", err)
	}

	// fmt.Printf("Se escribio bien")
	defer conn.Close()
	return nil
}

func main() {
	// Iniciar el servidor gRPC
	lis, err := net.Listen("tcp", ":50051") // Puerto donde el servidor gRPC estará escuchando
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	// Crear una nueva instancia del servidor gRPC
	s := grpc.NewServer()

	// Registrar el servidor en el gRPC server
	client.RegisterPublicadorServer(s, &server{}) // Usamos RegisterPublicadorServer del paquete 'client'

	// Iniciar el servidor
	fmt.Println("Servidor gRPC escuchando en puerto 50051...")
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
