//! Scientific data format I/O functions
//!
//! This module provides placeholder implementations for scientific data formats:
//! - FITS (Flexible Image Transport System) - Astronomy
//! - HDF5 (Hierarchical Data Format) - General scientific data
//! - NetCDF (Network Common Data Form) - Climate/atmospheric science
//!
//! Note: Full implementations would require native libraries.
//! These placeholders provide API compatibility and informative messages.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use xdl_core::{XdlError, XdlResult, XdlValue};

// ============================================================================
// FITS (Flexible Image Transport System) Functions
// ============================================================================

/// READFITS - Read a FITS file
/// IDL syntax: result = READFITS(filename [, header] [, /NOSCALE])
pub fn readfits(args: &[XdlValue], _keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "READFITS: Expected filename argument".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Check if file exists and has FITS signature
    let path = Path::new(&filename);
    if !path.exists() {
        return Err(XdlError::FileNotFound(filename));
    }

    // Try to read first few bytes to verify FITS format
    let mut file = File::open(path).map_err(|e| XdlError::IoError(e.to_string()))?;
    let mut header = vec![0u8; 80];
    file.read_exact(&mut header)
        .map_err(|e| XdlError::IoError(e.to_string()))?;

    // FITS files start with "SIMPLE  ="
    let header_str = String::from_utf8_lossy(&header);
    if !header_str.starts_with("SIMPLE") {
        return Err(XdlError::InvalidArgument(format!(
            "READFITS: '{}' does not appear to be a valid FITS file",
            filename
        )));
    }

    // Return placeholder message
    println!(
        "READFITS: File '{}' is a valid FITS file but full parsing requires native library.",
        filename
    );
    println!("To enable full FITS support, compile with the 'fits' feature.");

    // Return empty array as placeholder
    Ok(XdlValue::Array(vec![]))
}

/// WRITEFITS - Write data to a FITS file
/// IDL syntax: WRITEFITS, filename, data [, header]
pub fn writefits(args: &[XdlValue], _keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "WRITEFITS: Expected filename and data arguments".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    println!(
        "WRITEFITS: Would write to '{}' but full FITS support requires native library.",
        filename
    );
    println!("To enable full FITS support, compile with the 'fits' feature.");

    Ok(XdlValue::Undefined)
}

/// HEADFITS - Read FITS header
/// IDL syntax: header = HEADFITS(filename)
pub fn headfits(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HEADFITS: Expected filename argument".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let path = Path::new(&filename);
    if !path.exists() {
        return Err(XdlError::FileNotFound(filename));
    }

    // Read first header block (2880 bytes in FITS)
    let mut file = File::open(path).map_err(|e| XdlError::IoError(e.to_string()))?;
    let mut header_block = vec![0u8; 2880];
    let bytes_read = file
        .read(&mut header_block)
        .map_err(|e| XdlError::IoError(e.to_string()))?;

    // Parse header into lines (80 characters each)
    let mut header_lines: Vec<XdlValue> = Vec::new();
    for i in 0..(bytes_read / 80) {
        let start = i * 80;
        let end = start + 80;
        if end <= bytes_read {
            let line = String::from_utf8_lossy(&header_block[start..end])
                .trim_end()
                .to_string();
            if !line.is_empty() {
                header_lines.push(XdlValue::String(line));
            }
            // Stop at END keyword
            if header_block[start..start + 3] == *b"END" {
                break;
            }
        }
    }

    Ok(XdlValue::NestedArray(header_lines))
}

