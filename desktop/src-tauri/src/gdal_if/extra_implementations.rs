use gdal::spatial_ref::CoordTransform;
use geo::Point;
pub trait TransformPoint {
    fn transform_point(&self, point: Point) -> Point;
}

impl TransformPoint for CoordTransform {
    fn transform_point(&self, point: Point) -> Point {
        let mut x = [point.0.x];
        let mut y = [point.0.y];
        self.transform_coords(x.as_mut_slice(), y.as_mut_slice(), [].as_mut_slice())
            .unwrap();
        Point::new(x[0], y[0])
    }
}
