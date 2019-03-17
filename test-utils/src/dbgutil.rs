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

use std::ffi::CStr;
use std::path::Path;

use image;
use libc::{c_char, c_void, memcmp, memcpy};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::consts::*;
use quirc_rs::quirc::*;

use quirc_wrapper as qw;

fn data_type_str(dt: i32) -> &'static str {
    if dt == DATA_TYPE_KANJI {
        "KANJI"
    } else if dt == DATA_TYPE_BYTE {
        "BYTE"
    } else if dt == DATA_TYPE_ALPHA {
        "ALPHA"
    } else if dt == DATA_TYPE_NUMERIC {
        "NUMERIC"
    } else {
        "unknown"
    }
}

/// Dump decoded information on stdout.
pub unsafe fn dump_data(data: *const QuircData) {
    println!("    Version: {}", (*data).version);
    println!(
        "    ECC level: {}",
        (*b"MLHQ\0")[(*data).ecc_level as (usize)] as (i32)
    );
    println!("    Mask: {}", (*data).mask);
    println!(
        "    Data type: {} ({})",
        (*data).data_type,
        data_type_str((*data).data_type)
    );
    println!("    Length: {}", (*data).payload_len);
    println!(
        "    Payload: {}",
        CStr::from_ptr((*data).payload.as_ptr() as *const c_char)
            .to_str()
            .unwrap()
    );
    if (*data).eci != 0 {
        println!("    ECI: {}", (*data).eci);
    }
}

/// Dump a grid cell map on stdout.
pub unsafe fn dump_cells(code: *const QuircCode) {
    let mut u: i32;
    let mut v: i32;
    print!("    {} cells, corners:", (*code).size);
    u = 0i32;
    'loop1: loop {
        if !(u < 4i32) {
            break;
        }
        print!(
            " ({},{})",
            (*code).corners[u as (usize)].x,
            (*code).corners[u as (usize)].y
        );
        u = u + 1;
    }
    println!();
    v = 0i32;
    'loop3: loop {
        if !(v < (*code).size) {
            break;
        }
        print!("    ");
        u = 0i32;
        'loop6: loop {
            if !(u < (*code).size) {
                break;
            }
            let p: i32 = v * (*code).size + u;
            if (*code).cell_bitmap[(p >> 3i32) as (usize)] as (i32) & 1i32 << (p & 7i32) != 0 {
                print!("[]");
            } else {
                print!("  ");
            }
            u = u + 1;
        }
        println!();
        v = v + 1;
    }
}

/// Read an image into the decoder.
///
/// Note that you must call quirc_end() if the function returns
/// successfully (0).
pub fn load_image(q: &mut Quirc, path: &Path) -> Vec<u8> {
    let img = image::open(path).unwrap().grayscale().to_luma();
    let (width, height) = img.dimensions();

    quirc_resize(q, width, height);
    let img_bytes = img.into_raw();
    assert_eq!(img_bytes.len(), width as usize * height as usize);
    img_bytes
}

