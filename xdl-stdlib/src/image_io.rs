//! Image I/O functions for XDL
//!
//! This module provides functions for reading and writing various image formats.
//! Requires the `image-io` feature to be enabled.

use xdl_core::{XdlError, XdlResult, XdlValue};

#[cfg(feature = "image-io")]
use image::{DynamicImage, GenericImageView, ImageFormat, Rgb, Rgba, Luma, GrayImage, RgbImage, RgbaImage};

/// READ_PNG - Read a PNG image file
/// Returns a 2D or 3D array (height x width x channels)
#[cfg(feature = "image-io")]
pub fn read_png(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_PNG requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_PNG: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_PNG: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_png(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_PNG requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_PNG - Write an array to a PNG image file
#[cfg(feature = "image-io")]
pub fn write_png(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_PNG requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_PNG: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save_with_format(&filename, ImageFormat::Png)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_PNG: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_png(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_PNG requires the 'image-io' feature to be enabled".to_string()))
}

/// READ_JPEG - Read a JPEG image file
#[cfg(feature = "image-io")]
pub fn read_jpeg(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_JPEG requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_JPEG: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_JPEG: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_jpeg(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_JPEG requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_JPEG - Write an array to a JPEG image file
/// Optional quality parameter (1-100, default 90)
#[cfg(feature = "image-io")]
pub fn write_jpeg(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_JPEG requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_JPEG: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save_with_format(&filename, ImageFormat::Jpeg)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_JPEG: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_jpeg(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_JPEG requires the 'image-io' feature to be enabled".to_string()))
}

/// READ_TIFF - Read a TIFF image file
#[cfg(feature = "image-io")]
pub fn read_tiff(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_TIFF requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_TIFF: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_TIFF: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_tiff(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_TIFF requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_TIFF - Write an array to a TIFF image file
#[cfg(feature = "image-io")]
pub fn write_tiff(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_TIFF requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_TIFF: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save_with_format(&filename, ImageFormat::Tiff)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_TIFF: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_tiff(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_TIFF requires the 'image-io' feature to be enabled".to_string()))
}

/// READ_BMP - Read a BMP image file
#[cfg(feature = "image-io")]
pub fn read_bmp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_BMP requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_BMP: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_BMP: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_bmp(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_BMP requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_BMP - Write an array to a BMP image file
#[cfg(feature = "image-io")]
pub fn write_bmp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_BMP requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_BMP: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save_with_format(&filename, ImageFormat::Bmp)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_BMP: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_bmp(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_BMP requires the 'image-io' feature to be enabled".to_string()))
}

/// READ_GIF - Read a GIF image file (first frame only)
#[cfg(feature = "image-io")]
pub fn read_gif(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_GIF requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_GIF: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_GIF: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_gif(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_GIF requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_GIF - Write an array to a GIF image file
#[cfg(feature = "image-io")]
pub fn write_gif(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_GIF requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_GIF: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save_with_format(&filename, ImageFormat::Gif)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_GIF: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_gif(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_GIF requires the 'image-io' feature to be enabled".to_string()))
}

/// READ_IMAGE - Read any supported image format (auto-detect)
#[cfg(feature = "image-io")]
pub fn read_image(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("READ_IMAGE requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("READ_IMAGE: filename must be a string".to_string())),
    };

    let img = image::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("READ_IMAGE: failed to read '{}': {}", filename, e)))?;

    image_to_xdl_value(img)
}

#[cfg(not(feature = "image-io"))]
pub fn read_image(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("READ_IMAGE requires the 'image-io' feature to be enabled".to_string()))
}

/// WRITE_IMAGE - Write to any supported format (auto-detect from extension)
#[cfg(feature = "image-io")]
pub fn write_image(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("WRITE_IMAGE requires filename and image data".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("WRITE_IMAGE: filename must be a string".to_string())),
    };

    let img = xdl_value_to_image(&args[1])?;
    img.save(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("WRITE_IMAGE: failed to write '{}': {}", filename, e)))?;

    Ok(XdlValue::Int(1))
}

#[cfg(not(feature = "image-io"))]
pub fn write_image(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("WRITE_IMAGE requires the 'image-io' feature to be enabled".to_string()))
}

/// QUERY_IMAGE - Get image dimensions and format without loading full data
#[cfg(feature = "image-io")]
pub fn query_image(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("QUERY_IMAGE requires a filename".to_string()));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("QUERY_IMAGE: filename must be a string".to_string())),
    };

    let reader = image::io::Reader::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("QUERY_IMAGE: failed to open '{}': {}", filename, e)))?;

    let format = reader.format()
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|| "Unknown".to_string());

    let reader = reader.with_guessed_format()
        .map_err(|e| XdlError::RuntimeError(format!("QUERY_IMAGE: failed to guess format: {}", e)))?;

    let (width, height) = reader.into_dimensions()
        .map_err(|e| XdlError::RuntimeError(format!("QUERY_IMAGE: failed to read dimensions: {}", e)))?;

    // Return struct-like result: [width, height, format_string]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::Long(width as i64),
        XdlValue::Long(height as i64),
        XdlValue::String(format),
    ]))
}

