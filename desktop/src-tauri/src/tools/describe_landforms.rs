//! Note this file was largely written with the help of AI though with careful review and much post editing

use core::f64;
use std::collections::HashMap;
use std::path::Path;

use gdal::{Dataset, vector::LayerAccess};
use geo::{BoundingRect, Centroid, ConvexHull, GeodesicArea, LinesIter, Rotate};

use crate::{
    errors::ErrorDetails,
    gdal_if::LayerIndexDiscriminants,
    state::configurable_tools::{Input, InputType, NamedParsedParamValue, Tool, ToolOutputAction},
};

#[derive(Debug)]
struct Entry {
    landform: String,
    area: f64,
    x: f64,
    y: f64,
    orientation_deg: f64,
}

pub fn describe_landforms(path: impl AsRef<Path>) -> Result<String, String> {
    let dataset = Dataset::open(path.as_ref()).map_err(|e| format!("GDAL error: {}", e))?;
    let mut layer = dataset
        .layer(0)
        .map_err(|e| format!("Layer access error: {}", e))?;

    let mut features = vec![];

    let field_index = layer.defn().field_index("landform").unwrap();
    for feature in layer.features() {
        let Some(geom) = feature.geometry() else {
            continue;
        };
        let geo = geom
            .to_geo()
            .map_err(|e| format!("Geometry conversion error: {}", e))?;

        let area = geo.geodesic_area_signed();
        let centroid = geo.centroid().ok_or("Failed to compute centroid")?;
        let (x, y) = (centroid.x(), centroid.y());
        let landform = feature
            .field_as_string(field_index)
            .map_err(|err| format!("Missing landform or label field, error :{err}"))?
            .unwrap();

        // Largely adapted from the minimum_rotated_rect method
        let orientation_deg = {
            let convex_poly = geo.convex_hull();
            let mut min_area = f64::MAX;
            let mut min_angle: f64 = 0.0;
            let rotate_point = convex_poly.centroid().unwrap();
            for line in convex_poly.exterior().lines_iter() {
                let (ci, cii) = line.points();
                let angle = (cii.y() - ci.y()).atan2(cii.x() - ci.x()).to_degrees();
                let rotated_poly = convex_poly.rotate_around_point(-angle, rotate_point);
                let tmp_poly = rotated_poly.bounding_rect().unwrap().to_polygon();
                let area = tmp_poly.geodesic_area_unsigned();
                if area < min_area {
                    min_area = area;
                    min_angle = angle;
                }
            }
            min_angle
        };

        features.push(Entry {
            landform,
            area,
            x,
            y,
            orientation_deg,
        });
    }

    if features.is_empty() {
        return Err("No valid features found.".into());
    }

    // Compute bounds for zoning
    let (x_min, x_max) = features
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), entry| {
            (min.min(entry.x), max.max(entry.x))
        });

    let (y_min, y_max) = features
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), entry| {
            (min.min(entry.y), max.max(entry.y))
        });

    let x_third = (x_max - x_min) / 3.0;
    let y_third = (y_max - y_min) / 3.0;

    fn get_zone(x: f64, y: f64, x_min: f64, y_min: f64, x_third: f64, y_third: f64) -> String {
        let x_zone = if x < x_min + x_third {
            "West"
        } else if x > x_min + 2.0 * x_third {
            "East"
        } else {
            "Central"
        };

        let y_zone = if y > y_min + 2.0 * y_third {
            "North"
        } else if y < y_min + y_third {
            "South"
        } else {
            "Central"
        };

        format!("{}-{}", y_zone, x_zone)
    }

    fn size_description(area_m2: f64) -> &'static str {
        let km2 = area_m2 / 1_000_000.0;
        if km2 < 0.5 {
            "small"
        } else if km2 < 2.0 {
            "moderate-sized"
        } else {
            "large"
        }
    }

    fn orientation_to_compass(deg: f64) -> &'static str {
        let angle = ((deg + 360.0) % 180.0).round(); // mirror for bidirectionality
        match angle {
            a if !(22.5..157.5).contains(&a) => "east-west",
            a if a < 67.5 => "northeast-southwest",
            a if a < 112.5 => "north-south",
            _ => "northwest-southeast",
        }
    }

    #[derive(Default)]
    struct Aggregate {
        area: f64,
        orientation_sum: f64,
        orientation_weight: f64,
    }

    let mut zone_data: HashMap<String, HashMap<String, Aggregate>> = HashMap::new();

    for f in features {
        let zone = get_zone(f.x, f.y, x_min, y_min, x_third, y_third);
        let entry = zone_data
            .entry(zone)
            .or_default()
            .entry(f.landform.clone())
            .or_default();

        entry.area += f.area;
        entry.orientation_sum += f.orientation_deg * f.area;
        entry.orientation_weight += f.area;
    }

    // Build descriptions
    let mut output = String::new();

    let mut zones: Vec<_> = zone_data.keys().collect();
    zones.sort();

    for zone in zones {
        let mut forms: Vec<_> = zone_data[zone]
            .iter()
            .map(|(landform, agg)| {
                let avg_orientation = agg.orientation_sum / agg.orientation_weight;
                (landform, agg.area, avg_orientation)
            })
            .collect();

        forms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top = forms.iter().take(3);

        let mut phrases = vec![];
        for (landform, area, orientation) in top {
            let size = size_description(*area);
            let orientation_phrase = orientation_to_compass(*orientation);
            let desc = format!(
                "{} {}s oriented roughly {}",
                size,
                landform.to_lowercase(),
                orientation_phrase
            );
            phrases.push(desc);
        }

        if !phrases.is_empty() {
            output += &format!(
                "The {} region contains mostly {}.\n",
                zone.to_lowercase(),
                phrases.join(" and ")
            );
        }
    }

    Ok(output.trim().to_string())
}

#[derive(Clone, Debug)]
pub struct DescribeLandformsTool;

impl Tool for DescribeLandformsTool {
    fn get_label(&self) -> String {
        "Describe landforms".to_string()
    }

    fn get_expected_input_parameters(&self) -> Vec<Input> {
        vec![Input {
            label: "Landforms layer".to_string(),
            name: None,
            param_type: InputType::Layer(LayerIndexDiscriminants::Vector),
        }]
    }

    fn execute(
        &self,
        mut params: Vec<NamedParsedParamValue>,
    ) -> Result<ToolOutputAction, ErrorDetails> {
        let index = params.remove(0);
        let layer = index.try_as_raw().unwrap().try_as_vector().unwrap();
        let result =
            describe_landforms(&layer.info.shared.name).map_err(|err| ErrorDetails::Other(err))?;
        Ok(ToolOutputAction::Alert(result))
    }

    fn dyn_clone(&self) -> Box<dyn Tool> {
        Box::new(Self)
    }
}
