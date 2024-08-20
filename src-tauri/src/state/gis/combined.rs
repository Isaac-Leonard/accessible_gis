use super::{raster::StatefulRasterBand, shared::SharedInfo, vector::StatefulVectorLayer};

pub enum StatefulLayerEnum<'a> {
    Raster(StatefulRasterBand<'a>),
    Vector(StatefulVectorLayer<'a>),
}

impl<'a> From<StatefulVectorLayer<'a>> for StatefulLayerEnum<'a> {
    fn from(v: StatefulVectorLayer<'a>) -> Self {
        Self::Vector(v)
    }
}

impl<'a> From<StatefulRasterBand<'a>> for StatefulLayerEnum<'a> {
    fn from(v: StatefulRasterBand<'a>) -> Self {
        Self::Raster(v)
    }
}

impl<'a> StatefulLayerEnum<'a> {
    pub fn shared_mut(&mut self) -> &mut SharedInfo {
        match self {
            Self::Vector(layer) => &mut layer.info.shared,
            Self::Raster(band) => &mut band.info.shared,
        }
    }

    pub fn as_vector(&self) -> Option<&StatefulVectorLayer> {
        if let Self::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_raster(&self) -> Option<&StatefulRasterBand> {
        if let Self::Raster(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn try_into_raster(self) -> Result<StatefulRasterBand<'a>, Self> {
        if let Self::Raster(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_vector(self) -> Result<StatefulVectorLayer<'a>, Self> {
        if let Self::Vector(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}
