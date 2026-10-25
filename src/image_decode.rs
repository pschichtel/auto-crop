//! Decoding PDF image XObjects into RGB bitmaps.
//!
//! The colours only need to be good enough to tell the paper apart from the scanner
//! background, so colour management, `/Decode` arrays and soft masks are ignored.

use anyhow::{Context, Result, bail};
use image::{ImageFormat, RgbImage};
use lopdf::{Dictionary, Document, Object, Stream};

/// Decode an image XObject stream into an RGB bitmap.
pub fn decode_image(doc: &Document, stream: &Stream) -> Result<RgbImage> {
    let filters = stream.filters().unwrap_or_default();
    let (image_filter, data) = match filters.split_last() {
        Some((&last, leading)) if is_image_filter(last) => {
            (Some(last), apply_filters(stream, leading)?)
        }
        _ => (
            None,
            stream
                .decompressed_content()
                .context("failed to decompress image data")?,
        ),
    };

    match image_filter {
        None => decode_raw(doc, &stream.dict, &data),
        Some(b"DCTDecode") => Ok(
            image::load_from_memory_with_format(&data, ImageFormat::Jpeg)
                .context("failed to decode JPEG image")?
                .to_rgb8(),
        ),
        Some(other) => bail!(
            "unsupported image encoding /{}",
            String::from_utf8_lossy(other)
        ),
    }
}

fn is_image_filter(filter: &[u8]) -> bool {
    matches!(
        filter,
        b"DCTDecode" | b"JPXDecode" | b"CCITTFaxDecode" | b"JBIG2Decode"
    )
}

/// Apply only the given (generic) leading filters of a filter chain.
fn apply_filters(stream: &Stream, filters: &[&[u8]]) -> Result<Vec<u8>> {
    if filters.is_empty() {
        return Ok(stream.content.clone());
    }
    let mut dict = Dictionary::new();
    dict.set(
        "Filter",
        Object::Array(filters.iter().map(|f| Object::Name(f.to_vec())).collect()),
    );
    if let Ok(Object::Array(params)) = stream.dict.get(b"DecodeParms") {
        dict.set(
            "DecodeParms",
            Object::Array(params[..filters.len().min(params.len())].to_vec()),
        );
    }
    Stream::new(dict, stream.content.clone())
        .decompressed_content()
        .context("failed to decompress image data")
}

#[derive(Debug)]
enum ColorSpace {
    Gray,
    Rgb,
    Cmyk,
    /// Any other N-component space; the first component is used as gray.
    Other(usize),
    Indexed {
        base: Box<ColorSpace>,
        lookup: Vec<u8>,
    },
}

impl ColorSpace {
    fn components(&self) -> usize {
        match self {
            ColorSpace::Gray | ColorSpace::Indexed { .. } => 1,
            ColorSpace::Rgb => 3,
            ColorSpace::Cmyk => 4,
            ColorSpace::Other(n) => *n,
        }
    }

    fn parse(doc: &Document, obj: &Object, depth: usize) -> Result<ColorSpace> {
        if depth > 8 {
            bail!("colour space nesting too deep");
        }
        let (_, obj) = doc.dereference(obj)?;
        let name_cs = |name: &[u8]| match name {
            b"DeviceGray" | b"CalGray" | b"G" => ColorSpace::Gray,
            b"DeviceRGB" | b"CalRGB" | b"RGB" => ColorSpace::Rgb,
            b"DeviceCMYK" | b"CMYK" => ColorSpace::Cmyk,
            b"Lab" => ColorSpace::Other(3),
            _ => ColorSpace::Gray,
        };
        match obj {
            Object::Name(name) => Ok(name_cs(name)),
            Object::Array(items) => {
                let family = items.first().and_then(|o| o.as_name().ok()).unwrap_or(b"");
                match family {
                    b"ICCBased" => {
                        let (_, profile) =
                            doc.dereference(items.get(1).context("ICCBased without profile")?)?;
                        let n = profile
                            .as_stream()?
                            .dict
                            .get(b"N")
                            .and_then(Object::as_i64)
                            .unwrap_or(3);
                        Ok(match n {
                            1 => ColorSpace::Gray,
                            4 => ColorSpace::Cmyk,
                            _ => ColorSpace::Rgb,
                        })
                    }
                    b"Indexed" | b"I" => {
                        let base = ColorSpace::parse(
                            doc,
                            items.get(1).context("Indexed without base")?,
                            depth + 1,
                        )?;
                        let (_, lookup) =
                            doc.dereference(items.get(3).context("Indexed without lookup")?)?;
                        let lookup = match lookup {
                            Object::String(bytes, _) => bytes.clone(),
                            Object::Stream(s) => s.decompressed_content()?,
                            _ => bail!("invalid Indexed lookup table"),
                        };
                        Ok(ColorSpace::Indexed {
                            base: Box::new(base),
                            lookup,
                        })
                    }
                    b"Separation" => Ok(ColorSpace::Other(1)),
                    b"DeviceN" => {
                        let (_, names) =
                            doc.dereference(items.get(1).context("DeviceN without names")?)?;
                        Ok(ColorSpace::Other(names.as_array()?.len().max(1)))
                    }
                    b"Lab" => Ok(ColorSpace::Other(3)),
                    other => Ok(name_cs(other)),
                }
            }
            _ => bail!("invalid colour space"),
        }
    }

