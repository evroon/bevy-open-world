docker compose up -d
docker exec -it adsb-clickhouse clickhouse-client --query "$(cat setup.sql)"
