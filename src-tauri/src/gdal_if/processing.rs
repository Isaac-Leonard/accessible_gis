use gdal::{
    errors::GdalError,
    raster::processing::dem::{
        aspect as gdal_aspect, roughness as gdal_roughness, slope as gdal_slope, AspectOptions,
        RoughnessOptions, SlopeOptions,
    },
};

use super::WrappedDataset;
macro_rules! dem_proccessing_function {
    ($gdal_name:ident, $app_name:ident, $options_name:ident) => {
        pub fn $app_name(
            dataset: &WrappedDataset,
            name: String,
            options: &$options_name,
        ) -> Result<WrappedDataset, GdalError> {
            Ok(WrappedDataset::wrap_existing(
                $gdal_name(&dataset.dataset, &name, options)?,
                name,
            ))
        }
    };
}

dem_proccessing_function!(gdal_roughness, roughness, RoughnessOptions);

dem_proccessing_function!(gdal_slope, slope, SlopeOptions);

dem_proccessing_function!(gdal_aspect, aspect, AspectOptions);
