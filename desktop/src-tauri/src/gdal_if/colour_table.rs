use gdal::raster::{
    CmykEntry as GdalCmykEntry, ColorEntry as GdalColourEntry, ColorTable as GdalColourTable,
    GrayEntry as GdalGrayEntry, HlsEntry as GdalHlsEntry, PaletteInterpretation,
    RgbaEntry as GdalRgbaEntry,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// Note: The following code has been generated with an LLM after providing an initial scafold.
// The result has been inspected to ensure correctness

/// Colour table abstraction supporting all GDAL palette types.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
#[serde(tag = "PaletteInterpretation", content = "entries")]
pub enum ColourTable {
    Gray(Vec<GrayEntry>),
    Rgba(Vec<RgbaEntry>),
    Cmyk(Vec<CmykEntry>),
    Hls(Vec<HlsEntry>),
}

impl TryFrom<GdalColourTable<'_>> for ColourTable {
    type Error = ();

    fn try_from(value: GdalColourTable) -> Result<Self, Self::Error> {
        Ok(match value.palette_interpretation() {
            PaletteInterpretation::Gray => ColourTable::Gray(
                (0..value.entry_count())
                    .map(|i| {
                        let entry = value.entry(i).ok_or(())?;
                        GrayEntry::try_from(entry)
                    })
                    .try_collect()?,
            ),
            PaletteInterpretation::Rgba => ColourTable::Rgba(
                (0..value.entry_count())
                    .map(|i| {
                        let entry = value.entry(i).ok_or(())?;
                        RgbaEntry::try_from(entry)
                    })
                    .try_collect()?,
            ),
            PaletteInterpretation::Cmyk => ColourTable::Cmyk(
                (0..value.entry_count())
                    .map(|i| {
                        let entry = value.entry(i).ok_or(())?;
                        CmykEntry::try_from(entry)
                    })
                    .try_collect()?,
            ),
            PaletteInterpretation::Hls => ColourTable::Hls(
                (0..value.entry_count())
                    .map(|i| {
                        let entry = value.entry(i).ok_or(())?;
                        HlsEntry::try_from(entry)
                    })
                    .try_collect()?,
            ),
        })
    }
}

/// Serializable grayscale colour entry.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct GrayEntry {
    pub g: i16,
}

impl From<GdalGrayEntry> for GrayEntry {
    fn from(value: GdalGrayEntry) -> Self {
        Self { g: value.g }
    }
}

impl TryFrom<GdalColourEntry> for GrayEntry {
    type Error = ();
    fn try_from(value: GdalColourEntry) -> Result<Self, Self::Error> {
        match value {
            GdalColourEntry::Gray(entry) => Ok(entry.into()),
            _ => Err(()),
        }
    }
}

/// Serializable RGBA colour entry.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct RgbaEntry {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

impl From<GdalRgbaEntry> for RgbaEntry {
    fn from(value: GdalRgbaEntry) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl TryFrom<GdalColourEntry> for RgbaEntry {
    type Error = ();
    fn try_from(value: GdalColourEntry) -> Result<Self, Self::Error> {
        match value {
            GdalColourEntry::Rgba(entry) => Ok(entry.into()),
            _ => Err(()),
        }
    }
}

/// Serializable CMYK colour entry.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct CmykEntry {
    pub c: i16,
    pub m: i16,
    pub y: i16,
    pub k: i16,
}

impl From<GdalCmykEntry> for CmykEntry {
    fn from(value: GdalCmykEntry) -> Self {
        Self {
            c: value.c,
            m: value.m,
            y: value.y,
            k: value.k,
        }
    }
}

impl TryFrom<GdalColourEntry> for CmykEntry {
    type Error = ();
    fn try_from(value: GdalColourEntry) -> Result<Self, Self::Error> {
        match value {
            GdalColourEntry::Cmyk(entry) => Ok(entry.into()),
            _ => Err(()),
        }
    }
}

/// Serializable HLS colour entry.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct HlsEntry {
    pub h: i16,
    pub l: i16,
    pub s: i16,
}

impl From<GdalHlsEntry> for HlsEntry {
    fn from(value: GdalHlsEntry) -> Self {
        Self {
            h: value.h,
            l: value.l,
            s: value.s,
        }
    }
}

impl TryFrom<GdalColourEntry> for HlsEntry {
    type Error = ();
    fn try_from(value: GdalColourEntry) -> Result<Self, Self::Error> {
        match value {
            GdalColourEntry::Hls(entry) => Ok(entry.into()),
            _ => Err(()),
        }
    }
}

impl From<ColourTable> for GdalColourTable<'_> {
    fn from(value: ColourTable) -> Self {
        match value {
            ColourTable::Gray(entries) => {
                let mut ct = GdalColourTable::new(PaletteInterpretation::Gray);
                for (i, e) in entries.iter().enumerate() {
                    let gdal_entry = GdalColourEntry::Gray(GdalGrayEntry { g: e.g });
                    ct.set_color_entry(i as u16, &gdal_entry);
                }
                ct
            }
            ColourTable::Rgba(entries) => {
                let mut ct = GdalColourTable::new(PaletteInterpretation::Rgba);
                for (i, e) in entries.iter().enumerate() {
                    let gdal_entry = GdalColourEntry::Rgba(GdalRgbaEntry {
                        r: e.r,
                        g: e.g,
                        b: e.b,
                        a: e.a,
                    });
                    ct.set_color_entry(i as u16, &gdal_entry);
                }
                ct
            }
            ColourTable::Cmyk(entries) => {
                let mut ct = GdalColourTable::new(PaletteInterpretation::Cmyk);
                for (i, e) in entries.iter().enumerate() {
                    let gdal_entry = GdalColourEntry::Cmyk(GdalCmykEntry {
                        c: e.c,
                        m: e.m,
                        y: e.y,
                        k: e.k,
                    });
                    ct.set_color_entry(i as u16, &gdal_entry);
                }
                ct
            }
            ColourTable::Hls(entries) => {
                let mut ct = GdalColourTable::new(PaletteInterpretation::Hls);
                for (i, e) in entries.iter().enumerate() {
                    let gdal_entry = GdalColourEntry::Hls(GdalHlsEntry {
                        h: e.h,
                        l: e.l,
                        s: e.s,
                    });
                    ct.set_color_entry(i as u16, &gdal_entry);
                }
                ct
            }
        }
    }
}
