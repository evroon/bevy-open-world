import gzip
import os
import json
import pandas as pd

# https://www.adsbexchange.com/version-2-api-wip/
#
# mkdir 2025.08.01
# cat v2025.08.01-planes-readsb-prod-0.tar.aa v2025.08.01-planes-readsb-prod-0.tar.ab | tar -xf - -C 2025.08.01

date = "2025.12.28"
traces_path = f"data/archives/{date}/traces"

target_time = 1754051908


def get_positions(
    traces_path: str, target_time: float, max_planes: int = 1000
) -> tuple[list[float], list[float], list[float]]:
    lat, lon, heading = [], [], []
    for dir in os.listdir(traces_path):
        for path in os.listdir(f"{traces_path}/{dir}"):
            with gzip.open(f"{traces_path}/{dir}/{path}", "rb") as f_in:
                data = json.load(f_in)
                start_time = data["timestamp"]

                for trace in data["trace"]:
                    trace_time = start_time + trace[0]
                    if abs(trace_time - target_time) < 1.0:
                        lat.append(trace[1])
                        lon.append(trace[2])
                        heading.append(trace[5])

                        if len(lat) > max_planes:
                            return lat, lon, heading

    return lat, lon, heading


lat, lon, heading = get_positions(traces_path, target_time)
df = pd.DataFrame({"lat": lat, "lon": lon, "heading": heading})

with open(f"data/archives/{date}.{target_time}.csv", "wb") as f_out:
    df.to_csv(f_out)
