import pandas as pd
import pandas_geojson
import json
import re

def clean_coords(coordinates: str):
    return f"{coordinates[0]},{coordinates[1]}"

def find_stop_number(name: str):
    match = re.findall(r"#D*(\d+)", name)
    
    return match[0]

def main():
    df = pandas_geojson.read_geojson("../data/public_transport_stops.geojson").to_dataframe()
    df = df[df["properties.MODE"] == "METRO TRAM"]

    df["geometry.coordinates"] = df["geometry.coordinates"].apply(clean_coords)
    
    out = df[["geometry.coordinates", "properties.STOP_NAME"]]
    out = out.rename(columns={
        "geometry.coordinates": "coordinates",
        "properties.STOP_NAME": "name"
    })

    out = out.drop_duplicates(subset=["coordinates"])
    out = out.drop_duplicates(subset=["name"])

    out['stop_number'] = out['name'].apply(find_stop_number)

    out["name"] = out["name"].apply(lambda a: re.sub(r" #D*(\d+)", "", a))

    out.to_csv("../data/tram_stops.tsv","\t", index=False)


if __name__ == "__main__":
    main()
