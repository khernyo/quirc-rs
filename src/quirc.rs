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

#[derive(Copy)]
#[repr(C)]
pub struct quirc_code {
    pub corners: [quirc_point; 4],
    pub size: i32,
    pub cell_bitmap: [u8; consts::QUIRC_MAX_BITMAP],
}

impl Clone for quirc_code {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_data {
    pub version: i32,
    pub ecc_level: i32,
    pub mask: i32,
    pub data_type: i32,
    pub payload: [u8; consts::QUIRC_MAX_PAYLOAD],
    pub payload_len: i32,
    pub eci: u32,
}

impl Clone for quirc_data {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy, Debug)]
#[repr(C)]
pub struct quirc_point {
    pub x: i32,
    pub y: i32,
}

impl Clone for quirc_point {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_region {
    pub seed: quirc_point,
    pub count: i32,
    pub capstone: i32,
}

impl Clone for quirc_region {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy, Debug)]
#[repr(C)]
pub struct quirc_capstone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [quirc_point; 4],
    pub center: quirc_point,
    pub c: [f64; consts::QUIRC_PERSPECTIVE_PARAMS],
    pub qr_grid: i32,
}

impl Clone for quirc_capstone {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy, Debug)]
#[repr(C)]
pub struct quirc_grid {
    pub caps: [i32; 3],
    pub align_region: i32,
    pub align: quirc_point,
    pub tpep: [quirc_point; 3],
    pub hscan: i32,
    pub vscan: i32,
    pub grid_size: i32,
    pub c: [f64; consts::QUIRC_PERSPECTIVE_PARAMS],
}

impl Clone for quirc_grid {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc {
    pub image: *mut u8,
    pub pixels: *mut u8,
    pub row_average: *mut i32,
    pub w: i32,
    pub h: i32,
    pub num_regions: i32,
    pub regions: [quirc_region; 254],
    pub num_capstones: i32,
    pub capstones: [quirc_capstone; consts::QUIRC_MAX_CAPSTONES],
    pub num_grids: i32,
    pub grids: [quirc_grid; consts::QUIRC_MAX_GRIDS],
}

impl Clone for quirc {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn quirc_version() -> &'static str {
    "1.0"
}

pub unsafe extern "C" fn quirc_new() -> *mut quirc {
    let q: *mut quirc = malloc(::std::mem::size_of::<quirc>()) as (*mut quirc);
    if q.is_null() {
        0i32 as (*mut ::std::os::raw::c_void) as (*mut quirc)
    } else {
        memset(
            q as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<quirc>(),
        );
        q
    }
}

pub unsafe extern "C" fn quirc_destroy(q: *mut quirc) {
    free((*q).image as (*mut ::std::os::raw::c_void));
    if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
        free((*q).pixels as (*mut ::std::os::raw::c_void));
    }
    free((*q).row_average as (*mut ::std::os::raw::c_void));
    free(q as (*mut ::std::os::raw::c_void));
}

