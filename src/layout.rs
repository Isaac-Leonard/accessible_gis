use cacao::{
    button::Button,
    input::TextField,
    layout::{LayoutAnchorX, LayoutAnchorY, LayoutConstraint, SafeAreaLayoutGuide},
    listview::ListView,
    text::Label,
    view::View,
};

/// Takes a list of views, a parent view that  contains them and returns layout constraints that will position them from top to bottom separated by the specified padding.
/// The padding is also applied to the sides of each view.
pub fn top_to_bottom(
    views: Vec<&dyn HasLayout>,
    parent: &impl HasLayout,
    padding: f32,
) -> Vec<LayoutConstraint> {
    let (top, bottom) = if let (Some(first), Some(last)) = (views.first(), views.last()) {
        (
            first
                .get_top()
                .constraint_equal_to(parent.get_top())
                .offset(padding),
            last.get_bottom()
                .constraint_equal_to(parent.get_bottom())
                .offset(padding),
        )
    } else {
        // No views were passed
        return Vec::new();
    };
    let adjoining_constraints = views
        .array_windows::<2>()
        .map(|[a, b]| a.get_bottom().constraint_equal_to(b.get_top()));
    let side_constraints = views
        .iter()
        .map(|view| {
            [view
                .get_leading()
                .constraint_equal_to(parent.get_leading())
                .offset(padding)]
        })
        .flatten();
    vec![top, bottom]
        .into_iter()
        .chain(adjoining_constraints)
        .chain(side_constraints)
        .collect()
}

/// Returns a list of layout constraints that makes the given view fill the given safe area
pub fn fill_safe_area(
    view: &impl HasLayout,
    safe_area: &SafeAreaLayoutGuide,
) -> Vec<LayoutConstraint> {
    vec![
        view.get_top().constraint_equal_to(&safe_area.top),
        view.get_bottom().constraint_equal_to(&safe_area.bottom),
        view.get_leading().constraint_equal_to(&safe_area.leading),
        view.get_trailing().constraint_equal_to(&safe_area.trailing),
    ]
}

/// A trait to access a views layout anchors
pub trait HasLayout {
    fn get_top(&self) -> &LayoutAnchorY;
    fn get_bottom(&self) -> &LayoutAnchorY;
    fn get_leading(&self) -> &LayoutAnchorX;
    fn get_trailing(&self) -> &LayoutAnchorX;
}

impl HasLayout for Label {
    fn get_top(&self) -> &LayoutAnchorY {
        &self.top
    }
    fn get_bottom(&self) -> &LayoutAnchorY {
        &self.bottom
    }
    fn get_leading(&self) -> &LayoutAnchorX {
        &self.leading
    }
    fn get_trailing(&self) -> &LayoutAnchorX {
        &self.trailing
    }
}
impl HasLayout for TextField {
    fn get_top(&self) -> &LayoutAnchorY {
        &self.top
    }
    fn get_bottom(&self) -> &LayoutAnchorY {
        &self.bottom
    }
    fn get_leading(&self) -> &LayoutAnchorX {
        &self.leading
    }
    fn get_trailing(&self) -> &LayoutAnchorX {
        &self.trailing
    }
}
impl HasLayout for Button {
    fn get_top(&self) -> &LayoutAnchorY {
        &self.top
    }
    fn get_bottom(&self) -> &LayoutAnchorY {
        &self.bottom
    }
    fn get_leading(&self) -> &LayoutAnchorX {
        &self.leading
    }
    fn get_trailing(&self) -> &LayoutAnchorX {
        &self.trailing
    }
}
impl<T> HasLayout for View<T> {
    fn get_top(&self) -> &LayoutAnchorY {
        &self.top
    }
    fn get_bottom(&self) -> &LayoutAnchorY {
        &self.bottom
    }
    fn get_leading(&self) -> &LayoutAnchorX {
        &self.leading
    }
    fn get_trailing(&self) -> &LayoutAnchorX {
        &self.trailing
    }
}
impl<T> HasLayout for ListView<T> {
    fn get_top(&self) -> &LayoutAnchorY {
        &self.top
    }
    fn get_bottom(&self) -> &LayoutAnchorY {
        &self.bottom
    }
    fn get_leading(&self) -> &LayoutAnchorX {
        &self.leading
    }
    fn get_trailing(&self) -> &LayoutAnchorX {
        &self.trailing
    }
}
