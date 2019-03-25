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

use image::{ImageBuffer, Luma};
use qrcode::QrCode;

use quirc_rs::decode::quirc_decode;
use quirc_rs::identify::{quirc_extract, quirc_identify};
use quirc_rs::quirc::{quirc_count, Image, Quirc};
use test_utils::dbgutil::validate;

fn check_one(code: &QrCode, expected: &str) {
    let image: ImageBuffer<_, Vec<u8>> = code.render::<Luma<u8>>().build();
    let (width, height) = image.dimensions();
    let mut image_bytes = image.into_raw();
    let mut image_bytes_clone = image_bytes.clone();

    let mut decoder = Quirc::new(Image::new(width, height, &mut image_bytes));
    quirc_identify(&mut decoder);
    let count = quirc_count(&decoder);
    assert_eq!(count, 1);
    for i in 0..count {
        let data = quirc_extract(&mut decoder, i).unwrap();
        let data = quirc_decode(&data).unwrap();
        assert_eq!(data.payload(), expected.as_bytes());
    }

    unsafe {
        validate(
            &mut decoder,
            Image::new(width, height, &mut image_bytes_clone),
        );
    }
}

#[test]
fn test() {
    check_one(&QrCode::new("").unwrap(), "");
    let s_8895 = String::from_utf8(vec!['a' as u8; 2000]).unwrap();
    let s_8896 = String::from_utf8(vec!['a' as u8; 8896]).unwrap();
    let s_8897 = String::from_utf8(vec!['a' as u8; 8897]).unwrap();
    check_one(&QrCode::new(&s_8895).unwrap(), &s_8895);
    //    check_one(&QrCode::new(&s_8896).unwrap(), &s_8896);
    //    check_one(&QrCode::new(&s_8897).unwrap(), &s_8897);
}
