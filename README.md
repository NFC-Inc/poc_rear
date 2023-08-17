# Running Locally
```
docker compose up -d

RUST_LOG=INFO DEV_MODE=true MONGODB_URI=mongodb://0.0.0.0:27017 cargo run -p poc_rear
```

running in docker: (starting the app and the supporting services)
```
docker compose -f docker-compose.yml -f docker-compose-app.yml up -d
```

