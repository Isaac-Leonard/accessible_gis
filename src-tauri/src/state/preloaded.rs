use geo::{Contains, Intersects};
use geo_types::{Geometry as GeoGeometry, Point, Polygon};
use rstar::{primitives::GeomWithData, RTree, RTreeObject};

use crate::{
    gdal_if::{Field, LocalFeatureInfo},
    geometry::{GeoSingleGeometry, SingleGeometry},
};

pub type Country = GeomWithData<Polygon, Vec<Field>>;

impl From<LocalFeatureInfo> for Vec<Country> {
    fn from(value: LocalFeatureInfo) -> Self {
        let polygons = match value.geometry {
            GeoGeometry::Polygon(polygon) => vec![polygon],
            GeoGeometry::MultiPolygon(polygons) => polygons.0,
            _ => panic!("Unexpected geometry in country"),
        };
        polygons
            .into_iter()
            .map(|p| Country::new(p, value.fields.clone()))
            .collect()
    }
}

impl From<Country> for LocalFeatureInfo {
    fn from(value: Country) -> Self {
        Self {
            geometry: GeoGeometry::Polygon(value.geom().clone()),
            fields: value.data,
        }
    }
}

pub trait CountryImpl {
    fn get_code(&self) -> String;
    fn get_field(&self, name: &str) -> Option<String>;
}

impl CountryImpl for Country {
    fn get_code(&self) -> String {
        self.data
            .iter()
            .find(|x| x.name == "ISO_A2")
            .unwrap()
            .value
            .to_string()
    }

    fn get_field(&self, name: &str) -> Option<String> {
        Some(
            self.data
                .iter()
                .find(|f| f.name == name)?
                .value
                .to_string()
                .clone(),
        )
    }
}

#[derive(Clone)]
pub struct PreloadedAppData {
    pub countries: RTree<Country>,
}

impl PreloadedAppData {
    /// Point must be a WGS84 point
    pub fn find_country(&self, point: &Point) -> Option<&Country> {
        self.countries
            .iter()
            .find(|country| country.geom().contains(point))
    }

    pub fn get_intersecting_countries<'a>(
        &'a self,
        geometry: &'a SingleGeometry,
    ) -> Vec<&'a Country> {
        let from = GeoSingleGeometry::from(geometry.clone());
        self.countries
            .locate_in_envelope_intersecting(&from.envelope())
            // TODO: Not sure if this line is needed
            .filter(|x| SingleGeometry::Polygon(x.geom().clone().into()).intersects(geometry))
            .collect()
    }
}
