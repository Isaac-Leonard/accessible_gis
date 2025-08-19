import pyproj
from typing import Any, Dict, List
from shapely.geometry import shape
import fiona, fiona.transform

import argparse


def main(precipitation_features_file: str, towns_file: str) -> str:
    precipitation = fiona.open(precipitation_features_file, "r")
    print(precipitation.bounds)
    towns = fiona.open(towns_file, "r")
    transformer = pyproj.Transformer.from_crs(
        precipitation.crs,
        towns.crs,
    )
    bounds = transformer.transform_bounds(*precipitation.bounds)
    print(bounds)
    included_towns = [
        {
            "properties": town["properties"],
            "geometry": shape(
                fiona.transform.transform_geom(
                    towns.crs, precipitation.crs, town["geometry"]
                )
            ),
        }
        for town in towns.filter(bbox=(bounds[1], bounds[0], bounds[3], bounds[2]))
    ]
    for town in included_towns:
        print(town["properties"]["name"])
    grouped_towns: List[List[Dict[str, Any]]] = []
    precipitation_list = list(precip for precip in precipitation)
    for precip_feature in precipitation_list:
        towns_for_feature: List[Dict[str, Any]] = []
        for town in included_towns:
            town_geometry = town["geometry"]
            precip_geometry = shape(precip_feature["geometry"])
            if precip_geometry.contains(town_geometry):
                towns_for_feature.append(town["properties"])
        grouped_towns.append(towns_for_feature)

    for properties, feature in zip(grouped_towns, precipitation_list):
        properties.sort(key=lambda props: props["population"], reverse=True)
        if not properties:
            towns = "No towns"
        else:
            towns = properties[0]["name"]
            if len(towns) > 1:
                towns += ", " + properties[1]["name"]
                if len(towns) > 2:
                    towns += ", " + properties[2]["name"]

        area = shape(feature["geometry"]).area / 1000000
        if area > 25 or len(properties) > 0:
            print(f"feature with area {area:.2f}km^2 over {towns}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Find spots in a radar image where there is rain and generate polygons for any significant features"
    )
    parser.add_argument("precipitation_file", help="Vector of precipitation features")
    parser.add_argument(
        "towns_file", help="Path to a vector of towns for the relevant region"
    )
    args = parser.parse_args()

    main(args.precipitation_file, args.towns_file)
