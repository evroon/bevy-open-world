# ADSB visualizer

Run:

```sh
./setup.sh
./ingestion.sh
```

Check the number of rows in the db using:

```sh
docker exec -it adsb-clickhouse clickhouse-client --query "SELECT count(*) FROM planes_mercator;"
```
