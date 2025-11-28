import pandas as pd
import pandas_geojson as pdg
import math
import re

def extract_relations(rel):
    if type(rel) != dict:
        return None
    
    df = pd.DataFrame(rel)

    return df["reltags"]["name"] if rel else None

def find_standard_routes(s: str):
    if s == None:
        return False

    matches = re.match(r"Tram \d+:", s)
    
    if matches == None:
        return False
    
    return True

def clean_coords(coordinates: str):
    return f"{coordinates[0]},{coordinates[1]}"

def find_stop_number(name: str):
    match = re.findall(r"D*(\d+)", name)
    
    return match[0]

def _export_to_geojson(df: pd.DataFrame):
    df["type"] = "Point"
    # df = df[df["name"].isin(["Tram 70: Waterfront City => Wattle Park","Tram 70: Wattle Park => Waterfront City"])]

    geojson = pdg.GeoJSON.from_dataframe(df
                                     ,geometry_type_col='type'
                                     ,coordinate_col='geometry.coordinates'
                                     ,property_col_list=["name"])
    pdg.save_geojson(geojson, "../data/filtered.geojson")

def main():
    df = pdg.read_geojson("../data/overpass.geojson").to_dataframe()

    exploded = df.explode("properties.@relations")

    exploded["route_name"] = exploded["properties.@relations"].apply(extract_relations)
    exploded = exploded[exploded["route_name"].apply(find_standard_routes)]

    exploded["geometry.coordinates"] = exploded["geometry.coordinates"].apply(clean_coords)

    exploded['stop_number'] = exploded['properties.name'].apply(find_stop_number)

    exploded["name"] = exploded["properties.name"].apply(lambda a: re.sub(r"Stop \d*D*A*: ", "", a))

    out = exploded[["geometry.coordinates", "route_name", "name", "stop_number"]]

    out.to_csv("../data/osm_tram_stops.tsv", "\t", index=False)
    


if __name__ == "__main__":
    main()