use std::{path::Path, process::Command};

use super::WrappedDataset;

pub fn gdal_dem(
    mode: &'static str,
    dataset: &WrappedDataset,
    name: impl AsRef<Path>,
) -> Result<WrappedDataset, String> {
    let mut command = Command::new("gdaldem");
    command.arg(mode).arg(&dataset.file_name).arg(name.as_ref());
    let output = command.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        WrappedDataset::open(name)
    }
}

pub fn hillshade(
    dataset: &WrappedDataset,
    name: impl AsRef<Path>,
) -> Result<WrappedDataset, String> {
    gdal_dem("hillshade", dataset, name)
}

pub fn slope(dataset: &WrappedDataset, name: impl AsRef<Path>) -> Result<WrappedDataset, String> {
    gdal_dem("slope", dataset, name)
}

pub fn aspect(dataset: &WrappedDataset, name: impl AsRef<Path>) -> Result<WrappedDataset, String> {
    gdal_dem("aspect", dataset, name)
}

pub fn color_relief(
    dataset: &WrappedDataset,
    name: impl AsRef<Path>,
) -> Result<WrappedDataset, String> {
    gdal_dem("color-relief", dataset, name)
}

pub fn tri(dataset: &WrappedDataset, name: impl AsRef<Path>) -> Result<WrappedDataset, String> {
    gdal_dem("tri", dataset, name)
}

pub fn tpi(dataset: &WrappedDataset, name: impl AsRef<Path>) -> Result<WrappedDataset, String> {
    gdal_dem("tpi", dataset, name)
}

pub fn roughness(
    dataset: &WrappedDataset,
    name: impl AsRef<Path>,
) -> Result<WrappedDataset, String> {
    gdal_dem("roughness", dataset, name)
}
