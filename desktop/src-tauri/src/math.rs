use gdal::spatial_ref::SpatialRef;
use geo::{EuclideanDistance, GeodesicDistance};
use geo_types::Point;

pub fn distance_between_points(a: Point, b: Point, srs: &SpatialRef) -> f64 {
    // TODO: Would reprojecting each point into UTM work better?
    if srs.is_geographic() {
        a.geodesic_distance(&b)
    } else {
        a.euclidean_distance(&b)
    }
}

/*
impl SafePoint {
    fn distance(&self, p: &Self) -> f64 {
        self.srs.to_wkt().unwrap();
    }
}

pub struct Projection {}

pub struct CRS {}

pub struct Spheroid {
    name: String,
    minor_axis: f64,
    major_axis: f64,
}

pub enum ProjectionType {
    Planar,
    cylindrical,
    Conical,
}
pub struct Datum {
    name: String,
    spheroid: Spheroid,
    center: Point,
}
impl Projection {
    fn utm(zone: usize, north: bool) -> Self {
        if !north {
            northing = 10000000;
        }
        let meridian = -180 + zone * 6 - 3;
    }
}

enum UnitType {
    Meters,
    Feet,
    Degree,
}

pub struct GCS {
    name: String,
    datum: Datum,
    unit: Unit,
}

pub struct Unit {
    multiplier: f64,
    unit: UnitType,
}

impl GCS {
    fn gda1994() -> Self {
        Self {
            name: "GCS_GDA_1994".to_owned(),
            datum: Datum {
                name: "D_GDA_1994".to_owned(),
                spheroid: Spheroid {
                    name: "GRS_1980".to_owned(),
                    axis: 6378137.0,
                    flattening: 298.257222101,
                },
            },
            unit: Unit {
                multiplier: 0.0174532925199433,
                unit: UnitType::Degree,
            },
        }
    }
}
*/