/// SXPAR - Extract parameter from FITS header
/// IDL syntax: value = SXPAR(header, keyword)
pub fn sxpar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SXPAR: Expected header and keyword arguments".to_string(),
        ));
    }

    let header = match &args[0] {
        XdlValue::NestedArray(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string array (header)".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let keyword = match &args[1] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Search for keyword in header
    for line_val in header {
        if let XdlValue::String(line) = line_val {
            // FITS format: KEYWORD = value / comment
            if line.starts_with(&keyword) {
                // Extract value after '='
                if let Some(eq_pos) = line.find('=') {
                    let value_part = line[eq_pos + 1..].trim();
                    // Handle string values (enclosed in quotes)
                    if value_part.starts_with('\'') {
                        if let Some(end_quote) = value_part[1..].find('\'') {
                            return Ok(XdlValue::String(value_part[1..end_quote + 1].to_string()));
                        }
                    }
                    // Handle numeric values
                    let value_str = value_part.split('/').next().unwrap_or("").trim();
                    if let Ok(val) = value_str.parse::<f64>() {
                        return Ok(XdlValue::Double(val));
                    }
                    if let Ok(val) = value_str.parse::<i64>() {
                        return Ok(XdlValue::Long64(val));
                    }
                    // Return as string
                    return Ok(XdlValue::String(value_str.to_string()));
                }
            }
        }
    }

    // Keyword not found
    Ok(XdlValue::Undefined)
}

// ============================================================================
// HDF5 (Hierarchical Data Format) Functions
// ============================================================================

/// H5F_OPEN - Open an HDF5 file
/// IDL syntax: file_id = H5F_OPEN(filename)
pub fn h5f_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5F_OPEN: Expected filename argument".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let path = Path::new(&filename);
    if !path.exists() {
        return Err(XdlError::FileNotFound(filename));
    }

    // Check HDF5 signature (first 8 bytes)
    let mut file = File::open(path).map_err(|e| XdlError::IoError(e.to_string()))?;
    let mut signature = vec![0u8; 8];
    file.read_exact(&mut signature)
        .map_err(|e| XdlError::IoError(e.to_string()))?;

    // HDF5 signature: 0x89 0x48 0x44 0x46 0x0d 0x0a 0x1a 0x0a
    let hdf5_sig = [0x89, 0x48, 0x44, 0x46, 0x0d, 0x0a, 0x1a, 0x0a];
    if signature != hdf5_sig {
        return Err(XdlError::InvalidArgument(format!(
            "H5F_OPEN: '{}' does not appear to be a valid HDF5 file",
            filename
        )));
    }

    println!(
        "H5F_OPEN: File '{}' is a valid HDF5 file but full parsing requires hdf5 library.",
        filename
    );
    println!("To enable full HDF5 support, compile with the 'hdf5' feature.");

    // Return placeholder file ID
    Ok(XdlValue::Long(1))
}

/// H5F_CLOSE - Close an HDF5 file
pub fn h5f_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5F_CLOSE: Expected file_id argument".to_string(),
        ));
    }

    // Placeholder - just return success
    Ok(XdlValue::Long(0))
}

/// H5D_READ - Read HDF5 dataset (placeholder)
pub fn h5d_read(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5D_READ: Expected dataset_id argument".to_string(),
        ));
    }

    println!("H5D_READ: Full HDF5 dataset reading requires hdf5 library.");
    println!("To enable full HDF5 support, compile with the 'hdf5' feature.");

    Ok(XdlValue::Array(vec![]))
}

// ============================================================================
// NetCDF (Network Common Data Form) Functions
// ============================================================================

/// NCDF_OPEN - Open a NetCDF file
/// IDL syntax: ncid = NCDF_OPEN(filename)
pub fn ncdf_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NCDF_OPEN: Expected filename argument".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let path = Path::new(&filename);
    if !path.exists() {
        return Err(XdlError::FileNotFound(filename));
    }

    // Check NetCDF signature (first 4 bytes)
    let mut file = File::open(path).map_err(|e| XdlError::IoError(e.to_string()))?;
    let mut signature = vec![0u8; 4];
    file.read_exact(&mut signature)
        .map_err(|e| XdlError::IoError(e.to_string()))?;

    // NetCDF-3 signature: "CDF" + version byte (1 or 2)
    // NetCDF-4/HDF5 signature starts with HDF5 magic
    let is_netcdf3 = signature[0..3] == *b"CDF";
    let is_hdf5 = signature[0] == 0x89 && signature[1] == 0x48;

    if !is_netcdf3 && !is_hdf5 {
        return Err(XdlError::InvalidArgument(format!(
            "NCDF_OPEN: '{}' does not appear to be a valid NetCDF file",
            filename
        )));
    }

    let format = if is_netcdf3 { "NetCDF-3" } else { "NetCDF-4/HDF5" };
    println!(
        "NCDF_OPEN: File '{}' appears to be {} format but full parsing requires netcdf library.",
        filename, format
    );
    println!("To enable full NetCDF support, compile with the 'netcdf' feature.");

    // Return placeholder file ID
    Ok(XdlValue::Long(1))
}

/// NCDF_CLOSE - Close a NetCDF file
pub fn ncdf_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NCDF_CLOSE: Expected ncid argument".to_string(),
        ));
    }

    Ok(XdlValue::Long(0))
}

/// NCDF_VARGET - Read NetCDF variable (placeholder)
pub fn ncdf_varget(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NCDF_VARGET: Expected ncid and varid arguments".to_string(),
        ));
    }

    println!("NCDF_VARGET: Full NetCDF variable reading requires netcdf library.");
    println!("To enable full NetCDF support, compile with the 'netcdf' feature.");

    Ok(XdlValue::Array(vec![]))
}

