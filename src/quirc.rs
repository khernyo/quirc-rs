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

extern {
    fn calloc(
        __nmemb : usize, __size : usize
    ) -> *mut ::std::os::raw::c_void;
    fn free(__ptr : *mut ::std::os::raw::c_void);
    fn malloc(__size : usize) -> *mut ::std::os::raw::c_void;
    fn memcpy(
        __dest : *mut ::std::os::raw::c_void,
        __src : *const ::std::os::raw::c_void,
        __n : usize
    ) -> *mut ::std::os::raw::c_void;
    fn memset(
        __s : *mut ::std::os::raw::c_void, __c : i32, __n : usize
    ) -> *mut ::std::os::raw::c_void;
}

#[no_mangle]
pub unsafe extern fn quirc_version() -> *const u8 {
    (*b"1.0\0").as_ptr()
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_point {
    pub x : i32,
    pub y : i32,
}

impl Clone for quirc_point {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_region {
    pub seed : quirc_point,
    pub count : i32,
    pub capstone : i32,
}

impl Clone for quirc_region {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_capstone {
    pub ring : i32,
    pub stone : i32,
    pub corners : [quirc_point; 4],
    pub center : quirc_point,
    pub c : [f64; 8],
    pub qr_grid : i32,
}

impl Clone for quirc_capstone {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_grid {
    pub caps : [i32; 3],
    pub align_region : i32,
    pub align : quirc_point,
    pub tpep : [quirc_point; 3],
    pub hscan : i32,
    pub vscan : i32,
    pub grid_size : i32,
    pub c : [f64; 8],
}

impl Clone for quirc_grid {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc {
    pub image : *mut u8,
    pub pixels : *mut u8,
    pub row_average : *mut i32,
    pub w : i32,
    pub h : i32,
    pub num_regions : i32,
    pub regions : [quirc_region; 254],
    pub num_capstones : i32,
    pub capstones : [quirc_capstone; 32],
    pub num_grids : i32,
    pub grids : [quirc_grid; 8],
}

impl Clone for quirc {
    fn clone(&self) -> Self { *self }
}

#[no_mangle]
pub unsafe extern fn quirc_new() -> *mut quirc {
    let mut q
        : *mut quirc
        = malloc(::std::mem::size_of::<quirc>()) as (*mut quirc);
    if q.is_null() {
        0i32 as (*mut ::std::os::raw::c_void) as (*mut quirc)
    } else {
        memset(
            q as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<quirc>()
        );
        q
    }
}

#[no_mangle]
pub unsafe extern fn quirc_destroy(mut q : *mut quirc) {
    free((*q).image as (*mut ::std::os::raw::c_void));
    if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
        free((*q).pixels as (*mut ::std::os::raw::c_void));
    }
    free((*q).row_average as (*mut ::std::os::raw::c_void));
    free(q as (*mut ::std::os::raw::c_void));
}

#[no_mangle]
pub unsafe extern fn quirc_resize(
    mut q : *mut quirc, mut w : i32, mut h : i32
) -> i32 {
    let mut _currentBlock;
    let mut image
        : *mut u8
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut pixels
        : *mut u8
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut u8);
    let mut row_average
        : *mut i32
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut i32);
    if !(w < 0i32 || h < 0i32) {
        image = calloc(w as (usize),h as (usize)) as (*mut u8);
        if !image.is_null() {
            let mut olddim : usize = ((*q).w * (*q).h) as (usize);
            let mut newdim : usize = (w * h) as (usize);
            let mut min
                : usize
                = if olddim < newdim { olddim } else { newdim };
            memcpy(
                image as (*mut ::std::os::raw::c_void),
                (*q).image as (*const ::std::os::raw::c_void),
                min
            );
            if ::std::mem::size_of::<u8>() != ::std::mem::size_of::<u8>() {
                pixels = calloc(newdim,::std::mem::size_of::<u8>()) as (*mut u8);
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
                row_average = calloc(
                                  w as (usize),
                                  ::std::mem::size_of::<i32>()
                              ) as (*mut i32);
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

#[no_mangle]
pub unsafe extern fn quirc_count(mut q : *const quirc) -> i32 {
    (*q).num_grids
}

static mut error_table : [*const u8; 8] = [0 as (*const u8); 8];
// [QUIRC_SUCCESS] = "Success",
// [QUIRC_ERROR_INVALID_GRID_SIZE] = "Invalid grid size",
// [QUIRC_ERROR_INVALID_VERSION] = "Invalid version",
// [QUIRC_ERROR_FORMAT_ECC] = "Format data ECC failure",
// [QUIRC_ERROR_DATA_ECC] = "ECC failure",
// [QUIRC_ERROR_UNKNOWN_DATA_TYPE] = "Unknown data type",
// [QUIRC_ERROR_DATA_OVERFLOW] = "Data overflow",
// [QUIRC_ERROR_DATA_UNDERFLOW] = "Data underflow"


#[derive(Clone, Copy)]
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

#[no_mangle]
pub unsafe extern fn quirc_strerror(mut err : Enum1) -> *const u8 {
    if err as (i32) >= 0i32 && (err as (usize) < ::std::mem::size_of::<[*const u8; 8]>(
                                                 ).wrapping_div(
                                                     ::std::mem::size_of::<*const u8>()
                                                 )) {
        error_table[err as (usize)]
    } else {
        (*b"Unknown error\0").as_ptr()
    }
}
