use gdal::vector::Feature;
use geo::{Closest, ClosestPoint, GeodesicDistance};
use geo_types::{Geometry as GeoGeometry, Point};
use serde::{Deserialize, Serialize};

use super::fields::Field;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFeatureInfo {
    pub fields: Vec<Field>,
    pub geometry: GeoGeometry,
}

impl LocalFeatureInfo {
    pub fn get_field(&self, name: &str) -> Option<String> {
        Some(
            self.fields
                .iter()
                .find(|f| f.name == name)?
                .value
                .to_string()
                .clone(),
        )
    }

    /// Note, this operates on WGS84 coordinates
    pub fn nearest_point(&self, point: &Point) -> Option<f64> {
        Some(match self.geometry.closest_point(point) {
            Closest::SinglePoint(p) => p.geodesic_distance(point),
            Closest::Intersection(_) => 0.0,
            Closest::Indeterminate => return None,
        })
    }

    pub fn get_name(&self) -> Option<String> {
        Some(
            self.fields
                .iter()
                .find(|f| f.name == "name")?
                .value
                .to_string()
                .clone(),
        )
    }
}

impl From<Feature<'_>> for LocalFeatureInfo {
    fn from(feature: Feature) -> Self {
        LocalFeatureInfo {
            geometry: feature.geometry().unwrap().to_geo().unwrap(),
            fields: feature.fields().map(Into::into).collect(),
        }
    }
}
