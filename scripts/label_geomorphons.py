import argparse
from osgeo import ogr

# Define your mapping
landforms = [
    "flat",
    " peak",
    " ridge",
    " shoulder",
    " spur",
    " slope",
    " hollow",
    " footslope",
    " valley",
    " pit",
]


def main(input_file, field_name):
    ds = ogr.Open(input_file, 1)  # 1 = read/write
    if ds is None:
        raise RuntimeError(f"Failed to open {input_file}")
    layer = ds.GetLayer()

    new_field_name = "landform"
    if layer.GetLayerDefn().GetFieldIndex(new_field_name) == -1:
        new_field = ogr.FieldDefn(new_field_name, ogr.OFTString)
        layer.CreateField(new_field)

    for feature in layer:
        idx = feature.GetField(field_name)
        label = (
            landforms[idx - 1]
            if isinstance(idx, int) and 0 < idx <= len(landforms)
            else "Unknown"
        )
        feature.SetField(new_field_name, label)
        layer.SetFeature(feature)

    ds = None  # Save and close
    print(f"Updated {input_file} with new field '{new_field_name}'")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Convert integer index field to string labels."
    )
    parser.add_argument(
        "input_file", help="Path to the vector dataset (e.g. .gpkg or .shp)"
    )
    parser.add_argument("field_name", help="Name of the integer field to transform")
    args = parser.parse_args()

    main(args.input_file, args.field_name)
