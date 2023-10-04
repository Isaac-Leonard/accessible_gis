use std::collections::{BTreeMap, BTreeSet};
use std::ops::Index;

use crate::app::BasicApp;
use crate::events::{dispatch_action, Action};

use cacao_framework::{Component, VComponent, VLabel, VList, VNode};
use gdal::vector::{Layer, LayerAccess};

use cacao::listview::{RowAction, RowActionStyle, RowEdge};

use gdal::vector::{Feature, FieldValue, Geometry};

#[derive(Clone, PartialEq)]
pub struct VectorLayerProps {
    pub labeled_by: Option<String>,
    pub common_fields: Vec<(String, Vec<&'static str>)>,
    pub feature_props: Vec<FeatureViewProps>,
}

#[derive(Clone, PartialEq)]
pub struct VectorLayerView;
impl Component for VectorLayerView {
    type Props = VectorLayerProps;
    type State = ();
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        vec![
            (0, VNode::Text("Common fields")),
            (
                1,
                VNode::List(VList {
                    count: props.common_fields.len(),
                    render: render_fields_row,
                }),
            ),
        ]
        .into_iter()
        .chain(
            props
                .feature_props
                .iter()
                .enumerate()
                .map(|(index, feature)| {
                    (
                        index + 10,
                        VNode::Custom(VComponent::new::<FeatureView, BasicApp>(feature.clone())),
                    )
                }),
        )
        .collect()
    }
}

#[derive(Clone, PartialEq)]
pub struct FeatureProps {
    geometry: Geometry,
    fields: Vec<Attribute>,
}

impl<'a> From<Feature<'a>> for FeatureProps {
    fn from(value: Feature) -> Self {
        Self {
            fields: value.fields().collect(),
            geometry: value.geometry().unwrap().clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct FeatureViewProps {
    pub labeled_by: Option<String>,
    pub position: usize,
    pub feature: FeatureProps,
}

#[derive(Clone, PartialEq)]
struct FeatureView;
impl Component for FeatureView {
    type Props = FeatureViewProps;
    type State = ();
    fn render(props: &Self::Props, state: &Self::State) -> Vec<(usize, VNode<Self>)> {
        let labeled_by = props
            .labeled_by
            .clone()
            .and_then(|labeled_by| props.feature.fields.iter().find(|x| x.0 == labeled_by))
            .and_then(|x| x.1.as_ref())
            .map(custom_field_value_to_string)
            .unwrap_or("Unlabeled".to_owned());

        vec![
            (
                0,
                VNode::Label(VLabel {
                    text: format!("{}{}", labeled_by, props.feature.geometry.geometry_name(),),
                }),
            ),
            (
                1,
                VNode::List(VList {
                    count: props.feature.fields.len(),
                    render: render_attribute_row,
                }),
            ),
        ]
    }
}

type Attribute = (String, Option<FieldValue>);

fn render_attribute_row(index: usize, props: &FeatureViewProps, _: &()) -> Vec<VNode<FeatureView>> {
    let (ref key, ref val) = props.feature.fields[index];
    vec![
        VNode::Label(VLabel {
            text: key.to_owned(),
        }),
        VNode::Label({
            VLabel {
                text: val
                    .as_ref()
                    .map(custom_field_value_to_string)
                    .unwrap_or_default(),
            }
        }),
    ]
}

fn custom_field_value_to_string(val: &FieldValue) -> String {
    match val {
        FieldValue::StringValue(str) => str.clone(),
        FieldValue::IntegerValue(int) => int.to_string(),
        FieldValue::RealValue(float) => float.to_string(),
        FieldValue::DateValue(date) => date.to_string(),
        FieldValue::DateTimeValue(date_time) => date_time.to_string(),
        FieldValue::Integer64Value(int) => int.to_string(),
        FieldValue::StringListValue(strings) => format!("[{}]", strings.join(", ")),
        FieldValue::RealListValue(floats) => format!(
            "[{}]",
            floats
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        FieldValue::IntegerListValue(ints) => format!(
            "[{}]",
            ints.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        FieldValue::Integer64ListValue(ints) => format!(
            "[{}]",
            ints.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn custom_field_type_to_string(val: FieldValue) -> &'static str {
    match val {
        FieldValue::StringValue(_) => "String",
        FieldValue::IntegerValue(_) => "Integer",
        FieldValue::RealValue(_) => "Float",
        FieldValue::DateValue(_) => "Date",
        FieldValue::DateTimeValue(_) => "DateTime",
        FieldValue::Integer64Value(_) => "Integer64",
        FieldValue::StringListValue(_) => "StringList",
        FieldValue::RealListValue(_) => "FloatList",
        FieldValue::IntegerListValue(_) => "IntegerList",
        FieldValue::Integer64ListValue(_) => "Integer64List",
    }
}

fn render_fields_row(
    index: usize,
    props: &VectorLayerProps,
    state: &(),
) -> Vec<VNode<VectorLayerView>> {
    let (ref name, ref field_type) = props.common_fields[index];
    vec![
        VNode::Label(VLabel { text: name.clone() }),
        VNode::Label(VLabel {
            text: field_type.join(","),
        }),
    ]
}

fn actions(_row: usize, data: &(String, String), edge: RowEdge) -> Vec<RowAction> {
    eprintln!("actions called");
    if let RowEdge::Leading = edge {
        return vec![];
    }
    let name = data.0.clone();
    vec![RowAction::new(
        "Use as label",
        RowActionStyle::Regular,
        move |_, _| dispatch_action(Action::SetFeatureLabel(name.clone())),
    )]
}

pub fn get_fields(layer: &mut Layer) -> Vec<(String, Vec<&'static str>)> {
    let mut fields: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for feature in layer.features() {
        for (name, val) in feature.fields() {
            let field_type: &'static str = val.map(custom_field_type_to_string).unwrap_or("Empty");
            if let Some(ref mut types) = fields.get_mut(&name) {
                types.insert(field_type);
            } else {
                let mut types = BTreeSet::new();
                types.insert(field_type);
                fields.insert(name.clone(), types);
            }
        }
    }
    fields
        .into_iter()
        .map(|(name, types)| (name, types.into_iter().collect()))
        .collect()
}
