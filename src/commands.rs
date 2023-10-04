use optional_struct::{optional_struct, Applyable};
use std::{path::PathBuf, process};

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

pub fn list_drivers() -> Result<Vec<String>, String> {
    let mut gdal_info_command = process::Command::new("gdalinfo");
    gdal_info_command.arg("--formats");
    let output = gdal_info_command.output().map_err(|x| x.to_string())?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            // Split by newlines then skip the header
            .map(|output| output.split('\n').skip(1).map(ToOwned::to_owned).collect())
            .map_err(|x| x.to_string())
    } else {
        Err(String::from_utf8(output.stderr).map_err(|x| x.to_string())?)
    }
}
