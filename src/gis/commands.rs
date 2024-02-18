use gdal::DriverManager;
use optional_struct::{optional_struct, Applyable};
use std::{
    path::PathBuf,
    process::{self, Command, Output},
};

#[optional_struct]
pub struct WarpSettings {
    pub src: PathBuf,
    pub dest: PathBuf,
}

fn warp(settings: WarpSettings) -> Result<(), ()> {
    let mut gdal_warp_command = process::Command::new("gdalwarp");
    gdal_warp_command.arg(settings.src);
    gdal_warp_command.arg(settings.dest);
    let gdalwarp_output = gdal_warp_command.output();
    eprintln!("{:?}", gdalwarp_output);
    Err(())
}

pub fn list_drivers() -> Vec<String> {
    let mut drivers = Vec::new();
    for i in 0..DriverManager::count() {
        drivers.push(DriverManager::get_driver(i).unwrap().short_name())
    }
    drivers
}

pub fn slope_of_dataset(src: PathBuf, dest: PathBuf) -> std::io::Result<Output> {
    Command::new("gdaldem")
        .arg("slope")
        .arg(src)
        .arg(dest)
        .output()
}

pub fn aspect_of_dataset(src: PathBuf, dest: &PathBuf) -> std::io::Result<Output> {
    Command::new("gdaldem")
        .arg("aspect")
        .arg(src)
        .arg(dest)
        .output()
}
