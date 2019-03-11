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
use quirc_rs::quirc::*;
use quirc_rs::quirc::consts::*;

use quirc_wrapper as qw;

unsafe extern fn data_type_str(dt : i32) -> &'static str {
    if dt == QUIRC_DATA_TYPE_KANJI {
        "KANJI"
    } else if dt == QUIRC_DATA_TYPE_BYTE {
        "BYTE"
    } else if dt == QUIRC_DATA_TYPE_ALPHA {
        "ALPHA"
    } else if dt == QUIRC_DATA_TYPE_NUMERIC {
        "NUMERIC"
    } else {
        "unknown"
    }
}

pub unsafe extern fn dump_data(data : *mut quirc_data) {
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
        CStr::from_ptr((*data).payload.as_mut_ptr() as *mut c_char).to_str().unwrap()
    );
    if (*data).eci != 0 {
        println!("    ECI: {}", (*data).eci);
    }
}

pub unsafe extern fn dump_cells(code : *const quirc_code) {
    let mut u : i32;
    let mut v : i32;
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
            let p : i32 = v * (*code).size + u;
            if (*code).cell_bitmap[
                   (p >> 3i32) as (usize)
               ] as (i32) & 1i32 << (p & 7i32) != 0 {
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

pub unsafe fn load_image(q: *mut quirc, path: &Path) -> i32 {
    let img = image::open(path).unwrap().grayscale().to_luma();
    let (width, height) = img.dimensions();

    if !(quirc_resize(q, width as i32, height as i32) < 0i32) {
        let image_bytes = quirc_begin(
            q,
            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32)
        );

        let img_bytes = img.into_raw();
        assert_eq!(img_bytes.len(), width as usize * height as usize);
        libc::memcpy(image_bytes as *mut c_void, img_bytes.as_ptr() as *mut c_void, img_bytes.len());

        return 0i32;
    }
    -1i32
}

pub unsafe fn validate(
    decoder: *mut quirc,
    image: *const c_void,
) {
    let qw_decoder : *mut qw::quirc = qw::quirc_new();
    assert!(qw::quirc_resize(qw_decoder, (*decoder).w, (*decoder).h) >= 0);
    let image_bytes = qw::quirc_begin(
        qw_decoder,
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32)
    );
    memcpy(image_bytes as *mut c_void, image, ((*decoder).w * (*decoder).h) as usize);
    qw::quirc_end(qw_decoder);

    assert_eq!(memcmp((*decoder).image as *const c_void, (*qw_decoder).image as *const c_void, ((*decoder).w * (*decoder).h) as usize * std::mem::size_of_val(&*(*decoder).image)), 0);
    assert_eq!(memcmp((*decoder).pixels as *const c_void, (*qw_decoder).pixels as *const c_void, ((*decoder).w * (*decoder).h) as usize * std::mem::size_of_val(&*(*decoder).pixels)), 0);
    assert_eq!(memcmp((*decoder).row_average as *const c_void, (*qw_decoder).row_average as *const c_void, (*decoder).w as usize * std::mem::size_of_val(&*(*decoder).row_average)), 0);
    assert_eq!((*decoder).w, (*qw_decoder).w);
    assert_eq!((*decoder).h, (*qw_decoder).h);
    assert_eq!((*decoder).num_regions, (*qw_decoder).num_regions);
    assert_eq!(memcmp((*decoder).regions.as_ptr() as *const c_void, (*qw_decoder).regions.as_ptr() as *const c_void, std::mem::size_of_val(&(*decoder).regions[0]) * (*decoder).num_regions as usize), 0);
    assert_eq!((*decoder).num_capstones, (*qw_decoder).num_capstones);
    assert_eq!(memcmp((*decoder).capstones.as_ptr() as *const c_void, (*qw_decoder).capstones.as_ptr() as *const c_void, std::mem::size_of_val(&(*decoder).capstones[0]) * (*decoder).num_capstones as usize), 0);
    assert_eq!((*decoder).num_grids, (*qw_decoder).num_grids);
    assert_eq!(memcmp((*decoder).grids.as_ptr() as *const c_void, (*qw_decoder).grids.as_ptr() as *const c_void, std::mem::size_of_val(&(*decoder).grids[0]) * (*decoder).num_grids as usize), 0);

    let id_count = quirc_count(decoder);
    assert_eq!(id_count, qw::quirc_count(qw_decoder));

    for i in 0..id_count {
        let mut code: quirc_code = std::mem::uninitialized();
        let decode_result;
        let mut data: quirc_data = std::mem::uninitialized();
        quirc_extract(decoder, i, &mut code);
        decode_result = quirc_decode(&code, &mut data);

        let mut qw_code: qw::quirc_code = std::mem::uninitialized();
        let qw_decode_result;
        let mut qw_data: qw::quirc_data = std::mem::uninitialized();
        qw::quirc_extract(qw_decoder, i, &mut qw_code);
        qw_decode_result = qw::quirc_decode(&qw_code, &mut qw_data);

        assert_eq!(memcmp(code.corners.as_ptr() as *mut c_void, qw_code.corners.as_ptr() as *mut c_void, std::mem::size_of_val(&code.corners)), 0);
        assert_eq!(code.size, qw_code.size);
        assert_eq!(memcmp(code.cell_bitmap.as_ptr() as *mut c_void, qw_code.cell_bitmap.as_ptr() as *mut c_void, std::mem::size_of_val(&code.cell_bitmap)), 0);

        assert_eq!(decode_result as u32, qw_decode_result);
        assert_eq!(data.version, qw_data.version);
        assert_eq!(data.ecc_level, qw_data.ecc_level);
        assert_eq!(data.mask, qw_data.mask);
        assert_eq!(data.data_type, qw_data.data_type);
        assert_eq!(data.payload_len, qw_data.payload_len);
        assert_eq!(memcmp(data.payload.as_ptr() as *mut c_void, qw_data.payload.as_ptr() as *mut c_void, std::mem::size_of_val(&data.payload)), 0);
        assert_eq!(data.eci, qw_data.eci);
    }
}
