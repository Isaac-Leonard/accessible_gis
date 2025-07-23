use std::f64::consts::PI;

use geo::{Centroid, Coord, CoordsIter, EuclideanDistance, MapCoords, Polygon};

/// Algorithm derived from the paper at:
/// https://www.graphyonline.com/archives/IJCSE/2018/IJCSE-139/
pub fn shape_correspondence(shape1: Polygon, shape2: Polygon) -> f64 {
    // We must ensure each shape is translation and scale invariant.
    let shape1 = normalise_polygon(shape1);
    let shape2 = normalise_polygon(shape2);

    // The first shape must be the one with fewer points
    // o will be rotated to match p
    // Using short names p and o to match the algorithm in the paper
    let (o, p) = if shape1.coords_count() >= shape2.coords_count() {
        (shape1, shape2)
    } else {
        (shape2, shape1)
    };
    let mut correspondence_list = Vec::<(Coord, Coord)>::new();
    let mut outliers = Vec::new();
    for o_i in o.coords_iter() {
        let mut candidates = Vec::new();
        for p_j in p.coords_iter() {
            let theta_o = normalize_angle(o_i.y.atan2(o_i.x));
            let theta_p = normalize_angle(p_j.y.atan2(p_j.x));
            if (theta_p - theta_o).abs() < 2.0 * PI / o.coords_count() as f64 {
                candidates.push(p_j);
            };
        }
        let mut matched = false;
        candidates.sort_by_key(|a| FloatWrapper(a.euclidean_distance(&o_i)));
        for candidate in candidates {
            if !correspondence_list.contains(&(o_i, candidate)) {
                correspondence_list.push((o_i, candidate));
                matched = true;
                break;
            }
        }
        if !matched {
            outliers.push(o_i)
        }
    }

    let distance_iter = correspondence_list
        .iter()
        .map(|(a, b)| a.euclidean_distance(b));
    let mean_distance: f64 = (distance_iter.clone().sum::<f64>() + outliers.len() as f64)
        / (correspondence_list.len() + outliers.len()) as f64;
    let max_distance = distance_iter.clone().max_by(f64::total_cmp).unwrap();
    return mean_distance * 0.5 + max_distance * 0.5;
}

pub fn normalise_polygon(polygon: Polygon) -> Polygon {
    // We subtract the value of the centroid coordinates from each vertex that makes up the polygon, ensuring it is centered at the origin.

    // Unwrap is fine here, the centroid trait returns an Option but the implementation for polygons never returns None
    let centroid = polygon.centroid().unwrap();
    let polygon = polygon.map_coords(|Coord { x, y }| Coord {
        x: x - centroid.0.x,
        y: y - centroid.0.y,
    });

    // We now scale each point by the inverse of the distance of the point furthest from the center.
    // This ensures every point is less then 1 unit from the origin.
    let max_distance = polygon
        .coords_iter()
        .map(|Coord { x, y }| x.hypot(y))
        .max_by(f64::total_cmp)
        // Unwrap here is safe, polygons must have at least 3 points so we're guaranteed a result.
        .unwrap()
        // Take the square root here as the max function will produce the same result on the squared distances which means we can take the square root once instead of for each point in the polygon.
        .sqrt();

    polygon.map_coords(|Coord { x, y }| Coord {
        x: x / max_distance,
        y: y / max_distance,
    })
}

fn normalize_angle(theta: f64) -> f64 {
    let mut t = theta % (2.0 * PI);
    if t < 0.0 {
        t += 2.0 * PI;
    }
    t
}

/// Just used so we can use the sort_by_key method for an iterator of floats
/// Implements the Ord trait using the f64::total_cmp method
#[derive(PartialEq, PartialOrd)]
pub struct FloatWrapper(f64);

impl Eq for FloatWrapper {}

impl Ord for FloatWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
