FROM golang:1.21-slim as builder

WORKDIR /app

COPY . .

RUN go build -o api ./api/main.go

FROM gcr.io/distroless/base-debian10

COPY --from=builder /app/api /api

EXPOSE 8081

ENTRYPOINT ["/api"]