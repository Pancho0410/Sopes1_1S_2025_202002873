FROM golang:1.21-slim as builder

WORKDIR /app

# Copiamos todo el contexto
COPY . .

# Compilamos solo el servidor gRPC
RUN go build -o grpc_server ./grpc_server/main.go

# Imagen final más liviana
FROM gcr.io/distroless/base-debian10

COPY --from=builder /app/grpc_server /grpc_server

EXPOSE 50051

ENTRYPOINT ["/grpc_server"]
