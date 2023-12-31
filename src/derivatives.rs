use std::{
    f64::{consts::PI, NAN},
    path::PathBuf,
};

use gdal::{raster::Buffer, Dataset};
use ndarray::{Array2, Zip};

use crate::raster::{RasterIndex, RawRasterData};

/// Note that this code has been adapted from existing code from QGIS

const mCellSizeX: f64 = -1.0;
const mCellSizeY: f64 = -1.0;
const mZFactor: f64 = 1.0;

fn calc_x_derivative(cell: &Array2<f64>, no_data_value: f64) -> f64 {
    //the basic formula would be simple, but we need to test for nodata values...
    //return (( (x[((2,0))] - x[(0,0)]) + 2 * (x[(2,1)] - x[(0,1)]) + (x[(2,2)] - x[(0,2)]) ) / (8 * mCellSizeX));

    let mut weight = 0.0;
    let mut sum = 0.;

    //first row
    if cell[(2, 0)] != no_data_value && cell[(0, 0)] != no_data_value
    //the normal case
    {
        sum += cell[(2, 0)] - cell[(0, 0)];
        weight += 2.0;
    } else if cell[(2, 0)] == no_data_value
        && cell[(0, 0)] != no_data_value
        && cell[(1, 0)] != no_data_value
    //probably 3x3 window is at the border
    {
        sum += cell[(1, 0)] - cell[(0, 0)];
        weight += 1.0;
    } else if cell[(0, 0)] == no_data_value
        && cell[(2, 0)] != no_data_value
        && cell[(1, 0)] != no_data_value
    //probably 3x3 window is at the border
    {
        sum += cell[(2, 0)] - cell[(1, 0)];
        weight += 1.0;
    }

    //second row
    if cell[(2, 1)] != no_data_value && cell[(0, 1)] != no_data_value
    //the normal case
    {
        sum += 2.0 * (cell[(2, 1)] - cell[(0, 1)]);
        weight += 4.0;
    } else if cell[(2, 1)] == no_data_value
        && cell[(0, 1)] != no_data_value
        && cell[(1, 1)] != no_data_value
    {
        sum += 2.0 * (cell[(1, 1)] - cell[(0, 1)]);
        weight += 2.0;
    } else if cell[(0, 1)] == no_data_value
        && cell[(2, 1)] != no_data_value
        && cell[(1, 1)] != no_data_value
    {
        sum += 2.0 * (cell[(2, 1)] - cell[(1, 1)]);
        weight += 2.0;
    }

    //third row
    if cell[(2, 2)] != no_data_value && cell[(0, 2)] != no_data_value
    //the normal case
    {
        sum += cell[(2, 2)] - cell[(0, 2)];
        weight += 2.0;
    } else if cell[(2, 2)] == no_data_value
        && cell[(0, 2)] != no_data_value
        && cell[(1, 2)] != no_data_value
    {
        sum += cell[(1, 2)] - cell[(0, 2)];
        weight += 1.0;
    } else if cell[(0, 2)] == no_data_value
        && cell[(2, 2)] != no_data_value
        && cell[(1, 2)] != no_data_value
    {
        sum += cell[(2, 2)] - cell[(1, 2)];
        weight += 1.0;
    }

    if weight == 0.0 {
        return no_data_value;
    }

    sum / (weight * mCellSizeX) * mZFactor
}

fn calc_y_derivative(cell: &Array2<f64>, no_data_value: f64) -> f64 {
    //the basic formula would be simple, but we need to test for nodata values...
    //return (((x[(0,0)] - x[(0,2)]) + 2 * (x[(1,0)] - x[(1,2)]) + (x[((2,0))] - x[(2,2)])) / ( 8 * mCellSizeY));

    let mut sum = 0.;
    let mut weight = 0.0;

    //first row
    if cell[(0, 0)] != no_data_value && cell[(0, 2)] != no_data_value
    //normal case
    {
        sum += cell[(0, 0)] - cell[(0, 2)];
        weight += 2.0;
    } else if cell[(0, 0)] == no_data_value
        && cell[(0, 2)] != no_data_value
        && cell[(0, 1)] != no_data_value
    {
        sum += cell[(0, 1)] - cell[(0, 2)];
        weight += 1.0;
    } else if cell[((2, 0))] == no_data_value
        && cell[(0, 0)] != no_data_value
        && cell[(0, 1)] != no_data_value
    {
        sum += cell[(0, 0)] - cell[(0, 1)];
        weight += 1.0;
    }

    //second row
    if cell[(1, 0)] != no_data_value && cell[(1, 2)] != no_data_value {
        sum += 2.0 * (cell[(1, 0)] - cell[(1, 2)]);
        weight += 4.0;
    } else if cell[(1, 0)] == no_data_value
        && cell[(1, 2)] != no_data_value
        && cell[(1, 1)] != no_data_value
    {
        sum += 2.0 * (cell[(1, 1)] - cell[(1, 2)]);
        weight += 2.0;
    } else if cell[(1, 2)] == no_data_value
        && cell[(1, 0)] != no_data_value
        && cell[(1, 1)] != no_data_value
    {
        sum += 2.0 * (cell[(1, 0)] - cell[(1, 1)]);
        weight += 2.0;
    }

    //third row
    if cell[((2, 0))] != no_data_value && cell[(2, 2)] != no_data_value {
        sum += cell[((2, 0))] - cell[(2, 2)];
        weight += 2.0;
    } else if cell[((2, 0))] == no_data_value
        && cell[(2, 2)] != no_data_value
        && cell[(2, 1)] != no_data_value
    {
        sum += cell[(2, 1)] - cell[(2, 2)];
        weight += 1.0;
    } else if cell[(2, 2)] == no_data_value
        && cell[((2, 0))] != no_data_value
        && cell[(2, 1)] != no_data_value
    {
        sum += cell[((2, 0))] - cell[(2, 1)];
        weight += 1.0;
    }

    if weight == 0.0 {
        return no_data_value;
    }

    sum / (weight * mCellSizeY) * mZFactor
}

fn calc_slope(cell: &Array2<f64>, no_data_value: f64) -> f64 {
    let x = calc_x_derivative(cell, no_data_value);
    let y = calc_y_derivative(cell, no_data_value);
    if x == no_data_value || y == no_data_value {
        no_data_value
    } else {
        (x.powi(2) + y.powi(2)).sqrt().atan() * 180.0 / PI
    }
}

fn slope_of_raster(data: Array2<f64>, no_data_value: f64) -> Array2<f64> {
    Zip::from(data.windows((3, 3))).map_collect(|cell| calc_slope(&cell.to_owned(), no_data_value))
}

pub fn slope_of_dataset(dataset: Dataset, index: RasterIndex, path: &PathBuf) -> Dataset {
    let raster = dataset.rasterband(index.raster as isize).unwrap();
    let data = RawRasterData::new(raster, index);
    let slope = slope_of_raster(data.data, data.no_data_value.unwrap_or(NAN));
    let data = Buffer::new((slope.ncols(), slope.nrows()), slope.into_raw_vec());
    dataset
        .rasterband(index.raster as isize)
        .unwrap()
        .write((0, 0), data.size, &data)
        .unwrap();
    dataset
}

fn aspect_of_raster(data: Array2<f64>, no_data_value: f64) -> Array2<f64> {
    Zip::from(data.windows((3, 3))).map_collect(|cell| calc_slope(&cell.to_owned(), no_data_value))
}
