# Accessible GIS
A GUI application for working with GIS data for the blind
Note: This is extremely experimental and currently only intended for my own personal use
## How to run
This app requires gdal, nodejs and rustup to be installed.
Run
```
npm install
```
in the root directory
Run
```
npm install
```
in the external-touch-device directory
Then run
```
npm run tauri dev
```
in the root directory and the app should open.
## Contributing
Pull requests and issues are very welcome however any added features must be 100% accessible with the MacOS voiceover screenreader and should preferably be accessible with any other screen reader.

## Features
### Raster
#### DEM
It is currently possible to generate slope, aspect and roughness maps from a given DEM.
#### Reprojection
Any dataset can be reprojected given an EPSG code, PROJ4 string, ESRI code or WKT projection string.
#### Classification
The pixels in any raster dataset can be classified with custom ranges and target values to produce a new raster with the classification applied.
#### Audio
An audio histogram can be played of any raster, either directly from the command line with various options or from the UI with no options.
An entire raster can be played in audio, either directly from the command line with various options or from the UI with no options.
It is also possible to explore a raster image manually with an external touch screen device that plays a given tone depending on the brightness of the current pixel being touched.
### Vector
#### Descriptions
The Points and attributes / fields of vector features can be examined and simple descriptions can be generated.
#### Operations
Any dataset can be reprojected given an EPSG code, PROJ4 string, ESRI code or WKT projection string.
New datasets can be created by selecting a subset of features from an existing dataset.
New datasets can be created by simplifying the geometries of existing datasets.
### Example workflow
It is possible for a user to download a vector dataset of an area they want to examine, select a subset of specific features, reproject it to a specific projection, simplify the geometries to make it simpler to work with and then use the result to download a DEM of the area from a source like [Elvis](https://elevation.fsdf.org.au) then reproject into a new crs and generate derived datasets like slope or aspect profiles and finally classify the resulting pixels into a new datas
