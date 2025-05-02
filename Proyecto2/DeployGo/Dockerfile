FROM golang:1.24.2-bookworm AS build

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN go build \
    -ldflags="-linkmode external -extldflags -static" \
    -tags netgo \
    -o api-go

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=build /app/api-go /api-go

EXPOSE 8081

CMD ["/api-go"]
