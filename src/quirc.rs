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

extern "C" {
    fn calloc(__nmemb: usize, __size: usize) -> *mut ::std::os::raw::c_void;
    fn free(__ptr: *mut ::std::os::raw::c_void);
    fn malloc(__size: usize) -> *mut ::std::os::raw::c_void;
    fn memcpy(
        __dest: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
    fn memset(
        __s: *mut ::std::os::raw::c_void,
        __c: i32,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
}

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

/// This structure describes a location in the input image buffer.
#[derive(Copy, Debug)]
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

#[derive(Copy, Debug)]
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

#[derive(Copy)]
#[repr(C)]
pub struct Quirc {
    pub image: *mut u8,
    pub pixels: *mut u8,

    /// used by threshold()
    pub row_average: *mut i32,

    pub w: i32,
    pub h: i32,
    pub num_regions: i32,
    pub regions: [Region; consts::MAX_REGIONS as usize],
    pub num_capstones: i32,
    pub capstones: [Capstone; consts::MAX_CAPSTONES],
    pub num_grids: i32,
    pub grids: [Grid; consts::MAX_GRIDS],
}

impl Clone for Quirc {
    fn clone(&self) -> Self {
        *self
    }
}

/// Obtain the library version string.
pub fn quirc_version() -> &'static str {
    "1.0"
}

/// Construct a new QR-code recognizer. This function will return NULL
/// if sufficient memory could not be allocated.
pub unsafe extern "C" fn quirc_new() -> *mut Quirc {
    let q: *mut Quirc = malloc(::std::mem::size_of::<Quirc>()) as (*mut Quirc);
    if q.is_null() {
        0i32 as (*mut ::std::os::raw::c_void) as (*mut Quirc)
    } else {
        memset(
            q as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<Quirc>(),
        );
        q
    }
}

/// Destroy a QR-code recognizer.
pub unsafe extern "C" fn quirc_destroy(q: *mut Quirc) {
    free((*q).image as (*mut ::std::os::raw::c_void));
    // q->pixels may alias q->image when their type representation is of the
    // same size, so we need to be careful here to avoid a double free
    if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
        free((*q).pixels as (*mut ::std::os::raw::c_void));
    }
    free((*q).row_average as (*mut ::std::os::raw::c_void));
    free(q as (*mut ::std::os::raw::c_void));
}

/// Resize the QR-code recognizer. The size of an image must be
/// specified before codes can be analyzed.
///
/// This function returns 0 on success, or -1 if sufficient memory could
/// not be allocated.
pub unsafe extern "C" fn quirc_resize(mut q: *mut Quirc, w: i32, h: i32) -> i32 {
    let mut _currentBlock;
    let mut image: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut pixels: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut row_average: *mut i32 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut i32);

    // XXX: w and h should be size_t (or at least unsigned) as negatives
    // values would not make much sense. The downside is that it would break
    // both the API and ABI. Thus, at the moment, let's just do a sanity
    // check.
    if !(w < 0i32 || h < 0i32) {
        // alloc a new buffer for q->image. We avoid realloc(3) because we want
        // on failure to be leave `q` in a consistant, unmodified state.
        image = calloc(w as (usize), h as (usize)) as (*mut u8);
        if !image.is_null() {
            // compute the "old" (i.e. currently allocated) and the "new"
            // (i.e. requested) image dimensions
            let olddim: usize = ((*q).w * (*q).h) as (usize);
            let newdim: usize = (w * h) as (usize);
            let min: usize = if olddim < newdim { olddim } else { newdim };

            // copy the data into the new buffer, avoiding (a) to read beyond the
            // old buffer when the new size is greater and (b) to write beyond the
            // new buffer when the new size is smaller, hence the min computation.
            memcpy(
                image as (*mut ::std::os::raw::c_void),
                (*q).image as (*const ::std::os::raw::c_void),
                min,
            );

            // alloc a new buffer for q->pixels if needed
            if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
                pixels = calloc(newdim, ::std::mem::size_of::<u8>()) as (*mut u8);
                if pixels.is_null() {
                    _currentBlock = 8;
                } else {
                    _currentBlock = 4;
                }
            } else {
                _currentBlock = 4;
            }
            if _currentBlock == 8 {
            } else {
                // alloc a new buffer for q->row_average
                row_average = calloc(w as (usize), ::std::mem::size_of::<i32>()) as (*mut i32);
                if !row_average.is_null() {
                    // alloc succeeded, update `q` with the new size and buffers
                    (*q).w = w;
                    (*q).h = h;
                    free((*q).image as (*mut ::std::os::raw::c_void));
                    (*q).image = image;
                    if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
                        free((*q).pixels as (*mut ::std::os::raw::c_void));
                        (*q).pixels = pixels;
                    }
                    free((*q).row_average as (*mut ::std::os::raw::c_void));
                    (*q).row_average = row_average;
                    return 0i32;
                }
            }
        }
    }
    free(image as (*mut ::std::os::raw::c_void));
    free(pixels as (*mut ::std::os::raw::c_void));
    free(row_average as (*mut ::std::os::raw::c_void));
    -1i32
}

/// Return the number of QR-codes identified in the last processed
/// image.
pub unsafe extern "C" fn quirc_count(q: *const Quirc) -> i32 {
    (*q).num_grids
}

/// This enum describes the various decoder errors which may occur.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum DecodeResult {
    Success = 0i32,
    ErrorInvalidGridSize,
    ErrorInvalidVersion,
    ErrorFormatEcc,
    ErrorDataEcc,
    ErrorUnknownDataType,
    ErrorDataOverflow,
    ErrorDataUnderflow,
}

/// Return a string error message for an error code.
pub unsafe extern "C" fn quirc_strerror(err: DecodeResult) -> &'static str {
    match err {
        DecodeResult::Success => "Success",
        DecodeResult::ErrorInvalidGridSize => "Invalid grid size",
        DecodeResult::ErrorInvalidVersion => "Invalid version",
        DecodeResult::ErrorFormatEcc => "Format data ECC failure",
        DecodeResult::ErrorDataEcc => "ECC failure",
        DecodeResult::ErrorUnknownDataType => "Unknown data type",
        DecodeResult::ErrorDataOverflow => "Data overflow",
        DecodeResult::ErrorDataUnderflow => "Data underflow",
    }
}

pub mod consts {
    pub const PIXEL_WHITE: i32 = 0;
    pub const PIXEL_BLACK: i32 = 1;
    pub const PIXEL_REGION: i32 = 2;

    // TODO handle MAX_REGIONS > 254
    //  See https://github.com/dlbeer/quirc/commit/3a6efb3d84651f67da3ff210bc2eb0e113c0086c
    pub const MAX_REGIONS: i32 = 254;

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
