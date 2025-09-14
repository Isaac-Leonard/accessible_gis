use std::{f64::consts::PI, fmt::Display};

use geo::{Centroid, Coord, CoordsIter, EuclideanDistance, MapCoords, Polygon, Rotate, Simplify};
use serde::{Deserialize, Serialize};

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

    let mut best_score = 1.0;
    let mut delta = 0.0;
    // Currently just test each possible rotation to find matches, the paper outlines a way that this is not necessary but it is unclear and the figures and math are not accessible properly.
    for _ in 0..o.coords_count() * 100 {
        delta += PI / o.coords_count() as f64 / 50.0;
        let o = o.rotate_around_centroid(delta * 180.0 / PI);
        // The first and last points are the same so we must get rid of one of them.
        // We get the exteria ring, which is all we care about, then get the inner vecter and slice out all but the first element.
        // Thandkfully the geo crate has implemented all of their methods on slices of Coords so we don't need to reconstruct polygon objects.
        let o = &o.exterior().0[1..];
        let p = &p.exterior().0[1..];
        let mut correspondence_list = Vec::<(Coord, Coord)>::new();
        let mut outliers = Vec::new();
        for o_i in o.coords_iter() {
            let mut candidates = Vec::new();
            for p_j in p.coords_iter() {
                let theta_o = normalize_angle(o_i.y.atan2(o_i.x));
                let theta_p = normalize_angle(p_j.y.atan2(p_j.x));
                if (theta_p - theta_o).abs() < 4.0 * PI / o.coords_count() as f64 {
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
        let score = mean_distance * 0.5 + max_distance * 0.5;
        if score < best_score {
            best_score = score;
        }
        if best_score == 0.0 {
            return best_score;
        }
    }
    best_score
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
        .unwrap();

    polygon
        .map_coords(|Coord { x, y }| Coord {
            x: x / max_distance,
            y: y / max_distance,
        })
        // We want to remove points that don't contribute to the shapes
        .simplify(&0.05)
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
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize, specta::Type)]
pub struct FloatWrapper(pub f64);

impl Eq for FloatWrapper {}

impl Ord for FloatWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<f64> for FloatWrapper {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<FloatWrapper> for f64 {
    fn from(value: FloatWrapper) -> Self {
        value.0
    }
}

impl Display for FloatWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use geo::{Rotate, polygon};

    use crate::tools::shape_analysis::shape_correspondence;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn identical_shapes() {
        let triangle1 = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let triangle2 = triangle1.clone();

        let score = shape_correspondence(triangle1, triangle2);
        assert!(
            approx_eq(score, 0.0, 1e-6),
            "Identical shapes should have zero distance, got a score of {score} instead"
        );
    }

    #[test]
    fn translated_shapes() {
        let triangle1 = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let triangle2 = polygon![
            (x: 10.0, y: 10.0),
            (x: 11.0, y: 10.0),
            (x: 10.5, y: 11.0),
            (x: 10.0, y: 10.0),
        ];

        let score = shape_correspondence(triangle1, triangle2);
        assert!(
            approx_eq(score, 0.0, 1e-6),
            "Translated shape should match perfectly"
        );
    }

    #[test]
    fn scaled_shapes() {
        let triangle1 = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let triangle2 = polygon![
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 0.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 0.0),
        ];

        let score = shape_correspondence(triangle1, triangle2);
        assert!(
            approx_eq(score, 0.0, 1e-6),
            "Scaled shape should match perfectly, instead got score of {score}"
        );
    }

    #[test]
    fn rotated_shapes() {
        let square1 = polygon![
            (x: -1.0, y: -1.0),
            (x: 1.0, y: -1.0),
            (x: 1.0, y: 1.0),
            (x: -1.0, y: 1.0),
            (x: -1.0, y: -1.0),
        ];
        let square2 = square1.rotate_around_centroid(45.0);

        let score = shape_correspondence(square1, square2);
        assert!(
            score < 0.01,
            "Rotated square should match well, instead got a score of {score}"
        );
    }

    #[test]
    fn noisey_shape() {
        let triangle = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let noisy_triangle = polygon![
            (x: 0.0, y: 0.0),
            (x: 0.3, y: 0.1), // Noise point
            (x: 1.0, y: 0.0),
            (x: 0.75, y: 0.5), // Noise point
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let score = shape_correspondence(triangle, noisy_triangle);
        assert!(
            score < 0.5,
            "Noisy triangle should match fairly well, instead got score of {score}"
        );
    }

    #[test]
    fn different_shapes() {
        let triangle = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 0.5, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let square = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let score = shape_correspondence(triangle, square);
        assert!(
            score > 0.2,
            "Triangle and square should not match perfectly, got score of {score}"
        );
    }
}
