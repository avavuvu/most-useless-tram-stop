import pandas as pd
import math
import os
import requests

def merge_names(args):
    stop_a = args["stop_a_name"]
    stop_b = args["stop_b_name"]

    set_a = set(stop_a.split("/"))
    set_b = set(stop_b.split("/"))

    road_stops_are_on_set = set_a.intersection(set_b)

    if len(road_stops_are_on_set) != 1:
        print(f"Intersection not found in {set_a} and {set_b} ({road_stops_are_on_set})")

        return f"**{stop_a}** to **{stop_b}**"
    
    road_stops_are_on = list(road_stops_are_on_set)[0]
    unique_a = list(set_a.difference(set_b))[0]
    unique_b = list(set_b.difference(set_a))[0]

    return f"**{unique_a}** to **{unique_b}** on {road_stops_are_on}"

def id_name(row):
    name: str = row["merged_name"]
    return name.replace(" ", "-").replace("*", "").replace("/","-").lower()

def calculate_heading(lat_a, long_a, lat_b, long_b):
    lat1 = math.radians(lat_a)
    lat2 = math.radians(lat_b)
    diff_long = math.radians(long_b - long_a)
    
    x = math.sin(diff_long) * math.cos(lat2)
    y = math.cos(lat1) * math.sin(lat2) - (math.sin(lat1) * math.cos(lat2) * math.cos(diff_long))
    
    initial_bearing = math.atan2(x, y)
    
    initial_bearing = math.degrees(initial_bearing)
    heading = (initial_bearing + 360) % 360
    
    return heading


def generate_image(lat_a, long_a, lat_b, long_b, name):
    heading = calculate_heading(lat_b, long_b, lat_a, long_a)

    src = f"https://maps.googleapis.com/maps/api/streetview?size=640x480&location={lat_b},{long_b}&fov=80&heading={-heading}&pitch=0&key={APIKEY}"
    
    response = requests.get(src)

    file_name = f"../data/images/{name}.jpg"
    os.makedirs(os.path.dirname(file_name), exist_ok=True)

    if response.status_code == 200:
        with open(file_name, 'wb') as f:
            f.write(response.content)
        print(f"Image downloaded successfully as {file_name}")
        return heading
    else:
        print(f"Failed to download image. Status code: {response.status_code}")
        return None


def csv_to_md(images=False, take=15):
    df = pd.read_csv("../data/output.csv")

    df["merged_name"] = df.apply(merge_names, axis=1)
    df["id_name"] = df.apply(id_name, axis=1)

    df["length"] = df["length"].apply(lambda v : "{:.2f}m".format( v * 1000))
    df["link"] = df[["stop_a_lat", "stop_a_long"]].apply(
        lambda arg : f"[Google Maps](https://www.google.com/maps?q={arg["stop_a_lat"]},{arg["stop_a_long"]})",
        axis=1)
    df["street_view"] = df.apply(
        lambda row: f'{{% street-view lat="{row["stop_a_lat"]}" long="{row["stop_a_long"]}" name="{row["id_name"]}" /%}}',
        axis=1
    )

    if images:
        for i, row in df.iterrows():
            if i > take:
                continue

            generate_image(row["stop_a_lat"], row["stop_a_lat"], row["stop_b_lat"], row["stop_b_long"], row["id_name"])
        
    ## this is so dumb
    df["rank"] = df.index + 1
    df["rank"] = df["rank"].apply(lambda v : f"{v}\\.")

    df = df.rename(columns={
        "length": "Distance",
        "link": "Link",
        "street_view": "Street View",
        "merged_name": "Name",
        "rank": "Rank"
    })
    df=df[["Rank", "Name", "Distance", "Link", "Street View"]]

    rows = []
    for i, row in df.iterrows():
        if i >= take:
            break

        # Regular data row
        rows.append(f"- {row['Rank']} {{% .rank %}}")
        rows.append(f"- {row['Name']}")
        rows.append(f"- {row['Distance']}")
        rows.append(f"- {row['Link']}")
        rows.append("")
        rows.append("---")
        rows.append("")
        # Embed row with colspan
        rows.append(f"- {row['Street View']} {{% colspan=4 %}} ")
        rows.append("")
        rows.append("---")
        rows.append("")

    # Write to file
    with open("../data/markdown_table.md", "w") as f:
        f.write("{% table %}\n\n")
        # Header
        f.write("- Rank\n")
        f.write("- Name\n")
        f.write("- Distance\n")
        f.write("- Link\n\n")
        f.write("---\n\n")
        # Data rows
        f.write("\n".join(rows))
        f.write("{% /table %}\n")

if __name__ == "__main__":
    csv_to_md()