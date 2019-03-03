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

pub fn quirc_version() -> &'static str {
    "1.0"
}

#[derive(Copy)]
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

#[derive(Copy)]
#[repr(C)]
pub struct quirc_capstone {
    pub ring: i32,
    pub stone: i32,
    pub corners: [quirc_point; 4],
    pub center: quirc_point,
    pub c: [f64; 8],
    pub qr_grid: i32,
}

impl Clone for quirc_capstone {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_grid {
    pub caps: [i32; 3],
    pub align_region: i32,
    pub align: quirc_point,
    pub tpep: [quirc_point; 3],
    pub hscan: i32,
    pub vscan: i32,
    pub grid_size: i32,
    pub c: [f64; 8],
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
    pub capstones: [quirc_capstone; 32],
    pub num_grids: i32,
    pub grids: [quirc_grid; 8],
}

impl Clone for quirc {
    fn clone(&self) -> Self {
        *self
    }
}

pub unsafe extern "C" fn quirc_new() -> *mut quirc {
    let mut q: *mut quirc = malloc(::std::mem::size_of::<quirc>()) as (*mut quirc);
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

pub unsafe extern "C" fn quirc_destroy(mut q: *mut quirc) {
    free((*q).image as (*mut ::std::os::raw::c_void));
    if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
        free((*q).pixels as (*mut ::std::os::raw::c_void));
    }
    free((*q).row_average as (*mut ::std::os::raw::c_void));
    free(q as (*mut ::std::os::raw::c_void));
}

pub unsafe extern "C" fn quirc_resize(mut q: *mut quirc, mut w: i32, mut h: i32) -> i32 {
    let mut _currentBlock;
    let mut image: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut pixels: *mut u8 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut row_average: *mut i32 = 0i32 as (*mut ::std::os::raw::c_void) as (*mut i32);
    if !(w < 0i32 || h < 0i32) {
        image = calloc(w as (usize), h as (usize)) as (*mut u8);
        if !image.is_null() {
            let mut olddim: usize = ((*q).w * (*q).h) as (usize);
            let mut newdim: usize = (w * h) as (usize);
            let mut min: usize = if olddim < newdim { olddim } else { newdim };
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

pub unsafe extern "C" fn quirc_count(mut q: *const quirc) -> i32 {
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

pub unsafe extern "C" fn quirc_strerror(mut err: Enum1) -> &'static str {
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

pub(crate) mod consts {
    /* Limits on the maximum size of QR-codes and their content. */
    const QUIRC_MAX_BITMAP: usize = 3917;
    pub const QUIRC_MAX_PAYLOAD: usize = 8896;

    /* QR-code ECC types. */
    const QUIRC_ECC_LEVEL_M: usize = 0;
    const QUIRC_ECC_LEVEL_L: usize = 1;
    const QUIRC_ECC_LEVEL_H: usize = 2;
    const QUIRC_ECC_LEVEL_Q: usize = 3;

    /* QR-code data types. */
    const QUIRC_DATA_TYPE_NUMERIC: usize = 1;
    const QUIRC_DATA_TYPE_ALPHA: usize = 2;
    const QUIRC_DATA_TYPE_BYTE: usize = 4;
    const QUIRC_DATA_TYPE_KANJI: usize = 8;

    /* Common character encodings */
    const QUIRC_ECI_ISO_8859_1: usize = 1;
    const QUIRC_ECI_IBM437: usize = 2;
    const QUIRC_ECI_ISO_8859_2: usize = 4;
    const QUIRC_ECI_ISO_8859_3: usize = 5;
    const QUIRC_ECI_ISO_8859_4: usize = 6;
    const QUIRC_ECI_ISO_8859_5: usize = 7;
    const QUIRC_ECI_ISO_8859_6: usize = 8;
    const QUIRC_ECI_ISO_8859_7: usize = 9;
    const QUIRC_ECI_ISO_8859_8: usize = 10;
    const QUIRC_ECI_ISO_8859_9: usize = 11;
    const QUIRC_ECI_WINDOWS_874: usize = 13;
    const QUIRC_ECI_ISO_8859_13: usize = 15;
    const QUIRC_ECI_ISO_8859_15: usize = 17;
    const QUIRC_ECI_SHIFT_JIS: usize = 20;
    const QUIRC_ECI_UTF_8: usize = 26;
}
