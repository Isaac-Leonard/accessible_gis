use std::path::PathBuf;
use std::process;

use optional_struct::{optional_struct, Applyable};

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
