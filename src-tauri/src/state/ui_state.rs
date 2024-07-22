pub enum RootScreen {
    Settings,
    Layers(Option<LayerScreen>),
    NewDataset,
}

pub enum LayerScreen {
    Raster(RasterScreen),
    Vector(VectorScreen),
}

pub enum RasterScreen {}

pub enum VectorScreen {}