pub unsafe fn validate(decoder: &mut Quirc, image: &[u8]) {
    let qw_decoder: *mut qw::quirc = qw::quirc_new();
    assert!(qw::quirc_resize(qw_decoder, decoder.image.w, decoder.image.h) >= 0);
    let image_bytes = qw::quirc_begin(
        qw_decoder,
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
    );
    memcpy(
        image_bytes as *mut c_void,
        image.as_ptr() as *const c_void,
        (decoder.image.w * decoder.image.h) as usize,
    );
    qw::quirc_end(qw_decoder);

    assert_eq!(
        decoder.image.pixels.as_slice(),
        std::slice::from_raw_parts(
            (*qw_decoder).pixels,
            ((*qw_decoder).w * (*qw_decoder).h) as usize
        )
    );
    assert_eq!(
        decoder.row_average.as_slice(),
        std::slice::from_raw_parts((*qw_decoder).row_average, decoder.image.w as usize)
    );
    assert_eq!(decoder.image.w, (*qw_decoder).w);
    assert_eq!(decoder.image.h, (*qw_decoder).h);
    assert_eq!(decoder.regions.len(), (*qw_decoder).num_regions as usize);
    assert_eq!(
        memcmp(
            decoder.regions.as_ptr() as *const c_void,
            (*qw_decoder).regions.as_ptr() as *const c_void,
            std::mem::size_of_val(&decoder.regions[0]) * decoder.regions.len()
        ),
        0
    );
    assert_slice_eq(
        &decoder.capstones,
        std::slice::from_raw_parts(
            (*qw_decoder).capstones.as_ptr(),
            (*qw_decoder).num_capstones as usize,
        ),
        assert_capstone_eq,
    );
    assert_slice_eq(
        &decoder.grids,
        std::slice::from_raw_parts(
            (*qw_decoder).grids.as_ptr(),
            (*qw_decoder).num_grids as usize,
        ),
        assert_grid_eq,
    );

    let id_count = quirc_count(decoder);
    assert_eq!(id_count, qw::quirc_count(qw_decoder));

    for i in 0..id_count {
        let code = quirc_extract(decoder, i).unwrap();
        let decode_result = quirc_decode(&code);

        let mut qw_code: qw::quirc_code = std::mem::uninitialized();
        let qw_decode_result;
        let mut qw_data: qw::quirc_data = std::mem::uninitialized();
        qw::quirc_extract(qw_decoder, i, &mut qw_code);
        qw_decode_result = qw::quirc_decode(&qw_code, &mut qw_data);

        assert_eq!(
            memcmp(
                code.corners.as_ptr() as *mut c_void,
                qw_code.corners.as_ptr() as *mut c_void,
                std::mem::size_of_val(&code.corners)
            ),
            0
        );
        assert_eq!(code.size, qw_code.size);
        assert_eq!(
            memcmp(
                code.cell_bitmap.as_ptr() as *mut c_void,
                qw_code.cell_bitmap.as_ptr() as *mut c_void,
                std::mem::size_of_val(&code.cell_bitmap)
            ),
            0
        );

        assert_result_eq(decode_result, qw_decode_result);
        if let Ok(data) = decode_result {
            assert_data_eq(&data, &qw_data);
        }
    }
}

fn assert_slice_eq<A, B>(capstones: &[A], qw_capstones: &[B], f: fn(&A, &B)) {
    assert_eq!(capstones.len(), qw_capstones.len());
    capstones
        .iter()
        .zip(qw_capstones.iter())
        .for_each(|(c, qw_c)| f(c, qw_c));
}

fn assert_capstone_eq(capstone: &Capstone, qw_capstone: &qw::quirc_capstone) {
    assert_eq!(capstone.ring, qw_capstone.ring);
    assert_eq!(capstone.stone, qw_capstone.stone);
    assert_slice_eq(&capstone.corners, &qw_capstone.corners, assert_point_eq);
    assert_point_eq(&capstone.center, &qw_capstone.center);
    assert_eq!(capstone.c, qw_capstone.c);
    assert_eq!(capstone.qr_grid, qw_capstone.qr_grid);
}

fn assert_grid_eq(grid: &Grid, qw_grid: &qw::quirc_grid) {
    assert_eq!(grid.caps, qw_grid.caps);
    assert_eq!(grid.align_region, qw_grid.align_region);
    assert_point_eq(&grid.align, &qw_grid.align);
    assert_slice_eq(&grid.tpep, &qw_grid.tpep, assert_point_eq);
    assert_eq!(grid.hscan, qw_grid.hscan);
    assert_eq!(grid.vscan, qw_grid.vscan);
    assert_eq!(grid.grid_size, qw_grid.grid_size);
    assert_eq!(grid.c, qw_grid.c);
}

fn assert_point_eq(point: &Point, qw_point: &qw::quirc_point) {
    assert_eq!(point.x, qw_point.x);
    assert_eq!(point.y, qw_point.y);
}

fn assert_result_eq<T>(r: Result<T>, qw_r: qw::quirc_decode_error_t) {
    match r {
        Ok(_) => assert_eq!(qw::quirc_decode_error_t_QUIRC_SUCCESS, qw_r),
        Err(e) => assert_eq!(e as u32, qw_r),
    }
}

fn assert_data_eq(data: &QuircData, qw_data: &qw::quirc_data) {
    assert_eq!(data.version, qw_data.version);
    assert_eq!(data.ecc_level, qw_data.ecc_level);
    assert_eq!(data.mask, qw_data.mask);
    assert_eq!(data.data_type, qw_data.data_type);
    assert_eq!(data.payload_len, qw_data.payload_len);
    assert_eq!(
        &data.payload[0..data.payload_len as usize],
        &(*qw_data).payload[0..((*qw_data).payload_len) as usize],
    );
    assert_eq!(data.eci, qw_data.eci);
}