#[cfg(not(feature = "image-io"))]
pub fn query_image(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError("QUERY_IMAGE requires the 'image-io' feature to be enabled".to_string()))
}

/// TV - Display image (placeholder - returns image dimensions)
/// In a full implementation, this would display the image in a window
pub fn tv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("TV requires image data".to_string()));
    }

    // Get image dimensions
    match &args[0] {
        XdlValue::Array(arr) => {
            // 1D array
            println!("TV: Displaying 1D array of {} elements", arr.len());
            Ok(XdlValue::Int(1))
        }
        XdlValue::NestedArray(rows) => {
            if rows.is_empty() {
                return Err(XdlError::RuntimeError("TV: empty array".to_string()));
            }
            // Check if 2D (grayscale) or 3D (color)
            match &rows[0] {
                XdlValue::Array(row) => {
                    let height = rows.len();
                    let width = row.len();
                    println!("TV: Displaying {}x{} image", width, height);
                    Ok(XdlValue::Int(1))
                }
                XdlValue::NestedArray(row) => {
                    let height = rows.len();
                    let width = row.len();
                    println!("TV: Displaying {}x{} color image", width, height);
                    Ok(XdlValue::Int(1))
                }
                _ => {
                    println!("TV: Displaying nested array of {} elements", rows.len());
                    Ok(XdlValue::Int(1))
                }
            }
        }
        _ => Err(XdlError::RuntimeError("TV: requires array input".to_string())),
    }
}

/// TVSCL - Display scaled image (placeholder)
pub fn tvscl(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("TVSCL requires image data".to_string()));
    }

    // Same as TV but with auto-scaling
    tv(args)
}

// Helper functions for image conversion

#[cfg(feature = "image-io")]
fn image_to_xdl_value(img: DynamicImage) -> XdlResult<XdlValue> {
    let (width, height) = img.dimensions();

    match img {
        DynamicImage::ImageLuma8(gray) => {
            // Grayscale: return 2D array [height][width] as floats
            let mut rows: Vec<XdlValue> = Vec::with_capacity(height as usize);
            for y in 0..height {
                let mut row: Vec<f64> = Vec::with_capacity(width as usize);
                for x in 0..width {
                    let pixel = gray.get_pixel(x, y);
                    row.push(pixel[0] as f64);
                }
                rows.push(XdlValue::Array(row));
            }
            Ok(XdlValue::NestedArray(rows))
        }
        DynamicImage::ImageRgb8(rgb) => {
            // RGB: return 3D array [height][width][3]
            let mut rows: Vec<XdlValue> = Vec::with_capacity(height as usize);
            for y in 0..height {
                let mut row: Vec<XdlValue> = Vec::with_capacity(width as usize);
                for x in 0..width {
                    let pixel = rgb.get_pixel(x, y);
                    row.push(XdlValue::Array(vec![
                        pixel[0] as f64,
                        pixel[1] as f64,
                        pixel[2] as f64,
                    ]));
                }
                rows.push(XdlValue::NestedArray(row));
            }
            Ok(XdlValue::NestedArray(rows))
        }
        DynamicImage::ImageRgba8(rgba) => {
            // RGBA: return 3D array [height][width][4]
            let mut rows: Vec<XdlValue> = Vec::with_capacity(height as usize);
            for y in 0..height {
                let mut row: Vec<XdlValue> = Vec::with_capacity(width as usize);
                for x in 0..width {
                    let pixel = rgba.get_pixel(x, y);
                    row.push(XdlValue::Array(vec![
                        pixel[0] as f64,
                        pixel[1] as f64,
                        pixel[2] as f64,
                        pixel[3] as f64,
                    ]));
                }
                rows.push(XdlValue::NestedArray(row));
            }
            Ok(XdlValue::NestedArray(rows))
        }
        _ => {
            // Convert to RGB8 for other formats
            let rgb = img.to_rgb8();
            let mut rows: Vec<XdlValue> = Vec::with_capacity(height as usize);
            for y in 0..height {
                let mut row: Vec<XdlValue> = Vec::with_capacity(width as usize);
                for x in 0..width {
                    let pixel = rgb.get_pixel(x, y);
                    row.push(XdlValue::Array(vec![
                        pixel[0] as f64,
                        pixel[1] as f64,
                        pixel[2] as f64,
                    ]));
                }
                rows.push(XdlValue::NestedArray(row));
            }
            Ok(XdlValue::NestedArray(rows))
        }
    }
}