/// NCDF_INQUIRE - Inquire about NetCDF file (placeholder)
pub fn ncdf_inquire(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NCDF_INQUIRE: Expected ncid argument".to_string(),
        ));
    }

    println!("NCDF_INQUIRE: Full NetCDF inquiry requires netcdf library.");

    // Return a placeholder structure
    Ok(XdlValue::NestedArray(vec![
        XdlValue::Long(0),  // ndims
        XdlValue::Long(0),  // nvars
        XdlValue::Long(0),  // ngatts
        XdlValue::Long(-1), // recdim
    ]))
}

// ============================================================================
// Additional HDF5 Functions
// ============================================================================

/// H5A_OPEN - Open an HDF5 attribute
pub fn h5a_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "H5A_OPEN: Expected loc_id and attr_name".to_string(),
        ));
    }
    println!("H5A_OPEN: Opening attribute (requires hdf5 library for full support)");
    Ok(XdlValue::Long(1))
}

/// H5A_READ - Read HDF5 attribute data
pub fn h5a_read(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5A_READ: Expected attr_id".to_string(),
        ));
    }
    println!("H5A_READ: Reading attribute (requires hdf5 library for full support)");
    Ok(XdlValue::Array(vec![]))
}

/// H5A_CLOSE - Close an HDF5 attribute
pub fn h5a_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5A_CLOSE: Expected attr_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5A_GET_NAME - Get HDF5 attribute name
pub fn h5a_get_name(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5A_GET_NAME: Expected attr_id".to_string(),
        ));
    }
    Ok(XdlValue::String("unknown".to_string()))
}

/// H5A_GET_NUM_ATTRS - Get number of attributes
pub fn h5a_get_num_attrs(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5A_GET_NUM_ATTRS: Expected loc_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5D_OPEN - Open an HDF5 dataset
pub fn h5d_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "H5D_OPEN: Expected file_id and dataset_name".to_string(),
        ));
    }

    let dataset_name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "unknown".to_string(),
    };

    println!("H5D_OPEN: Opening dataset '{}' (requires hdf5 library)", dataset_name);
    Ok(XdlValue::Long(1))
}

/// H5D_CLOSE - Close an HDF5 dataset
pub fn h5d_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5D_CLOSE: Expected dataset_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5D_GET_SPACE - Get dataspace of dataset
pub fn h5d_get_space(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5D_GET_SPACE: Expected dataset_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(1))
}

/// H5D_GET_TYPE - Get datatype of dataset
pub fn h5d_get_type(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5D_GET_TYPE: Expected dataset_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(1))
}

/// H5G_OPEN - Open an HDF5 group
pub fn h5g_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "H5G_OPEN: Expected loc_id and group_name".to_string(),
        ));
    }

    let group_name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "/".to_string(),
    };

    println!("H5G_OPEN: Opening group '{}' (requires hdf5 library)", group_name);
    Ok(XdlValue::Long(1))
}

/// H5G_CLOSE - Close an HDF5 group
pub fn h5g_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5G_CLOSE: Expected group_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5G_GET_NMEMBERS - Get number of objects in a group
pub fn h5g_get_nmembers(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "H5G_GET_NMEMBERS: Expected loc_id and group_name".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5G_GET_MEMBER_NAME - Get name of object in a group
pub fn h5g_get_member_name(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "H5G_GET_MEMBER_NAME: Expected loc_id, group_name, and index".to_string(),
        ));
    }
    Ok(XdlValue::String("unknown".to_string()))
}

/// H5S_GET_SIMPLE_EXTENT_DIMS - Get dataspace dimensions
pub fn h5s_get_simple_extent_dims(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5S_GET_SIMPLE_EXTENT_DIMS: Expected space_id".to_string(),
        ));
    }
    Ok(XdlValue::Array(vec![]))
}

/// H5S_GET_SIMPLE_EXTENT_NDIMS - Get number of dimensions
pub fn h5s_get_simple_extent_ndims(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5S_GET_SIMPLE_EXTENT_NDIMS: Expected space_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5S_CLOSE - Close a dataspace
pub fn h5s_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5S_CLOSE: Expected space_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

/// H5T_GET_SIZE - Get datatype size
pub fn h5t_get_size(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5T_GET_SIZE: Expected type_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(8)) // Default to 8 bytes (double)
}

/// H5T_CLOSE - Close a datatype
pub fn h5t_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "H5T_CLOSE: Expected type_id".to_string(),
        ));
    }
    Ok(XdlValue::Long(0))
}

// ============================================================================
// Additional NetCDF Functions
// ============================================================================

