use serde::{Deserialize, Serialize};

use super::{raster::WrappedRasterBand, vector::WrappedLayer};

pub enum LayerEnum<'a> {
    Layer(WrappedLayer<'a>),
    Band(WrappedRasterBand<'a>),
}

impl<'a> From<WrappedRasterBand<'a>> for LayerEnum<'a> {
    fn from(v: WrappedRasterBand<'a>) -> Self {
        Self::Band(v)
    }
}

impl<'a> From<WrappedLayer<'a>> for LayerEnum<'a> {
    fn from(v: WrappedLayer<'a>) -> Self {
        Self::Layer(v)
    }
}

impl<'a> LayerEnum<'a> {
    pub fn as_layer(&self) -> Option<&WrappedLayer<'a>> {
        if let Self::Layer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_layer_mut(&mut self) -> Option<&mut WrappedLayer<'a>> {
        if let Self::Layer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_band(&self) -> Option<&WrappedRasterBand<'a>> {
        if let Self::Band(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn try_into_layer(self) -> Result<WrappedLayer<'a>, Self> {
        if let Self::Layer(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    /// Returns `true` if the layer enum is [`Band`].
    ///
    /// [`Band`]: LayerEnum::Band
    #[must_use]
    pub fn is_band(&self) -> bool {
        matches!(self, Self::Band(..))
    }

    /// Returns `true` if the layer enum is [`Layer`].
    ///
    /// [`Layer`]: LayerEnum::Layer
    #[must_use]
    pub fn is_layer(&self) -> bool {
        matches!(self, Self::Layer(..))
    }

    pub fn try_into_band(self) -> Result<WrappedRasterBand<'a>, Self> {
        if let Self::Band(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Debug, specta::Type)]
#[serde(tag = "type", content = "index")]
pub enum LayerIndex {
    Vector(usize),
    Raster(usize),
}

impl LayerIndex {
    pub fn as_vector(&self) -> Option<&usize> {
        if let Self::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_raster(&self) -> Option<&usize> {
        if let Self::Raster(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the layer index is [`Vector`].
    ///
    /// [`Vector`]: LayerIndex::Vector
    #[must_use]
    pub fn is_vector(&self) -> bool {
        matches!(self, Self::Vector(..))
    }

    /// Returns `true` if the layer index is [`Raster`].
    ///
    /// [`Raster`]: LayerIndex::Raster
    #[must_use]
    pub fn is_raster(&self) -> bool {
        matches!(self, Self::Raster(..))
    }
}
