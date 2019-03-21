/* quirc -- QR-code recognition library
 * Copyright (C) 2019 Szabolcs Berecz <szabolcs.berecz@gmail.com>
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
extern crate image;
extern crate quirc_rs;

use std::ffi::CStr;

use libc::c_char;

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let image_path = match &args.as_slice() {
        [_, path] => path,
        [cmd] => {
            println!("Usage: {} <image-path>", cmd);
            std::process::exit(1);
        }
        _ => panic!(),
    };

    let img = image::open(&image_path).unwrap().grayscale().to_luma();
    let (width, height) = img.dimensions();
    let mut image_bytes = img.into_raw();

    let mut decoder = Quirc::new(Image::new(width, height, &mut image_bytes));
    quirc_identify(&mut decoder);

    let count: i32 = quirc_count(&decoder);
    println!("Found {} QR codes", count);
    for i in 0..count {
        let code = quirc_extract(&mut decoder, i).unwrap();
        let result = quirc_decode(&code);
        match result {
            Ok(data) => {
                println!("  Decoding successful:");
                println!("    Data type: {}", data.data_type);
                println!("    Length: {}", data.payload_len);
                unsafe {
                    println!(
                        "    Payload: {}",
                        CStr::from_ptr(data.payload.as_ptr() as *const c_char)
                            .to_str()
                            .unwrap()
                    );
                }
            }
            Err(e) => println!("  Decoding FAILED: {}", quirc_strerror(e)),
        }
    }
}
