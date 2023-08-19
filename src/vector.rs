use crate::list_view::{ConfigurableRow, MyListView};
use gdal::vector::{Layer, LayerAccess};

use cacao::layout::{Layout, LayoutConstraint};
use cacao::listview::ListView;

use cacao::text::Label;
use cacao::view;
use cacao::view::{View, ViewDelegate};
use gdal::vector::{Feature, FieldValue, Geometry};
pub struct VectorLayerView {
    content: View,
    common_fields: ListView<MyListView<CommonFieldsRow>>,
    feature_views: Vec<View<FeatureView>>,
}
impl VectorLayerView {
    pub fn new(layer: Layer) -> Self {
        let mut layer = layer;
        let mut names = Vec::new();
        for feature in layer.features() {
            for field in feature.fields() {
                names.push(field.0.clone())
            }
        }
        Self {
            content: View::new(),
            common_fields: ListView::with(MyListView::new(names)),
            feature_views: layer
                .features()
                .enumerate()
                .map(|x| View::with(FeatureView::new(x.1, x.0)))
                .collect(),
        }
    }
}

impl ViewDelegate for VectorLayerView {
    const NAME: &'static str = "VectorLayerView";

    fn did_load(&mut self, view: View) {
        self.content.add_subview(&self.common_fields);
        for view in &self.feature_views {
            self.content.add_subview(view)
        }
        view.add_subview(&self.content);

        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
        ])
    }
}

struct FeatureView {
    pub content: view::View,
    label: Label,
    attribute_table: ListView<MyListView<AttributeViewRow>>,
    position: usize,
    projection_label: Label,
    geometry: Geometry,
}

impl FeatureView {
    pub fn new(feature: Feature, position: usize) -> Self {
        FeatureView {
            content: View::new(),
            label: Label::default(),
            attribute_table: ListView::with(MyListView::new(feature.fields().collect())),
            position,
            projection_label: Label::new(),
            geometry: feature.geometry().unwrap().clone(),
        }
    }
}

impl ViewDelegate for FeatureView {
    const NAME: &'static str = "VectorView";

    fn did_load(&mut self, view: View) {
        self.content.add_subview(&self.attribute_table);
        self.label.set_text(self.geometry.geometry_name());
        self.label
            .set_text_color(cacao::color::Color::rgb(255, 255, 255));
        self.content.add_subview(&self.label);

        self.content.add_subview(&self.projection_label);
        view.add_subview(&self.content);
        // Add layout constraints to be 100% excluding the safe area
        // Do last because it will crash because the view needs to be inside the hierarchy
        LayoutConstraint::activate(&[
            self.content
                .top
                .constraint_equal_to(&view.safe_layout_guide.top)
                .offset(self.position as f64 * 50.),
            self.content
                .leading
                .constraint_equal_to(&view.safe_layout_guide.leading),
            self.content
                .trailing
                .constraint_equal_to(&view.safe_layout_guide.trailing),
            self.content
                .bottom
                .constraint_equal_to(&view.safe_layout_guide.bottom),
        ])
    }
}

#[derive(Default, Debug)]
pub struct AttributeViewRow {
    pub key: Label,
    pub value: Label,
}

type Attribute = (String, Option<FieldValue>);

impl ConfigurableRow for AttributeViewRow {
    type Data = Attribute;
    /// Called when this view is being presented, and configures itself     pub
    fn configure_with(&mut self, (key, val): &Attribute) {
        self.key.set_text(key);
        self.value.set_text(format!(
            "{}",
            val.clone()
                .map(custom_field_value_to_string)
                .unwrap_or_default()
        ));
    }
}

impl ViewDelegate for AttributeViewRow {
    const NAME: &'static str = "AttributeViewRow";

    /// Called when the view is first created; handles setup of layout and associated styling that
    /// doesn't change.
    fn did_load(&mut self, view: View) {
        view.add_subview(&self.key);
        view.add_subview(&self.value);

        LayoutConstraint::activate(&[
            self.key.top.constraint_equal_to(&view.top).offset(16.),
            self.key
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.key
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.value
                .top
                .constraint_equal_to(&self.key.bottom)
                .offset(8.),
            self.value
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.value
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.value
                .bottom
                .constraint_equal_to(&view.bottom)
                .offset(-16.),
        ]);
    }
}

fn custom_field_value_to_string(val: FieldValue) -> String {
    match val {
        FieldValue::StringValue(str) => str,
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

#[derive(Default, Debug)]
pub struct CommonFieldsRow {
    name: Label,
}

impl ConfigurableRow for CommonFieldsRow {
    type Data = String;
    /// Called when this view is being presented, and configures itself     pub
    fn configure_with(&mut self, name: &String) {
        self.name.set_text(name);
    }
}

impl ViewDelegate for CommonFieldsRow {
    const NAME: &'static str = "SharedFieldsRow";

    /// Called when the view is first created; handles setup of layout and associated styling that
    /// doesn't change.
    fn did_load(&mut self, view: View) {
        view.add_subview(&self.name);

        LayoutConstraint::activate(&[
            self.name.top.constraint_equal_to(&view.top).offset(16.),
            self.name
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.name
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
        ]);
    }
}
