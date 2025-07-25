use gdal::vector::Feature;
use geo::{Closest, ClosestPoint, GeodesicDistance};
use geo_types::{Geometry as GeoGeometry, Point};
use serde::{Deserialize, Serialize};

use crate::errors::ErrorDetails;

use super::fields::Field;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFeatureInfo {
    pub fields: Vec<Field>,
    pub geometry: Option<GeoGeometry>,
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
        Some(match self.geometry.as_ref()?.closest_point(point) {
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

impl TryFrom<Feature<'_>> for LocalFeatureInfo {
    type Error = ErrorDetails;

    fn try_from(feature: Feature) -> Result<Self, Self::Error> {
        Ok(LocalFeatureInfo {
            geometry: feature
                .geometry()
                .map(|geom| {
                    geom.to_geo()
                        .map_err(|err| ErrorDetails::Other(err.to_string()))
                })
                .transpose()?,
            fields: feature.fields().map(Into::into).collect(),
        })
    }
}
