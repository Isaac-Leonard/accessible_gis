import tempfile
import argparse
from osgeo import ogr, gdal
import whitebox

wbt = whitebox.WhiteboxTools()
wbt.set_whitebox_dir("/Users/isaac/programming/rust/whitebox-tools/wbt")


def main(input_file: str, output_file: str):
    temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=".tif").name
    wbt.greater_than(input_file, 0, temp_file)
    classified_raster = gdal.Open(temp_file, 1)
    band = classified_raster.GetRasterBand(1)
    gdal.SieveFilter(
        srcBand=band,
        maskBand=None,
        dstBand=band,
        threshold=10,
        connectedness=8,
        callback=None,
    )
    driver = ogr.GetDriverByName("ESRI Shapefile")
    output = driver.CreateDataSource(output_file)
    layer = output.CreateLayer("polygonized", srs=classified_raster.GetSpatialRef())
    field = ogr.FieldDefn("precip", ogr.OFTInteger)
    layer.CreateField(field)
    gdal.Polygonize(band, None, layer, 0, [], callback=None)
    output.Destroy()
    classified_raster = None


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Find spots in a radar image where there is rain and generate polygons for any significant features"
    )
    parser.add_argument("input_file", help="Path to the radar raster dataset")
    parser.add_argument("output_file", help="Path to the output vector file")
    args = parser.parse_args()

    main(args.input_file, args.output_file)
