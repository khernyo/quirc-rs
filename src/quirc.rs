/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use std::ops::{Index, IndexMut};

/// This structure is used to return information about detected QR codes
/// in the input image.
#[derive(Copy)]
#[repr(C)]
pub struct QuircCode {
    /// The four corners of the QR-code, from top left, clockwise
    pub corners: [Point; 4],

    /* The number of cells across in the QR-code. The cell bitmap
     * is a bitmask giving the actual values of cells. If the cell
     * at (x, y) is black, then the following bit is set:
     *
     *     cell_bitmap[i >> 3] & (1 << (i & 7))
     *
     * where i = (y * size) + x.
     */
    pub size: i32,
    pub cell_bitmap: [u8; consts::MAX_BITMAP],
}

impl Clone for QuircCode {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for QuircCode {
    fn default() -> Self {
        QuircCode {
            corners: [Default::default(); 4],
            size: 0,
            cell_bitmap: [0; consts::MAX_BITMAP],
        }
    }
}

/// This structure holds the decoded QR-code data
#[derive(Copy)]
#[repr(C)]
pub struct QuircData {
    /* Various parameters of the QR-code. These can mostly be
     * ignored if you only care about the data.
     */
    pub version: i32,
    pub ecc_level: i32,
    pub mask: i32,

    /// This field is the highest-valued data type found in the QR
    /// code.
    pub data_type: i32,

    /* Data payload. For the Kanji datatype, payload is encoded as
     * Shift-JIS. For all other datatypes, payload is ASCII text.
     */
    pub payload: [u8; consts::MAX_PAYLOAD],
    pub payload_len: i32,

    /// ECI assignment number
    pub eci: u32,
}

impl Clone for QuircData {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for QuircData {
    fn default() -> Self {
        QuircData {
            version: 0,
            ecc_level: 0,
            mask: 0,
            data_type: 0,
            payload: [0; consts::MAX_PAYLOAD],
            payload_len: 0,
            eci: 0,
        }
    }
}

/// This structure describes a location in the input image buffer.
#[derive(Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Clone for Point {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for Point {
    fn default() -> Self {
        Point { x: 0, y: 0 }
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct Region {
    pub seed: Point,
    pub count: i32,
    pub capstone: i32,
}

impl Clone for Region {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for Region {
    fn default() -> Self {
        Region {
            seed: Default::default(),
            count: 0,
            capstone: 0,
        }
    }
}

#[derive(Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Capstone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [Point; 4],
    pub center: Point,
    pub c: [f64; consts::PERSPECTIVE_PARAMS],
    pub qr_grid: i32,
}

impl Clone for Capstone {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for Capstone {
    fn default() -> Self {
        Capstone {
            ring: 0,
            stone: 0,
            corners: [Default::default(); 4],
            center: Default::default(),
            c: [0f64; consts::PERSPECTIVE_PARAMS],
            qr_grid: 0,
        }
    }
}

#[derive(Copy, Debug)]
#[repr(C)]
pub struct Grid {
    /// Capstone indices
    pub caps: [i32; 3],

    /// Alignment pattern region and corner
    pub align_region: i32,
    pub align: Point,

    /// Timing pattern endpoints
    pub tpep: [Point; 3],
    pub hscan: i32,
    pub vscan: i32,

    /// Grid size and perspective transform
    pub grid_size: i32,
    pub c: [f64; consts::PERSPECTIVE_PARAMS],
}

impl Clone for Grid {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            caps: [0; 3],
            align_region: 0,
            align: Default::default(),
            tpep: [Default::default(); 3],
            hscan: 0,
            vscan: 0,
            grid_size: 0,
            c: [0f64; consts::PERSPECTIVE_PARAMS],
        }
    }
}

pub struct Image<'a> {
    pub(crate) pixels: &'a mut [u8],
    pub(crate) w: i32,
    pub(crate) h: i32,
}

impl<'a> Image<'a> {
    pub fn new(width: u32, height: u32, pixels: &mut [u8]) -> Image {
        assert_eq!((width * height) as usize, pixels.len());

        Image {
            pixels,
            w: width as i32,
            h: height as i32,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.pixels
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }
}

impl<'a> Index<usize> for Image<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pixels[index]
    }
}

