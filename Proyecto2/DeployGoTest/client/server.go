package client

import (
	"context"
	"log"
)

type servidor struct {
	UnimplementedPublicadorServer
}

func NewServer() *servidor {
	return &servidor{}
}

func (s *servidor) PublicarRabbit(ctx context.Context, msg *Mensaje) (*Respuesta, error) {
	log.Println("PublicarRabbit:", msg.Contenido)
	return &Respuesta{Estado: "Mensaje enviado a RabbitMQ (simulado)"}, nil
}

func (s *servidor) PublicarKafka(ctx context.Context, msg *Mensaje) (*Respuesta, error) {
	log.Println("PublicarKafka:", msg.Contenido)
	return &Respuesta{Estado: "Mensaje enviado a Kafka (simulado falta configurar el servicio)"}, nil
}