    /// Convert one pixel's 8-bit components to RGB.
    fn to_rgb(&self, c: &[u8]) -> [u8; 3] {
        match self {
            ColorSpace::Gray | ColorSpace::Other(_) => [c[0]; 3],
            ColorSpace::Rgb => [c[0], c[1], c[2]],
            ColorSpace::Cmyk => {
                let k = c[3] as u16;
                let ch = |v: u8| 255 - (v as u16 + k).min(255) as u8;
                [ch(c[0]), ch(c[1]), ch(c[2])]
            }
            ColorSpace::Indexed { base, lookup } => {
                let n = base.components();
                let start = c[0] as usize * n;
                match lookup.get(start..start + n) {
                    Some(entry) => base.to_rgb(entry),
                    None => [0; 3],
                }
            }
        }
    }
}

/// Decode uncompressed (after generic filters) sample data.
fn decode_raw(doc: &Document, dict: &Dictionary, data: &[u8]) -> Result<RgbImage> {
    let get_int = |key: &[u8]| dict.get_deref(key, doc).and_then(Object::as_i64).ok();
    let width = get_int(b"Width").context("image without /Width")? as usize;
    let height = get_int(b"Height").context("image without /Height")? as usize;
    let image_mask = dict
        .get_deref(b"ImageMask", doc)
        .and_then(Object::as_bool)
        .unwrap_or(false);

    let (color_space, bpc) = if image_mask {
        (ColorSpace::Gray, 1)
    } else {
        let cs = match dict.get(b"ColorSpace") {
            Ok(obj) => ColorSpace::parse(doc, obj, 0)?,
            Err(_) => ColorSpace::Gray,
        };
        (cs, get_int(b"BitsPerComponent").unwrap_or(8) as usize)
    };
    if !matches!(bpc, 1 | 2 | 4 | 8 | 16) {
        bail!("unsupported bits per component: {bpc}");
    }

    let ncomp = color_space.components();
    let stride = (width * ncomp * bpc).div_ceil(8);
    if width == 0 || height == 0 || data.len() < stride * height {
        bail!("image data too short for {width}x{height}");
    }

    let indexed = matches!(color_space, ColorSpace::Indexed { .. });
    let max = (1u32 << bpc.min(8)) - 1;
    let sample = |row: &[u8], i: usize| -> u8 {
        match bpc {
            8 => row[i],
            16 => row[i * 2],
            _ => {
                let bit = i * bpc;
                let v = (row[bit / 8] >> (8 - bpc - bit % 8)) as u32 & max;
                // Palette indices must stay indices; everything else is scaled to 0..=255.
                if indexed {
                    v as u8
                } else {
                    (v * 255 / max) as u8
                }
            }
        }
    };

    let mut img = RgbImage::new(width as u32, height as u32);
    let mut comps = vec![0u8; ncomp];
    for y in 0..height {
        let row = &data[y * stride..(y + 1) * stride];
        for x in 0..width {
            for (c, comp) in comps.iter_mut().enumerate() {
                *comp = sample(row, x * ncomp + c);
            }
            img.put_pixel(x as u32, y as u32, image::Rgb(color_space.to_rgb(&comps)));
        }
    }
    Ok(img)
}