#[cfg(feature = "image-io")]
fn xdl_value_to_image(value: &XdlValue) -> XdlResult<DynamicImage> {
    match value {
        XdlValue::NestedArray(rows) => {
            if rows.is_empty() {
                return Err(XdlError::RuntimeError("Empty image array".to_string()));
            }

            let height = rows.len();

            // Check first row to determine image type
            match &rows[0] {
                XdlValue::Array(first_row) => {
                    // Grayscale image: each row is Vec<f64>
                    let width = first_row.len();
                    let mut img = GrayImage::new(width as u32, height as u32);
                    for (y, row) in rows.iter().enumerate() {
                        if let XdlValue::Array(row_arr) = row {
                            for (x, &px) in row_arr.iter().enumerate() {
                                let gray = (px.clamp(0.0, 255.0)) as u8;
                                img.put_pixel(x as u32, y as u32, Luma([gray]));
                            }
                        }
                    }
                    Ok(DynamicImage::ImageLuma8(img))
                }
                XdlValue::NestedArray(first_row) => {
                    // Color image: each row is Vec<XdlValue> where each pixel is Array
                    if first_row.is_empty() {
                        return Err(XdlError::RuntimeError("Empty image row".to_string()));
                    }

                    let width = first_row.len();

                    // Check if RGB or RGBA
                    let channels = match &first_row[0] {
                        XdlValue::Array(pixel) => pixel.len(),
                        _ => 3, // Default to RGB
                    };

                    if channels == 3 {
                        // RGB
                        let mut img = RgbImage::new(width as u32, height as u32);
                        for (y, row) in rows.iter().enumerate() {
                            if let XdlValue::NestedArray(row_arr) = row {
                                for (x, px) in row_arr.iter().enumerate() {
                                    if let XdlValue::Array(pixel_arr) = px {
                                        let r = pixel_arr.get(0).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        let g = pixel_arr.get(1).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        let b = pixel_arr.get(2).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                                    }
                                }
                            }
                        }
                        Ok(DynamicImage::ImageRgb8(img))
                    } else if channels >= 4 {
                        // RGBA
                        let mut img = RgbaImage::new(width as u32, height as u32);
                        for (y, row) in rows.iter().enumerate() {
                            if let XdlValue::NestedArray(row_arr) = row {
                                for (x, px) in row_arr.iter().enumerate() {
                                    if let XdlValue::Array(pixel_arr) = px {
                                        let r = pixel_arr.get(0).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        let g = pixel_arr.get(1).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        let b = pixel_arr.get(2).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                                        let a = pixel_arr.get(3).copied().unwrap_or(255.0).clamp(0.0, 255.0) as u8;
                                        img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
                                    }
                                }
                            }
                        }
                        Ok(DynamicImage::ImageRgba8(img))
                    } else {
                        Err(XdlError::RuntimeError(format!("Unsupported channel count: {}", channels)))
                    }
                }
                _ => Err(XdlError::RuntimeError("Invalid image format: expected 2D array".to_string())),
            }
        }
        XdlValue::Array(data) => {
            // 1D array - treat as single row grayscale
            let width = data.len();
            let mut img = GrayImage::new(width as u32, 1);
            for (x, &px) in data.iter().enumerate() {
                let gray = (px.clamp(0.0, 255.0)) as u8;
                img.put_pixel(x as u32, 0, Luma([gray]));
            }
            Ok(DynamicImage::ImageLuma8(img))
        }
        _ => Err(XdlError::RuntimeError("Image must be an array".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tv_placeholder() {
        let arr = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 128.0, 255.0]),
            XdlValue::Array(vec![64.0, 192.0, 32.0]),
        ]);
        let result = tv(&[arr]);
        assert!(result.is_ok());
    }
}