/// NCDF_VARINQ - Inquire about a NetCDF variable
pub fn ncdf_varinq(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NCDF_VARINQ: Expected ncid and varid".to_string(),
        ));
    }

    println!("NCDF_VARINQ: Variable inquiry requires netcdf library");

    // Return placeholder structure
    let mut result = HashMap::new();
    result.insert("NAME".to_string(), XdlValue::String("unknown".to_string()));
    result.insert("DATATYPE".to_string(), XdlValue::String("FLOAT".to_string()));
    result.insert("NDIMS".to_string(), XdlValue::Long(0));
    result.insert("NATTS".to_string(), XdlValue::Long(0));
    result.insert("DIM".to_string(), XdlValue::Array(vec![]));

    Ok(XdlValue::Struct(result))
}

/// NCDF_DIMINQ - Inquire about a NetCDF dimension
pub fn ncdf_diminq(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NCDF_DIMINQ: Expected ncid and dimid".to_string(),
        ));
    }

    println!("NCDF_DIMINQ: Dimension inquiry requires netcdf library");

    // Return placeholder: [name, size]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::String("unknown".to_string()),
        XdlValue::Long(0),
    ]))
}

/// NCDF_DIMID - Get dimension ID from name
pub fn ncdf_dimid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NCDF_DIMID: Expected ncid and dim_name".to_string(),
        ));
    }

    let dim_name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "unknown".to_string(),
    };

    println!("NCDF_DIMID: Getting dimension ID for '{}' (requires netcdf library)", dim_name);
    Ok(XdlValue::Long(-1))
}

/// NCDF_VARID - Get variable ID from name
pub fn ncdf_varid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NCDF_VARID: Expected ncid and var_name".to_string(),
        ));
    }

    let var_name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "unknown".to_string(),
    };

    println!("NCDF_VARID: Getting variable ID for '{}' (requires netcdf library)", var_name);
    Ok(XdlValue::Long(-1))
}

/// NCDF_ATTNAME - Get attribute name by index
pub fn ncdf_attname(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_ATTNAME: Expected ncid, varid, and attnum".to_string(),
        ));
    }
    Ok(XdlValue::String("unknown".to_string()))
}

/// NCDF_ATTGET - Get attribute value
pub fn ncdf_attget(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_ATTGET: Expected ncid, varid, and name".to_string(),
        ));
    }

    println!("NCDF_ATTGET: Attribute reading requires netcdf library");
    Ok(XdlValue::Undefined)
}

/// NCDF_ATTINQ - Inquire about an attribute
pub fn ncdf_attinq(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_ATTINQ: Expected ncid, varid, and name".to_string(),
        ));
    }

    // Return placeholder structure
    let mut result = HashMap::new();
    result.insert("DATATYPE".to_string(), XdlValue::String("CHAR".to_string()));
    result.insert("LENGTH".to_string(), XdlValue::Long(0));

    Ok(XdlValue::Struct(result))
}

/// NCDF_CREATE - Create a new NetCDF file
pub fn ncdf_create(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NCDF_CREATE: Expected filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let clobber = keywords.contains_key("CLOBBER");
    let netcdf4 = keywords.contains_key("NETCDF4_FORMAT");

    println!("NCDF_CREATE: Would create '{}' (clobber={}, netcdf4={})", filename, clobber, netcdf4);
    println!("  Full NetCDF creation requires netcdf library");

    Ok(XdlValue::Long(1))
}

/// NCDF_DIMDEF - Define a dimension
pub fn ncdf_dimdef(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_DIMDEF: Expected ncid, name, and size".to_string(),
        ));
    }

    let name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "dim".to_string(),
    };

    println!("NCDF_DIMDEF: Would define dimension '{}' (requires netcdf library)", name);
    Ok(XdlValue::Long(0))
}

/// NCDF_VARDEF - Define a variable
pub fn ncdf_vardef(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_VARDEF: Expected ncid, name, and dims".to_string(),
        ));
    }

    let name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "var".to_string(),
    };

    println!("NCDF_VARDEF: Would define variable '{}' (requires netcdf library)", name);
    Ok(XdlValue::Long(0))
}

/// NCDF_VARPUT - Write data to a variable
pub fn ncdf_varput(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "NCDF_VARPUT: Expected ncid, varid, and data".to_string(),
        ));
    }

    println!("NCDF_VARPUT: Variable writing requires netcdf library");
    Ok(XdlValue::Undefined)
}

/// NCDF_ATTPUT - Write attribute value
pub fn ncdf_attput(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "NCDF_ATTPUT: Expected ncid, varid, name, and value".to_string(),
        ));
    }

    println!("NCDF_ATTPUT: Attribute writing requires netcdf library");
    Ok(XdlValue::Undefined)
}

