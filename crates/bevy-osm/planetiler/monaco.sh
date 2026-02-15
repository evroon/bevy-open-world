docker run -e JAVA_TOOL_OPTIONS="-Xmx1g" -v "$(pwd)/data":/data ghcr.io/onthegomap/planetiler:latest --download --area=monaco
sudo chown -R erik:erik data
docker run --rm -it -v $(pwd)/data:/data -p 8080:8080 maptiler/tileserver-gl-light --file output.mbtiles
