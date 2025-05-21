const params = new URLSearchParams(location.search);
const vectorUrl = params.get("vector");
const rasterUrl = params.get("raster");
const minLat = Number(params.get("min_lat"));
const maxLat = Number(params.get("max_lat"));
const minLon = Number(params.get("min_lat"));
const maxLon = Number(params.get("max_lon"));