/// NCDF_CONTROL - Control NetCDF file state
pub fn ncdf_control(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NCDF_CONTROL: Expected ncid".to_string(),
        ));
    }

    let endef = keywords.contains_key("ENDEF");
    let redef = keywords.contains_key("REDEF");
    let sync = keywords.contains_key("SYNC");
    let fill = keywords.contains_key("FILL");

    println!("NCDF_CONTROL: (endef={}, redef={}, sync={}, fill={})", endef, redef, sync, fill);

    Ok(XdlValue::Undefined)
}

// ============================================================================
// Additional FITS Functions
// ============================================================================

/// MRDFITS - Read FITS file with extended options (wrapper for READFITS)
pub fn mrdfits(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MRDFITS: Expected filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let extension = if args.len() > 1 {
        match &args[1] {
            XdlValue::Int(i) => *i as i32,
            XdlValue::Long(l) => *l,
            _ => 0,
        }
    } else {
        0
    };

    let silent = keywords.contains_key("SILENT");

    if !silent {
        println!("MRDFITS: Reading extension {} from '{}' (requires cfitsio library)", extension, filename);
    }

    readfits(args, keywords)
}

/// MWRFITS - Write FITS file with extended options
pub fn mwrfits(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "MWRFITS: Expected data and filename".to_string(),
        ));
    }

    let filename = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let create = keywords.contains_key("CREATE");
    let silent = keywords.contains_key("SILENT");

    if !silent {
        println!("MWRFITS: Would write to '{}' (create={}) - requires cfitsio library", filename, create);
    }

    Ok(XdlValue::Long(1))
}

/// FXREAD - Read FITS primary array with flexible options
pub fn fxread(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FXREAD: Expected filename".to_string(),
        ));
    }

    println!("FXREAD: FITS array reading requires cfitsio library");
    readfits(args, keywords)
}

/// FXWRITE - Write FITS primary array
pub fn fxwrite(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "FXWRITE: Expected filename and data".to_string(),
        ));
    }

    println!("FXWRITE: FITS array writing requires cfitsio library");
    writefits(args, keywords)
}

/// FITS_INFO - Get information about FITS file
pub fn fits_info(args: &[XdlValue], _keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FITS_INFO: Expected filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    // Try to read header and extract basic info
    let path = Path::new(&filename);
    if !path.exists() {
        return Err(XdlError::FileNotFound(filename));
    }

    let mut file = File::open(path).map_err(|e| XdlError::IoError(e.to_string()))?;
    let mut header_block = vec![0u8; 2880];
    let bytes_read = file.read(&mut header_block).map_err(|e| XdlError::IoError(e.to_string()))?;

    // Parse header for basic info
    let mut n_ext = 0;
    let mut bitpix = 0;
    let mut naxis = 0;

    for i in 0..(bytes_read / 80) {
        let start = i * 80;
        let end = start + 80;
        if end <= bytes_read {
            let line = String::from_utf8_lossy(&header_block[start..end]);
            if line.starts_with("BITPIX") {
                if let Some(val) = line.split('=').nth(1) {
                    bitpix = val.split('/').next().unwrap_or("0").trim().parse().unwrap_or(0);
                }
            } else if line.starts_with("NAXIS ") || line.starts_with("NAXIS=") {
                if let Some(val) = line.split('=').nth(1) {
                    naxis = val.split('/').next().unwrap_or("0").trim().parse().unwrap_or(0);
                }
            } else if line.starts_with("XTENSION") {
                n_ext += 1;
            }
            if line.starts_with("END") {
                break;
            }
        }
    }

    println!("FITS_INFO: '{}' - BITPIX={}, NAXIS={}, Extensions={}", filename, bitpix, naxis, n_ext);

    let mut result = HashMap::new();
    result.insert("FILENAME".to_string(), XdlValue::String(filename));
    result.insert("BITPIX".to_string(), XdlValue::Long(bitpix));
    result.insert("NAXIS".to_string(), XdlValue::Long(naxis));
    result.insert("N_EXT".to_string(), XdlValue::Long(n_ext));

    Ok(XdlValue::Struct(result))
}

/// FXADDPAR - Add/modify FITS header parameter
pub fn fxaddpar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "FXADDPAR: Expected header, name, and value".to_string(),
        ));
    }

    let name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => "UNKNOWN".to_string(),
    };

    println!("FXADDPAR: Would add '{}' to header (in-memory only)", name);

    // Return modified header (placeholder)
    Ok(args[0].clone())
}

/// FXPAR - Extract FITS header parameter (alias for SXPAR)
pub fn fxpar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    sxpar(args)
}
