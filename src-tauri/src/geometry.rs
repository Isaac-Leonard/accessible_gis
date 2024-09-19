use enum_as_inner::EnumAsInner;
use geo::Intersects;
pub use geo_types::{
    Coord, Geometry as GeoGeometry, GeometryCollection as GeoGeometryCollection, Line as GeoLine,
    LineString as GeoLineString, MultiLineString as GeoMultiLineString,
    MultiPoint as GeoMultiPoint, MultiPolygon as GeoMultiPolygon, Point as GeoPoint,
    Polygon as GeoPolygon,
};
use itertools::Itertools;
use proj::Coord as _;
use rstar::RTreeObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl From<GeoPoint> for Point {
    fn from(value: GeoPoint) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
        }
    }
}

impl From<Point> for GeoPoint {
    fn from(value: Point) -> Self {
        GeoPoint::from_xy(value.x, value.y)
    }
}

impl From<Coord> for Point {
    fn from(value: Coord) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Point> for Coord {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct LineString {
    pub points: Vec<Point>,
}

impl From<LineString> for GeoLineString {
    fn from(value: LineString) -> Self {
        GeoLineString::new(value.points.into_iter().map_into().collect())
    }
}

impl From<GeoLineString> for LineString {
    fn from(value: GeoLineString) -> Self {
        Self {
            points: value.into_iter().map_into().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Polygon {
    pub exterior: LineString,
    pub interior: Vec<LineString>,
}

impl From<Polygon> for GeoPolygon {
    fn from(value: Polygon) -> Self {
        Self::new(
            value.exterior.into(),
            value.interior.into_iter().map_into().collect(),
        )
    }
}

impl From<GeoPolygon> for Polygon {
    fn from(value: GeoPolygon) -> Self {
        {
            let (exterior, interior) = value.into_inner();
            Self {
                exterior: exterior.into(),
                interior: interior.into_iter().map_into().collect(),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct MultiPoint {
    points: Vec<Point>,
}

impl From<MultiPoint> for GeoMultiPoint {
    fn from(value: MultiPoint) -> Self {
        Self(value.points.into_iter().map_into().collect())
    }
}

impl From<GeoMultiPoint> for MultiPoint {
    fn from(value: GeoMultiPoint) -> Self {
        Self {
            points: value.0.into_iter().map_into().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct MultiLineString {
    lines: Vec<LineString>,
}

impl From<MultiLineString> for GeoMultiLineString {
    fn from(value: MultiLineString) -> Self {
        Self(value.lines.into_iter().map_into().collect())
    }
}

impl From<GeoMultiLineString> for MultiLineString {
    fn from(value: GeoMultiLineString) -> Self {
        Self {
            lines: value.0.into_iter().map_into().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct MultiPolygon {
    polygons: Vec<Polygon>,
}

impl From<MultiPolygon> for GeoMultiPolygon {
    fn from(value: MultiPolygon) -> Self {
        Self(value.polygons.into_iter().map_into().collect())
    }
}

impl From<GeoMultiPolygon> for MultiPolygon {
    fn from(value: GeoMultiPolygon) -> Self {
        Self {
            polygons: value.into_iter().map_into().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl From<Line> for GeoLine {
    fn from(value: Line) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl From<GeoLine> for Line {
    fn from(value: GeoLine) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type, EnumAsInner)]
#[serde(tag = "type")]
pub enum Geometry {
    Point(Point),
    Line(Line),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}

impl From<GeoGeometry> for Geometry {
    fn from(value: GeoGeometry) -> Self {
        match value {
            GeoGeometry::Point(p) => Geometry::Point(p.into()),
            GeoGeometry::Line(line) => Self::Line(line.into()),
            GeoGeometry::LineString(points) => Self::LineString(points.into()),
            GeoGeometry::Polygon(poly) => Self::Polygon(poly.into()),
            GeoGeometry::MultiPoint(points) => Self::MultiPoint(points.into()),
            GeoGeometry::MultiLineString(lines) => Self::MultiLineString(lines.into()),
            GeoGeometry::MultiPolygon(polygons) => Self::MultiPolygon(polygons.into()),
            GeoGeometry::GeometryCollection(geometries) => {
                Self::GeometryCollection(geometries.into())
            }
            GeoGeometry::Rect(rect) => Self::Polygon(rect.to_polygon().into()),
            GeoGeometry::Triangle(triangle) => Self::Polygon(triangle.to_polygon().into()),
        }
    }
}

impl From<Geometry> for GeoGeometry {
    fn from(value: Geometry) -> Self {
        match value {
            Geometry::Point(p) => Self::Point(p.into()),
            Geometry::Line(line) => Self::Line(line.into()),
            Geometry::LineString(line) => Self::LineString(line.into()),
            Geometry::Polygon(poly) => Self::Polygon(poly.into()),
            Geometry::MultiPoint(points) => Self::MultiPoint(points.into()),
            Geometry::MultiLineString(lines) => Self::MultiLineString(lines.into()),
            Geometry::MultiPolygon(polygons) => Self::MultiPolygon(polygons.into()),
            Geometry::GeometryCollection(geometries) => Self::GeometryCollection(geometries.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct GeometryCollection {
    geometries: Vec<Geometry>,
}

impl From<GeometryCollection> for GeoGeometryCollection {
    fn from(value: GeometryCollection) -> Self {
        GeoGeometryCollection(value.geometries.into_iter().map(Into::into).collect())
    }
}

impl From<GeoGeometryCollection> for GeometryCollection {
    fn from(value: GeoGeometryCollection) -> Self {
        Self {
            geometries: value.0.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeometryType {
    Polygon,
    Point,
    Line,
    LineString,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SingleGeometryType {
    Polygon,
    Point,
    Line,
    LineString,
}

pub trait ToGeometryType {
    fn to_type(&self) -> GeometryType;
}

impl ToGeometryType for GeoGeometry {
    fn to_type(&self) -> GeometryType {
        match self {
            Self::Point(_) => GeometryType::Point,
            Self::Line(_) => GeometryType::Line,
            Self::LineString(_) => GeometryType::LineString,
            Self::Polygon(_) => GeometryType::Polygon,
            Self::MultiPoint(_) => GeometryType::MultiPoint,
            Self::MultiPolygon(_) => GeometryType::MultiPolygon,
            Self::MultiLineString(_) => GeometryType::MultiLineString,
            Self::GeometryCollection(_) => GeometryType::GeometryCollection,
            Self::Rect(_) | Self::Triangle(_) => GeometryType::Polygon,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumAsInner)]
#[serde(tag = "type")]
pub enum SingleGeometry {
    Point(Point),
    Line(Line),
    LineString(LineString),
    Polygon(Polygon),
}

impl From<SingleGeometry> for Geometry {
    fn from(value: SingleGeometry) -> Self {
        match value {
            SingleGeometry::Point(point) => Geometry::Point(point),
            SingleGeometry::Line(line) => Geometry::Line(line),
            SingleGeometry::LineString(line) => Geometry::LineString(line),
            SingleGeometry::Polygon(poly) => Geometry::Polygon(poly),
        }
    }
}

impl TryFrom<Geometry> for SingleGeometry {
    type Error = ();
    fn try_from(value: Geometry) -> Result<Self, ()> {
        Ok(match value {
            Geometry::Point(point) => SingleGeometry::Point(point),
            Geometry::Line(line) => SingleGeometry::Line(line),
            Geometry::LineString(line) => SingleGeometry::LineString(line),
            Geometry::Polygon(poly) => SingleGeometry::Polygon(poly),
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeoSingleGeometry {
    Point(GeoPoint),
    Line(GeoLine),
    LineString(GeoLineString),
    Polygon(GeoPolygon),
}

impl From<GeoSingleGeometry> for GeoGeometry {
    fn from(value: GeoSingleGeometry) -> Self {
        match value {
            GeoSingleGeometry::Point(point) => Self::Point(point),
            GeoSingleGeometry::Line(line) => Self::Line(line),
            GeoSingleGeometry::LineString(line) => Self::LineString(line),
            GeoSingleGeometry::Polygon(poly) => Self::Polygon(poly),
        }
    }
}

impl TryFrom<GeoGeometry> for GeoSingleGeometry {
    type Error = ();
    fn try_from(value: GeoGeometry) -> Result<Self, ()> {
        Ok(match value {
            GeoGeometry::Point(point) => Self::Point(point),
            GeoGeometry::Line(line) => Self::Line(line),
            GeoGeometry::LineString(points) => Self::LineString(points),
            GeoGeometry::Polygon(poly) => Self::Polygon(poly),
            _ => return Err(()),
        })
    }
}

impl From<SingleGeometry> for GeoSingleGeometry {
    fn from(value: SingleGeometry) -> Self {
        match value {
            SingleGeometry::Point(point) => Self::Point(point.into()),
            SingleGeometry::Line(line) => Self::Line(line.into()),
            SingleGeometry::LineString(line) => Self::LineString(line.into()),
            SingleGeometry::Polygon(poly) => Self::Polygon(poly.into()),
        }
    }
}

impl From<GeoSingleGeometry> for SingleGeometry {
    fn from(value: GeoSingleGeometry) -> Self {
        match value {
            GeoSingleGeometry::Point(point) => Self::Point(point.into()),
            GeoSingleGeometry::Line(line) => Self::Line(line.into()),
            GeoSingleGeometry::LineString(line) => Self::LineString(line.into()),
            GeoSingleGeometry::Polygon(poly) => Self::Polygon(poly.into()),
        }
    }
}

impl RTreeObject for GeoSingleGeometry {
    type Envelope = <GeoPolygon as RTreeObject>::Envelope;
    fn envelope(&self) -> Self::Envelope {
        match self {
            Self::Polygon(poly) => poly.envelope(),
            Self::LineString(line) => line.envelope(),
            Self::Line(line) => line.envelope(),
            Self::Point(point) => point.envelope(),
        }
    }
}

impl Intersects for Geometry {
    fn intersects(&self, rhs: &Self) -> bool {
        GeoGeometry::from(self.clone()).intersects(&GeoGeometry::from(rhs.clone()))
    }
}

impl Intersects for SingleGeometry {
    fn intersects(&self, rhs: &Self) -> bool {
        Geometry::from((*self).clone()).intersects(&Geometry::from((*rhs).to_owned()))
    }
}

pub trait AsPoint {
    fn as_point(&self) -> Option<&GeoPoint>;
}

impl AsPoint for GeoGeometry {
    fn as_point(&self) -> Option<&GeoPoint> {
        match self {
            Self::Point(ref p) => Some(p),
            _ => None,
        }
    }
}

pub fn points_to_single_geometry(
    points: Vec<GeoPoint>,
    geometry: SingleGeometryType,
) -> Result<GeoSingleGeometry, (String, Vec<GeoPoint>)> {
    let res = match geometry {
        SingleGeometryType::Point => {
            if points.len() > 1 {
                Err("Too many points to save as a single point".to_owned())
            } else {
                Ok(GeoSingleGeometry::Point(points[0]))
            }
        }
        SingleGeometryType::Line => match points[..] {
            [start, end] => Ok(GeoSingleGeometry::Line(GeoLine::new(start, end))),
            _ => Err(format!("Cannot make line from {} points", points.len())),
        },
        SingleGeometryType::Polygon => {
            if points.len() < 3 {
                Err("A polygon needs at least 3 points".to_owned())
            } else {
                Ok(GeoSingleGeometry::Polygon(GeoPolygon::new(
                    points.clone().into(),
                    Vec::new(),
                )))
            }
        }
        SingleGeometryType::LineString => {
            if points.len() < 2 {
                Err("A line string needs at least 2 points".to_owned())
            } else {
                Ok(GeoSingleGeometry::LineString(GeoLineString::from(
                    points.clone(),
                )))
            }
        }
    };
    res.map_err(|e| (e, points))
}
