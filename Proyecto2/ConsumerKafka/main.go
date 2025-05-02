package main

import (
	"context"
	"fmt"
	"log"
	"strconv"

	"github.com/google/uuid"
	"github.com/redis/go-redis/v9"
	"github.com/segmentio/kafka-go"
)

const (
	// kafkaBroker = "localhost:9092"
	// kafkaBroker = "kafka:9092"
	kafkaBroker = "my-cluster-kafka-bootstrap:9092"
	kafkaTopic  = "tweet"
	// redisAddr  = "localhost:6379"
	redisAddr = "redis-service:6379"
	redisKey  = "climate_data"
)

var ctx = context.Background()

func main() {
	// Conexión a Redis
	rdb := redis.NewClient(&redis.Options{
		Addr:     redisAddr,
		Password: "",
		DB:       0,
	})
	defer rdb.Close()

	// Verificar conexión Redis
	_, err := rdb.Ping(ctx).Result()
	if err != nil {
		log.Fatalf("Error conectando a Redis :( : %v", err)
	}
	fmt.Println("Conectado a Redis :)")

	// Configurar el lector de Kafka
	reader := kafka.NewReader(kafka.ReaderConfig{
		Brokers:     []string{kafkaBroker},
		Topic:       kafkaTopic,
		Partition:   0,
		MinBytes:    10e3, // 10KB
		MaxBytes:    10e6, // 10MB
		StartOffset: kafka.LastOffset,
		// GroupID:     "grupo-clima",
		GroupID: uuid.New().String(), // Generar un nuevo ID de grupo único
	})
	defer reader.Close()

	fmt.Println("Escuchando mensajes de Kafka...")
	counter := 0
	for {
		m, err := reader.ReadMessage(ctx)
		if err != nil {
			log.Printf("LOG Error leyendo mensaje: %v", err)
			continue
		} else {
			counter++
		}

		msg := string(m.Value)
		// fmt.Printf("Mensaje recibido de Kafka: %s\n", msg)
		log.Printf("LOG Mensaje recibido de Kafka: %s\n", msg)
		// fmt.Printf("message at offset %d: %s = %s\n", m.Offset, string(m.Key), string(m.Value))

		err = reader.CommitMessages(ctx, m)
		if err != nil {
			log.Printf("Error confirmando mensaje: %v", err)
		}

		// if err := reader.Close(); err != nil {
		// 	log.Printf("Error cerrando lector: %v", err)
		// }

		// Almacenar en Redis (puedes cambiar esto según tu formato preferido)

		// err = rdb.LPush(ctx, redisKey, msg).Err()
		// if err != nil {
		// 	log.Printf("Error guardando en Redis: %v", err)
		// } else {
		// 	fmt.Println("Guardado en Redis")
		// 	log.Printf("Guardado en Redis")
		// }

		errr := rdb.Set(ctx, redisKey, strconv.Itoa(counter), 0).Err()
		if errr != nil {
			log.Fatalf("Could not set value in Redis: %v", err) //no write
		} else {
			// log.Printf("Set value in Redis: %s = %s", redisKey, msg) //write
			log.Printf("Set value in Redis: %s = %d", redisKey, counter) //write
			log.Printf("---------------------------------")
		}

		// err = rdb.HSet(ctx, "redis", redisKey, msg).Err()
		// if err != nil {
		// 	log.Fatalf("Could not set value in Redis HASH: %v", err) //no write
		// } else {
		// 	log.Printf("Set value in Redis HASH: %s = %s", redisKey, msg) //write
		// }
	}
}