pub unsafe extern "C" fn quirc_resize(mut q: *mut quirc, w: i32, h: i32) -> i32 {
    let mut _currentBlock;
    let mut image: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut pixels: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut row_average: *mut i32 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut i32);
    if !(w < 0i32 || h < 0i32) {
        image = calloc(w as (usize), h as (usize)) as (*mut u8);
        if !image.is_null() {
            let olddim: usize = ((*q).w * (*q).h) as (usize);
            let newdim: usize = (w * h) as (usize);
            let min: usize = if olddim < newdim { olddim } else { newdim };
            memcpy(
                image as (*mut ::std::os::raw::c_void),
                (*q).image as (*const ::std::os::raw::c_void),
                min,
            );
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
                row_average = calloc(w as (usize), ::std::mem::size_of::<i32>()) as (*mut i32);
                if !row_average.is_null() {
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

pub unsafe extern "C" fn quirc_count(q: *const quirc) -> i32 {
    (*q).num_grids
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum Enum1 {
    QUIRC_SUCCESS = 0i32,
    QUIRC_ERROR_INVALID_GRID_SIZE,
    QUIRC_ERROR_INVALID_VERSION,
    QUIRC_ERROR_FORMAT_ECC,
    QUIRC_ERROR_DATA_ECC,
    QUIRC_ERROR_UNKNOWN_DATA_TYPE,
    QUIRC_ERROR_DATA_OVERFLOW,
    QUIRC_ERROR_DATA_UNDERFLOW,
}

pub unsafe extern "C" fn quirc_strerror(err: Enum1) -> &'static str {
    match err {
        Enum1::QUIRC_SUCCESS => "Success",
        Enum1::QUIRC_ERROR_INVALID_GRID_SIZE => "Invalid grid size",
        Enum1::QUIRC_ERROR_INVALID_VERSION => "Invalid version",
        Enum1::QUIRC_ERROR_FORMAT_ECC => "Format data ECC failure",
        Enum1::QUIRC_ERROR_DATA_ECC => "ECC failure",
        Enum1::QUIRC_ERROR_UNKNOWN_DATA_TYPE => "Unknown data type",
        Enum1::QUIRC_ERROR_DATA_OVERFLOW => "Data overflow",
        Enum1::QUIRC_ERROR_DATA_UNDERFLOW => "Data underflow",
    }
}

pub mod consts {
    pub const QUIRC_PIXEL_WHITE: i32 = 0;
    pub const QUIRC_PIXEL_BLACK: i32 = 1;
    pub const QUIRC_PIXEL_REGION: i32 = 2;

    // TODO handle QUIRC_MAX_REGIONS > 254
    //  See https://github.com/dlbeer/quirc/commit/3a6efb3d84651f67da3ff210bc2eb0e113c0086c
    pub const QUIRC_MAX_REGIONS: i32 = 254;

    pub const QUIRC_MAX_CAPSTONES: usize = 32;
    pub const QUIRC_MAX_GRIDS: usize = 8;

    pub const QUIRC_PERSPECTIVE_PARAMS: usize = 8;

    /* Limits on the maximum size of QR-codes and their content. */
    pub const QUIRC_MAX_BITMAP: usize = 3917;
    pub const QUIRC_MAX_PAYLOAD: usize = 8896;

    /* QR-code ECC types. */
    pub const QUIRC_ECC_LEVEL_M: i32 = 0;
    pub const QUIRC_ECC_LEVEL_L: i32 = 1;
    pub const QUIRC_ECC_LEVEL_H: i32 = 2;
    pub const QUIRC_ECC_LEVEL_Q: i32 = 3;

    /* QR-code data types. */
    pub const QUIRC_DATA_TYPE_NUMERIC: i32 = 1;
    pub const QUIRC_DATA_TYPE_ALPHA: i32 = 2;
    pub const QUIRC_DATA_TYPE_BYTE: i32 = 4;
    pub const QUIRC_DATA_TYPE_KANJI: i32 = 8;

    /* Common character encodings */
    pub const QUIRC_ECI_ISO_8859_1: i32 = 1;
    pub const QUIRC_ECI_IBM437: i32 = 2;
    pub const QUIRC_ECI_ISO_8859_2: i32 = 4;
    pub const QUIRC_ECI_ISO_8859_3: i32 = 5;
    pub const QUIRC_ECI_ISO_8859_4: i32 = 6;
    pub const QUIRC_ECI_ISO_8859_5: i32 = 7;
    pub const QUIRC_ECI_ISO_8859_6: i32 = 8;
    pub const QUIRC_ECI_ISO_8859_7: i32 = 9;
    pub const QUIRC_ECI_ISO_8859_8: i32 = 10;
    pub const QUIRC_ECI_ISO_8859_9: i32 = 11;
    pub const QUIRC_ECI_WINDOWS_874: i32 = 13;
    pub const QUIRC_ECI_ISO_8859_13: i32 = 15;
    pub const QUIRC_ECI_ISO_8859_15: i32 = 17;
    pub const QUIRC_ECI_SHIFT_JIS: i32 = 20;
    pub const QUIRC_ECI_UTF_8: i32 = 26;
}
