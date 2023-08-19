use cacao::layout::LayoutConstraint;
use cacao::listview::{ListView, ListViewDelegate, RowAction, RowEdge};
use cacao::view::ViewDelegate;

pub trait ConfigurableRow {
    type Data;
    fn configure_with(&mut self, data: &Self::Data);
    fn actions(row: usize, data: &Self::Data, edge: RowEdge) -> Vec<RowAction> {
        Vec::new()
    }
}

/// A generic list view
pub struct MyListView<R: ViewDelegate + ConfigurableRow + Default + 'static> {
    view: Option<ListView>,
    data: Vec<R::Data>,
}

impl<R> MyListView<R>
where
    R: ViewDelegate + ConfigurableRow + Default + 'static,
{
    pub fn new(data: Vec<R::Data>) -> Self {
        Self { view: None, data }
    }

    /// Not a good name
    pub fn with(data: Vec<R::Data>) -> ListView<Self> {
        ListView::with(Self::new(data))
    }
}

impl<R> ListViewDelegate for MyListView<R>
where
    R: ViewDelegate + ConfigurableRow + Default + 'static,
{
    const NAME: &'static str = "ThisIsIgnored";
    fn subclass_name(&self) -> &'static str {
        R::NAME
    }
    /// Essential configuration and retaining of a `ListView` handle to do updates later on.
    fn did_load(&mut self, view: ListView) {
        view.register(R::NAME, R::default);
        view.set_uses_alternating_backgrounds(true);
        view.set_row_height(64.);
        LayoutConstraint::activate(&[
            view.height.constraint_equal_to_constant(50.0),
            view.width.constraint_equal_to_constant(50.0),
        ]);
        self.view = Some(view);
    }

    /// The number of attributes we have.
    fn number_of_items(&self) -> usize {
        self.data.len()
    }

    /// For a given row, dequeues a view from the system and passes the appropriate `Transfer` for
    /// configuration.
    fn item_for(&self, row: usize) -> cacao::listview::ListViewRow {
        let mut view = self.view.as_ref().unwrap().dequeue::<R>(R::NAME);

        if let Some(view) = &mut view.delegate {
            let data = &self.data[row];
            view.configure_with(data);
        }

        view.into_row()
    }

    fn actions_for(&self, row: usize, edge: RowEdge) -> Vec<RowAction> {
        R::actions(row, &self.data[row], edge)
    }
}