impl<'a> IndexMut<usize> for Image<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pixels[index]
    }
}

#[repr(C)]
pub struct Quirc<'a> {
    pub image: Image<'a>,

    /// used by threshold()
    pub row_average: Vec<i32>,

    pub regions: Vec<Region>,
    pub capstones: Vec<Capstone>,
    pub grids: Vec<Grid>,
}

impl<'a> Quirc<'a> {
    pub fn new(image: Image<'a>) -> Self {
        let width = image.w;
        Quirc {
            image,
            row_average: vec![0; width as usize],
            regions: vec![Default::default(); 2],
            capstones: Vec::new(),
            grids: Vec::new(),
        }
    }
}

/// Obtain the library version string.
pub fn quirc_version() -> &'static str {
    "1.0"
}

/// Return the number of QR-codes identified in the last processed
/// image.
pub fn quirc_count(q: &Quirc) -> i32 {
    q.grids.len() as i32
}

pub type Result<T> = core::result::Result<T, DecodeError>;

/// This enum describes the various decoder errors which may occur.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum DecodeError {
    InvalidGridSize = 1,
    InvalidVersion,
    FormatEcc,
    DataEcc,
    UnknownDataType,
    DataOverflow,
    DataUnderflow,
}

/// Return a string error message for an error code.
pub fn quirc_strerror(err: DecodeError) -> &'static str {
    match err {
        DecodeError::InvalidGridSize => "Invalid grid size",
        DecodeError::InvalidVersion => "Invalid version",
        DecodeError::FormatEcc => "Format data ECC failure",
        DecodeError::DataEcc => "ECC failure",
        DecodeError::UnknownDataType => "Unknown data type",
        DecodeError::DataOverflow => "Data overflow",
        DecodeError::DataUnderflow => "Data underflow",
    }
}

pub mod consts {
    pub const PIXEL_WHITE: i32 = 0;
    pub const PIXEL_BLACK: i32 = 1;
    pub const PIXEL_REGION: i32 = 2;

    // TODO handle MAX_REGIONS > 254
    //  See https://github.com/dlbeer/quirc/commit/3a6efb3d84651f67da3ff210bc2eb0e113c0086c
    pub const MAX_REGIONS: usize = 254;

    pub const MAX_CAPSTONES: usize = 32;
    pub const MAX_GRIDS: usize = 8;

    pub const PERSPECTIVE_PARAMS: usize = 8;

    /* Limits on the maximum size of QR-codes and their content. */
    pub const MAX_BITMAP: usize = 3917;
    pub const MAX_PAYLOAD: usize = 8896;

    /* QR-code ECC types. */
    pub const ECC_LEVEL_M: i32 = 0;
    pub const ECC_LEVEL_L: i32 = 1;
    pub const ECC_LEVEL_H: i32 = 2;
    pub const ECC_LEVEL_Q: i32 = 3;

    /* QR-code data types. */
    pub const DATA_TYPE_NUMERIC: i32 = 1;
    pub const DATA_TYPE_ALPHA: i32 = 2;
    pub const DATA_TYPE_BYTE: i32 = 4;
    pub const DATA_TYPE_KANJI: i32 = 8;

    /* Common character encodings */
    pub const ECI_ISO_8859_1: i32 = 1;
    pub const ECI_IBM437: i32 = 2;
    pub const ECI_ISO_8859_2: i32 = 4;
    pub const ECI_ISO_8859_3: i32 = 5;
    pub const ECI_ISO_8859_4: i32 = 6;
    pub const ECI_ISO_8859_5: i32 = 7;
    pub const ECI_ISO_8859_6: i32 = 8;
    pub const ECI_ISO_8859_7: i32 = 9;
    pub const ECI_ISO_8859_8: i32 = 10;
    pub const ECI_ISO_8859_9: i32 = 11;
    pub const ECI_WINDOWS_874: i32 = 13;
    pub const ECI_ISO_8859_13: i32 = 15;
    pub const ECI_ISO_8859_15: i32 = 17;
    pub const ECI_SHIFT_JIS: i32 = 20;
    pub const ECI_UTF_8: i32 = 26;
}
