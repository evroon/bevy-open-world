# https://github.com/adsblol/globe_history

date=2025.12.28

archive_dir=data/archives
filename="v$date-planes-readsb-prod-0.tar"
download_url="https://github.com/adsblol/globe_history_2025/releases/download"

mkdir -p $archive_dir

wget -nc -O $archive_dir/${filename}.aa ${download_url}/v${date}-planes-readsb-prod-0/${filename}.aa
wget -nc -O $archive_dir/${filename}.ab ${download_url}/v${date}-planes-readsb-prod-0/${filename}.ab

mkdir -p ${archive_dir}/${date} || true
cat $archive_dir/${filename}.aa $archive_dir/${filename}.ab | tar -xf - -C ${archive_dir}/${date}

docker exec -it adsb-clickhouse bash -c "ls -la /var/lib/clickhouse/user_files/${date}"
./ingest-file.sh "/var/lib/clickhouse/user_files/${date}/traces/**/*.json"
