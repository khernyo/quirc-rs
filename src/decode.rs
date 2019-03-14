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

use crate::quirc::consts::*;
use crate::quirc::*;
use crate::version_db::*;

extern "C" {
    fn abs(__x: i32) -> i32;
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

const MAX_POLY: usize = 64;

/*************************************************************************
 * Galois fields
 */

const GF16_EXP: [u8; 16] = [
    0x01, 0x02, 0x04, 0x08, 0x03, 0x06, 0x0c, 0x0b, 0x05, 0x0a, 0x07, 0x0e, 0x0f, 0x0d, 0x09, 0x01,
];

const GF16_LOG: [u8; 16] = [
    0x00, 0x0f, 0x01, 0x04, 0x02, 0x08, 0x05, 0x0a, 0x03, 0x0e, 0x09, 0x07, 0x06, 0x0d, 0x0b, 0x0c,
];

#[derive(Copy)]
#[repr(C)]
struct GaloisField {
    p: i32,
    log: &'static [u8],
    exp: &'static [u8],
}

impl Clone for GaloisField {
    fn clone(&self) -> Self {
        *self
    }
}

const GF16: GaloisField = GaloisField {
    p: 15,
    log: &GF16_LOG,
    exp: &GF16_EXP,
};

const GF256_EXP: [u8; 256] = [
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1d, 0x3a, 0x74, 0xe8, 0xcd, 0x87, 0x13, 0x26,
    0x4c, 0x98, 0x2d, 0x5a, 0xb4, 0x75, 0xea, 0xc9, 0x8f, 0x03, 0x06, 0x0c, 0x18, 0x30, 0x60, 0xc0,
    0x9d, 0x27, 0x4e, 0x9c, 0x25, 0x4a, 0x94, 0x35, 0x6a, 0xd4, 0xb5, 0x77, 0xee, 0xc1, 0x9f, 0x23,
    0x46, 0x8c, 0x05, 0x0a, 0x14, 0x28, 0x50, 0xa0, 0x5d, 0xba, 0x69, 0xd2, 0xb9, 0x6f, 0xde, 0xa1,
    0x5f, 0xbe, 0x61, 0xc2, 0x99, 0x2f, 0x5e, 0xbc, 0x65, 0xca, 0x89, 0x0f, 0x1e, 0x3c, 0x78, 0xf0,
    0xfd, 0xe7, 0xd3, 0xbb, 0x6b, 0xd6, 0xb1, 0x7f, 0xfe, 0xe1, 0xdf, 0xa3, 0x5b, 0xb6, 0x71, 0xe2,
    0xd9, 0xaf, 0x43, 0x86, 0x11, 0x22, 0x44, 0x88, 0x0d, 0x1a, 0x34, 0x68, 0xd0, 0xbd, 0x67, 0xce,
    0x81, 0x1f, 0x3e, 0x7c, 0xf8, 0xed, 0xc7, 0x93, 0x3b, 0x76, 0xec, 0xc5, 0x97, 0x33, 0x66, 0xcc,
    0x85, 0x17, 0x2e, 0x5c, 0xb8, 0x6d, 0xda, 0xa9, 0x4f, 0x9e, 0x21, 0x42, 0x84, 0x15, 0x2a, 0x54,
    0xa8, 0x4d, 0x9a, 0x29, 0x52, 0xa4, 0x55, 0xaa, 0x49, 0x92, 0x39, 0x72, 0xe4, 0xd5, 0xb7, 0x73,
    0xe6, 0xd1, 0xbf, 0x63, 0xc6, 0x91, 0x3f, 0x7e, 0xfc, 0xe5, 0xd7, 0xb3, 0x7b, 0xf6, 0xf1, 0xff,
    0xe3, 0xdb, 0xab, 0x4b, 0x96, 0x31, 0x62, 0xc4, 0x95, 0x37, 0x6e, 0xdc, 0xa5, 0x57, 0xae, 0x41,
    0x82, 0x19, 0x32, 0x64, 0xc8, 0x8d, 0x07, 0x0e, 0x1c, 0x38, 0x70, 0xe0, 0xdd, 0xa7, 0x53, 0xa6,
    0x51, 0xa2, 0x59, 0xb2, 0x79, 0xf2, 0xf9, 0xef, 0xc3, 0x9b, 0x2b, 0x56, 0xac, 0x45, 0x8a, 0x09,
    0x12, 0x24, 0x48, 0x90, 0x3d, 0x7a, 0xf4, 0xf5, 0xf7, 0xf3, 0xfb, 0xeb, 0xcb, 0x8b, 0x0b, 0x16,
    0x2c, 0x58, 0xb0, 0x7d, 0xfa, 0xe9, 0xcf, 0x83, 0x1b, 0x36, 0x6c, 0xd8, 0xad, 0x47, 0x8e, 0x01,
];

const GF256_LOG: [u8; 256] = [
    0x00, 0xff, 0x01, 0x19, 0x02, 0x32, 0x1a, 0xc6, 0x03, 0xdf, 0x33, 0xee, 0x1b, 0x68, 0xc7, 0x4b,
    0x04, 0x64, 0xe0, 0x0e, 0x34, 0x8d, 0xef, 0x81, 0x1c, 0xc1, 0x69, 0xf8, 0xc8, 0x08, 0x4c, 0x71,
    0x05, 0x8a, 0x65, 0x2f, 0xe1, 0x24, 0x0f, 0x21, 0x35, 0x93, 0x8e, 0xda, 0xf0, 0x12, 0x82, 0x45,
    0x1d, 0xb5, 0xc2, 0x7d, 0x6a, 0x27, 0xf9, 0xb9, 0xc9, 0x9a, 0x09, 0x78, 0x4d, 0xe4, 0x72, 0xa6,
    0x06, 0xbf, 0x8b, 0x62, 0x66, 0xdd, 0x30, 0xfd, 0xe2, 0x98, 0x25, 0xb3, 0x10, 0x91, 0x22, 0x88,
    0x36, 0xd0, 0x94, 0xce, 0x8f, 0x96, 0xdb, 0xbd, 0xf1, 0xd2, 0x13, 0x5c, 0x83, 0x38, 0x46, 0x40,
    0x1e, 0x42, 0xb6, 0xa3, 0xc3, 0x48, 0x7e, 0x6e, 0x6b, 0x3a, 0x28, 0x54, 0xfa, 0x85, 0xba, 0x3d,
    0xca, 0x5e, 0x9b, 0x9f, 0x0a, 0x15, 0x79, 0x2b, 0x4e, 0xd4, 0xe5, 0xac, 0x73, 0xf3, 0xa7, 0x57,
    0x07, 0x70, 0xc0, 0xf7, 0x8c, 0x80, 0x63, 0x0d, 0x67, 0x4a, 0xde, 0xed, 0x31, 0xc5, 0xfe, 0x18,
    0xe3, 0xa5, 0x99, 0x77, 0x26, 0xb8, 0xb4, 0x7c, 0x11, 0x44, 0x92, 0xd9, 0x23, 0x20, 0x89, 0x2e,
    0x37, 0x3f, 0xd1, 0x5b, 0x95, 0xbc, 0xcf, 0xcd, 0x90, 0x87, 0x97, 0xb2, 0xdc, 0xfc, 0xbe, 0x61,
    0xf2, 0x56, 0xd3, 0xab, 0x14, 0x2a, 0x5d, 0x9e, 0x84, 0x3c, 0x39, 0x53, 0x47, 0x6d, 0x41, 0xa2,
    0x1f, 0x2d, 0x43, 0xd8, 0xb7, 0x7b, 0xa4, 0x76, 0xc4, 0x17, 0x49, 0xec, 0x7f, 0x0c, 0x6f, 0xf6,
    0x6c, 0xa1, 0x3b, 0x52, 0x29, 0x9d, 0x55, 0xaa, 0xfb, 0x60, 0x86, 0xb1, 0xbb, 0xcc, 0x3e, 0x5a,
    0xcb, 0x59, 0x5f, 0xb0, 0x9c, 0xa9, 0xa0, 0x51, 0x0b, 0xf5, 0x16, 0xeb, 0x7a, 0x75, 0x2c, 0xd7,
    0x4f, 0xae, 0xd5, 0xe9, 0xe6, 0xe7, 0xad, 0xe8, 0x74, 0xd6, 0xf4, 0xea, 0xa8, 0x50, 0x58, 0xaf,
];

const GF256: GaloisField = GaloisField {
    p: 255,
    log: &GF256_LOG,
    exp: &GF256_EXP,
};

/************************************************************************
 * Polynomial operations
 */

unsafe fn poly_add(dst: *mut u8, src: *const u8, c: u8, shift: i32, gf: *const GaloisField) {
    if c == 0 {
        return;
    }

    let log_c: i32 = (*gf).log[c as usize] as i32;
    for i in 0..MAX_POLY as i32 {
        let p: i32 = i + shift;
        let v: u8 = *src.offset(i as isize);

        if !(p < 0 || p >= MAX_POLY as i32) {
            if !(v == 0) {
                *dst.offset(p as isize) ^=
                    (*gf).exp[(((*gf).log[v as usize] as i32 + log_c) % (*gf).p) as usize];
            }
        }
    }
}

unsafe fn poly_eval(s: *const u8, x: u8, gf: *const GaloisField) -> u8 {
    if x == 0 {
        *s.offset(0)
    } else {
        let mut sum: u8 = 0;
        let log_x: u8 = (*gf).log[x as usize];

        for i in 0..MAX_POLY as i32 {
            let c: u8 = *s.offset(i as isize);

            if !(c == 0) {
                sum ^= (*gf).exp
                    [(((*gf).log[c as usize] as i32 + log_x as i32 * i) % (*gf).p) as usize];
            }
        }
        sum
    }
}

/// Berlekamp-Massey algorithm for finding error locator polynomials.
unsafe fn berlekamp_massey(s: *const u8, N: i32, gf: *const GaloisField, sigma: *mut u8) {
    let mut C: [u8; MAX_POLY] = [0; MAX_POLY];
    let mut B: [u8; MAX_POLY] = [0; MAX_POLY];
    let mut L: i32 = 0;
    let mut m: i32 = 1;
    let mut b: u8 = 1;

    B[0] = 1;
    C[0] = 1;

    for n in 0..N {
        let mut d: u8 = *s.offset(n as isize);

        for i in 1..=L {
            if C[i as usize] != 0 && (*s.offset((n - i) as isize) != 0) {
                d ^= (*gf).exp[(((*gf).log[C[i as usize] as usize] as i32
                    + (*gf).log[*s.offset((n - i) as isize) as usize] as i32)
                    % (*gf).p) as usize];
            }
        }

        let mult = (*gf).exp[(((*gf).p - (*gf).log[b as usize] as i32
            + (*gf).log[d as usize] as i32)
            % (*gf).p) as usize];

        if d == 0 {
            m += 1;
        } else if L * 2 <= n {
            let T = C;
            poly_add(C.as_mut_ptr(), B.as_mut_ptr() as (*const u8), mult, m, gf);
            B = T;
            L = n + 1 - L;
            b = d;
            m = 1;
        } else {
            poly_add(C.as_mut_ptr(), B.as_mut_ptr() as (*const u8), mult, m, gf);
            m += 1;
        }
    }

    memcpy(
        sigma as (*mut ::std::os::raw::c_void),
        C.as_mut_ptr() as (*const ::std::os::raw::c_void),
        MAX_POLY,
    );
}

/************************************************************************
 * Code stream error correction
 *
 * Generator polynomial for GF(2^8) is x^8 + x^4 + x^3 + x^2 + 1
 */

unsafe fn block_syndromes(data: *const u8, bs: i32, npar: i32, s: *mut u8) -> i32 {
    let mut nonzero: i32 = 0;

    memset(s as (*mut ::std::os::raw::c_void), 0, MAX_POLY);

    for i in 0..npar {
        for j in 0..bs {
            let c: u8 = *data.offset((bs - j - 1) as isize);

            if !(c == 0) {
                *s.offset(i as isize) ^=
                    GF256_EXP[((GF256_LOG[c as usize] as i32 + i * j) % 255) as usize];
            }
        }

        if *s.offset(i as isize) != 0 {
            nonzero = 1;
        }
    }

    nonzero
}

unsafe fn eloc_poly(omega: *mut u8, s: *const u8, sigma: *const u8, npar: i32) {
    memset(omega as (*mut ::std::os::raw::c_void), 0, MAX_POLY);

    for i in 0..npar {
        let a: u8 = *sigma.offset(i as isize);
        let log_a: u8 = GF256_LOG[a as usize];

        if !(a == 0) {
            for j in 0..MAX_POLY as i32 - 1 {
                let b: u8 = *s.offset((j + 1) as isize);

                if i + j >= npar {
                    break;
                }

                if !(b == 0) {
                    *omega.offset((i + j) as isize) ^=
                        GF256_EXP[((log_a as i32 + GF256_LOG[b as usize] as i32) % 255) as usize];
                }
            }
        }
    }
}

unsafe fn correct_block(data: *mut u8, ecc: *const RsParams) -> Result<()> {
    let npar: i32 = (*ecc).bs - (*ecc).dw;
    let mut s: [u8; MAX_POLY] = [0; MAX_POLY];

    /* Compute syndrome vector */
    if block_syndromes(data as *const u8, (*ecc).bs, npar, s.as_mut_ptr()) == 0 {
        return Ok(());
    }

    let mut sigma: [u8; MAX_POLY] = [0; MAX_POLY];
    berlekamp_massey(
        s.as_mut_ptr() as (*const u8),
        npar,
        &GF256 as (*const GaloisField),
        sigma.as_mut_ptr(),
    );

    /* Compute derivative of sigma */
    let mut sigma_deriv: [u8; MAX_POLY] = [0; MAX_POLY];
    for i in (0..MAX_POLY as i32 - 1).step_by(2) {
        sigma_deriv[i as usize] = sigma[(i + 1) as usize];
    }

    /* Compute error evaluator polynomial */
    let mut omega: [u8; MAX_POLY] = [0; MAX_POLY];
    eloc_poly(
        omega.as_mut_ptr(),
        s.as_mut_ptr() as (*const u8),
        sigma.as_mut_ptr() as (*const u8),
        npar - 1,
    );

    /* Find error locations and magnitudes */
    for i in 0..(*ecc).bs {
        let xinv: u8 = GF256_EXP[(255 - i) as usize];

        if poly_eval(
            sigma.as_mut_ptr() as (*const u8),
            xinv,
            &GF256 as (*const GaloisField),
        ) == 0
        {
            let sd_x: u8 = poly_eval(
                sigma_deriv.as_mut_ptr() as (*const u8),
                xinv,
                &GF256 as (*const GaloisField),
            );
            let omega_x: u8 = poly_eval(
                omega.as_mut_ptr() as (*const u8),
                xinv,
                &GF256 as (*const GaloisField),
            );
            let error: u8 = GF256_EXP[((255 - GF256_LOG[sd_x as usize] as i32
                + GF256_LOG[omega_x as usize] as i32)
                % 255) as usize];

            *data.offset(((*ecc).bs - i - 1) as isize) ^= error;
        }
    }

    if block_syndromes(data as (*const u8), (*ecc).bs, npar, s.as_mut_ptr()) != 0 {
        Err(DecodeError::DataEcc)
    } else {
        Ok(())
    }
}

/************************************************************************
 * Format value error correction
 *
 * Generator polynomial for GF(2^4) is x^4 + x + 1
 */

const FORMAT_MAX_ERROR: i32 = 3;
const FORMAT_SYNDROMES: i32 = (FORMAT_MAX_ERROR * 2);
const FORMAT_BITS: i32 = 15;

unsafe fn format_syndromes(u: u16, s: *mut u8) -> i32 {
    let mut nonzero: i32 = 0;

    memset(s as (*mut ::std::os::raw::c_void), 0, MAX_POLY);

    for i in 0..FORMAT_SYNDROMES {
        *s.offset(i as isize) = 0;

        for j in 0..FORMAT_BITS {
            if u as i32 & 1 << j != 0 {
                *s.offset(i as isize) ^= GF16_EXP[((i + 1) * j % 15) as usize];
            }
        }

        if *s.offset(i as isize) != 0 {
            nonzero = 1;
        }
    }

    nonzero
}

unsafe fn correct_format(f_ret: *mut u16) -> Result<()> {
    let mut u: u16 = *f_ret;
    let mut s: [u8; MAX_POLY] = [0; MAX_POLY];

    // Evaluate U (received codeword) at each of alpha_1 .. alpha_6
    // to get S_1 .. S_6 (but we index them from 0).
    if format_syndromes(u, s.as_mut_ptr()) == 0 {
        return Ok(());
    }

    let mut sigma: [u8; MAX_POLY] = [0; MAX_POLY];
    berlekamp_massey(
        s.as_mut_ptr() as (*const u8),
        FORMAT_SYNDROMES,
        &GF16 as (*const GaloisField),
        sigma.as_mut_ptr(),
    );

    // Now, find the roots of the polynomial
    for i in 0..15 {
        if poly_eval(
            sigma.as_mut_ptr() as (*const u8),
            GF16_EXP[(15 - i) as usize],
            &GF16 as (*const GaloisField),
        ) == 0
        {
            u = (u as i32 ^ 1 << i) as u16;
        }
    }

    if format_syndromes(u, s.as_mut_ptr()) != 0 {
        Err(DecodeError::FormatEcc)
    } else {
        *f_ret = u;
        Ok(())
    }
}

/************************************************************************
 * Decoder algorithm
 */

#[derive(Copy)]
#[repr(C)]
struct DataStream {
    raw: [u8; MAX_PAYLOAD],
    data_bits: i32,
    ptr: i32,
    data: [u8; MAX_PAYLOAD],
}

impl Clone for DataStream {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for DataStream {
    fn default() -> Self {
        DataStream {
            raw: [0; MAX_PAYLOAD],
            data_bits: 0,
            ptr: 0,
            data: [0; MAX_PAYLOAD],
        }
    }
}

fn grid_bit(code: &QuircCode, x: i32, y: i32) -> i32 {
    let p: i32 = y * code.size + x;

    code.cell_bitmap[(p >> 3) as usize] as i32 >> (p & 7) & 1
}

unsafe fn read_format(code: &QuircCode, mut data: &mut QuircData, which: i32) -> Result<()> {
    let mut format: u16 = 0;

    if which != 0 {
        for i in 0..7 {
            format = ((format as i32) << 1 | grid_bit(code, 8, code.size - 1 - i)) as u16;
        }
        for i in 0..8 {
            format = ((format as i32) << 1 | grid_bit(code, code.size - 8 + i, 8)) as u16;
        }
    } else {
        const XS: [i32; 15] = [8, 8, 8, 8, 8, 8, 8, 8, 7, 5, 4, 3, 2, 1, 0];
        const YS: [i32; 15] = [0, 1, 2, 3, 4, 5, 7, 8, 8, 8, 8, 8, 8, 8, 8];

        for i in (0..=14).rev() {
            format = ((format as i32) << 1 | grid_bit(code, XS[i as usize], YS[i as usize])) as u16;
        }
    }

    format = (format as i32 ^ 0x5412) as u16;

    correct_format(&mut format as (*mut u16))?;

    let fdata = ((format as i32) >> 10) as u16;
    data.ecc_level = (fdata as i32) >> 3;
    data.mask = (fdata as i32) & 7;

    Ok(())
}

fn mask_bit(mask: i32, i: i32, j: i32) -> i32 {
    match mask {
        0 => ((i + j) % 2 == 0) as i32,
        1 => (i % 2 == 0) as i32,
        2 => (j % 3 == 0) as i32,
        3 => ((i + j) % 3 == 0) as i32,
        4 => ((i / 2 + j / 3) % 2 == 0) as i32,
        5 => (i * j % 2 + i * j % 3 == 0) as i32,
        6 => ((i * j % 2 + i * j % 3) % 2 == 0) as i32,
        7 => ((i * j % 3 + (i + j) % 2) % 2 == 0) as i32,
        _ => 0,
    }
}

unsafe fn reserved_cell(version: i32, i: i32, j: i32) -> i32 {
    // Finder + format: top left
    if i < 9 && (j < 9) {
        return 1;
    }

    // Finder + format: bottom left
    let size: i32 = version * 4 + 17;
    if i + 8 >= size && (j < 9) {
        return 1;
    }

    // Finder + format: top right
    if i < 9 && (j + 8 >= size) {
        return 1;
    }

    // Exclude timing patterns
    if i == 6 || j == 6 {
        return 1;
    }

    // Exclude version info, if it exists. Version info sits adjacent to
    // the top-right and bottom-left finders in three rows, bounded by
    // the timing pattern.
    if version >= 7 {
        if i < 6 && (j + 11 >= size) {
            return 1;
        }
        if i + 11 >= size && (j < 6) {
            return 1;
        }
    }

    // Exclude alignment patterns
    let ver: *const VersionInfo = &VERSION_DB[version as usize] as (*const VersionInfo);
    let mut ai: i32 = -1;
    let mut aj: i32 = -1;

    let mut a = 0;
    while a < QUIRC_MAX_ALIGNMENT as i32 && (*ver).apat[a as usize] != 0 {
        let p: i32 = (*ver).apat[a as usize];

        if abs(p - i) < 3 {
            ai = a;
        }
        if abs(p - j) < 3 {
            aj = a;
        }
        a += 1;
    }

    if ai >= 0 && (aj >= 0) {
        a -= 1;
        if ai > 0 && (ai < a) {
            return 1;
        }
        if aj > 0 && (aj < a) {
            return 1;
        }
        if aj == a && (ai == a) {
            return 1;
        }
    }

    0
}

fn read_bit(code: &QuircCode, data: &QuircData, ds: &mut DataStream, i: i32, j: i32) {
    let bitpos: i32 = ds.data_bits & 7;
    let bytepos: i32 = ds.data_bits >> 3;
    let mut v: i32 = grid_bit(code, j, i);

    if mask_bit(data.mask, i, j) != 0 {
        v ^= 1;
    }

    if v != 0 {
        ds.raw[bytepos as usize] |= 0x80 >> bitpos
    }

    ds.data_bits += 1;
}

unsafe fn read_data(code: &QuircCode, data: &QuircData) -> DataStream {
    let mut y: i32 = code.size - 1;
    let mut x: i32 = code.size - 1;
    let mut dir: i32 = -1;

    let mut ds: DataStream = Default::default();

    while x > 0 {
        if x == 6 {
            x -= 1;
        }

        if reserved_cell(data.version, y, x) == 0 {
            read_bit(code, data, &mut ds, y, x);
        }

        if reserved_cell(data.version, y, x - 1) == 0 {
            read_bit(code, data, &mut ds, y, x - 1);
        }

        y = y + dir;
        if y < 0 || y >= code.size {
            dir = -dir;
            x = x - 2;
            y = y + dir;
        }
    }

    ds
}

unsafe fn codestream_ecc(data: *mut QuircData, ds: *mut DataStream) -> Result<()> {
    let ver: *const VersionInfo = &VERSION_DB[(*data).version as usize] as (*const VersionInfo);
    let sb_ecc: *const RsParams = &(*ver).ecc[(*data).ecc_level as usize] as (*const RsParams);
    let lb_count: i32 = ((*ver).data_bytes - (*sb_ecc).bs * (*sb_ecc).ns) / ((*sb_ecc).bs + 1);
    let bc: i32 = lb_count + (*sb_ecc).ns;
    let ecc_offset: i32 = (*sb_ecc).dw * bc + lb_count;
    let mut dst_offset: i32 = 0;

    let mut lb_ecc = *sb_ecc;
    lb_ecc.dw += 1;
    lb_ecc.bs += 1;

    for i in 0..bc {
        let dst: *mut u8 = (*ds).data.as_mut_ptr().offset(dst_offset as isize);
        let ecc: *const RsParams = if i < (*sb_ecc).ns {
            sb_ecc
        } else {
            &mut lb_ecc
        };
        let num_ec: i32 = (*ecc).bs - (*ecc).dw;

        for j in 0..(*ecc).dw {
            *dst.offset(j as isize) = (*ds).raw[(j * bc + i) as usize];
        }
        for j in 0..num_ec {
            *dst.offset(((*ecc).dw + j) as isize) = (*ds).raw[(ecc_offset + j * bc + i) as usize];
        }

        correct_block(dst, ecc)?;

        dst_offset += (*ecc).dw;
    }

    (*ds).data_bits = dst_offset * 8;
    Ok(())
}

fn bits_remaining(ds: &DataStream) -> i32 {
    ds.data_bits - ds.ptr
}

fn take_bits(ds: &mut DataStream, mut len: i32) -> i32 {
    let mut ret: i32 = 0;

    while len != 0 && (ds.ptr < ds.data_bits) {
        let b: u8 = ds.data[(ds.ptr >> 3) as usize];
        let bitpos: i32 = ds.ptr & 7;

        ret <<= 1;
        if (b as i32) << bitpos & 0x80 != 0 {
            ret |= 1;
        }

        ds.ptr += 1;
        len -= 1;
    }

    ret
}

fn numeric_tuple(data: &mut QuircData, ds: &mut DataStream, bits: i32, digits: i32) -> i32 {
    if bits_remaining(ds) < bits {
        -1
    } else {
        let mut tuple = take_bits(ds, bits);

        for i in (0..=digits - 1).rev() {
            data.payload[(data.payload_len + i) as usize] = (tuple % 10 + b'0' as i32) as u8;
            tuple /= 10;
        }

        data.payload_len += digits;

        0
    }
}

fn decode_numeric(data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    let bits = if data.version < 10 {
        10
    } else if data.version < 27 {
        12
    } else {
        14
    };

    let mut count = take_bits(ds, bits);
    if data.payload_len + count + 1 > MAX_PAYLOAD as i32 {
        return Err(DecodeError::DataOverflow);
    }

    while count >= 3 {
        if numeric_tuple(data, ds, 10, 3) < 0 {
            return Err(DecodeError::DataUnderflow);
        }
        count -= 3;
    }

    if count >= 2 {
        if numeric_tuple(data, ds, 7, 2) < 0 {
            return Err(DecodeError::DataUnderflow);
        }
        count -= 2;
    }

    if count != 0 {
        if numeric_tuple(data, ds, 4, 1) < 0 {
            return Err(DecodeError::DataUnderflow);
        }
    }

    Ok(())
}

fn alpha_tuple(data: &mut QuircData, ds: &mut DataStream, bits: i32, digits: i32) -> i32 {
    if bits_remaining(ds) < bits {
        -1
    } else {
        let mut tuple = take_bits(ds, bits);

        for i in 0..digits {
            const ALPHA_MAP: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
            data.payload[(data.payload_len + digits - i - 1) as usize] =
                ALPHA_MAP[(tuple % 45) as usize];
            tuple /= 45;
        }

        data.payload_len += digits;
        0
    }
}

fn decode_alpha(data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    let bits = if data.version < 10 {
        9
    } else if data.version < 27 {
        11
    } else {
        13
    };

    let mut count = take_bits(ds, bits);
    if data.payload_len + count + 1 > MAX_PAYLOAD as i32 {
        return Err(DecodeError::DataOverflow);
    }

    while count >= 2 {
        if alpha_tuple(data, ds, 11, 2) < 0 {
            return Err(DecodeError::DataUnderflow);
        }
        count -= 2;
    }

    if count != 0 {
        if alpha_tuple(data, ds, 6, 1) < 0 {
            return Err(DecodeError::DataUnderflow);
        }
    }

    Ok(())
}

fn decode_byte(mut data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    let bits = if data.version < 10 { 8 } else { 16 };

    let count = take_bits(ds, bits);
    if data.payload_len + count + 1 > MAX_PAYLOAD as i32 {
        Err(DecodeError::DataOverflow)
    } else if bits_remaining(ds) < count * 8 {
        Err(DecodeError::DataUnderflow)
    } else {
        for _ in 0..count {
            data.payload[data.payload_len as usize] = take_bits(ds, 8) as u8;
            data.payload_len += 1;
        }

        Ok(())
    }
}

fn decode_kanji(mut data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    let bits = if data.version < 10 {
        8
    } else if data.version < 27 {
        10
    } else {
        12
    };

    let count = take_bits(ds, bits);
    if data.payload_len + count * 2 + 1 > MAX_PAYLOAD as i32 {
        Err(DecodeError::DataOverflow)
    } else if bits_remaining(ds) < count * 13 {
        Err(DecodeError::DataUnderflow)
    } else {
        for _ in 0..count {
            let d: i32 = take_bits(ds, 13);
            let ms_byte: i32 = d / 0xc0;
            let ls_byte: i32 = d % 0xc0;
            let intermediate: i32 = ms_byte << 8 | ls_byte;

            let sjw: u16 = if intermediate + 0x8140 <= 0x9ffc {
                // bytes are in the range 0x8140 to 0x9FFC
                (intermediate + 0x8140) as u16
            } else {
                // bytes are in the range 0xE040 to 0xEBBF
                (intermediate + 0xc140) as u16
            };

            data.payload[data.payload_len as usize] = (sjw as i32 >> 8) as u8;
            data.payload_len += 1;
            data.payload[data.payload_len as usize] = (sjw as i32 & 0xff) as u8;
            data.payload_len += 1;
        }

        Ok(())
    }
}

fn decode_eci(mut data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    if bits_remaining(ds) < 8 {
        Err(DecodeError::DataUnderflow)
    } else {
        data.eci = take_bits(ds, 8) as u32;

        if data.eci & 0xc0 == 0x80 {
            if bits_remaining(ds) < 8 {
                return Err(DecodeError::DataUnderflow);
            }

            data.eci = data.eci << 8 | take_bits(ds, 8) as u32;
        } else if data.eci & 0xe0 == 0xc0 {
            if bits_remaining(ds) < 16 {
                return Err(DecodeError::DataUnderflow);
            }

            data.eci = data.eci << 16 | take_bits(ds, 16) as u32;
        }

        Ok(())
    }
}

fn decode_payload(data: &mut QuircData, ds: &mut DataStream) -> Result<()> {
    while bits_remaining(ds) >= 4 {
        let type_: i32 = take_bits(ds, 4);
        match type_ {
            DATA_TYPE_NUMERIC => decode_numeric(data, ds)?,
            DATA_TYPE_ALPHA => decode_alpha(data, ds)?,
            DATA_TYPE_BYTE => decode_byte(data, ds)?,
            DATA_TYPE_KANJI => decode_kanji(data, ds)?,
            7 => decode_eci(data, ds)?,
            _ => break,
        };

        if type_ & type_ - 1 == 0 && (type_ > data.data_type) {
            data.data_type = type_;
        }
    }

    // Add nul terminator to all payloads
    if data.payload_len as usize >= ::std::mem::size_of::<[u8; 8896]>() {
        data.payload_len -= 1;
    }
    data.payload[data.payload_len as usize] = 0;

    Ok(())
}

/// Decode a QR-code, returning the payload data.
pub unsafe fn quirc_decode(code: &QuircCode) -> Result<QuircData> {
    if (code.size - 17) % 4 != 0 {
        return Err(DecodeError::InvalidGridSize);
    }

    let version = (code.size - 17) / 4;

    if version < 1 || version > QUIRC_MAX_VERSION as i32 {
        return Err(DecodeError::InvalidVersion);
    }

    let mut data = QuircData {
        version,
        ..Default::default()
    };

    // Read format information -- try both locations
    read_format(code, &mut data, 0).or_else(|_| read_format(code, &mut data, 1))?;

    let mut ds = read_data(code, &mut data);
    codestream_ecc(&mut data, &mut ds)?;

    decode_payload(&mut data, &mut ds)?;

    Ok(data)
}
