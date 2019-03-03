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

static mut gf16_exp: [u8; 16] = [
    0x1u8, 0x2u8, 0x4u8, 0x8u8, 0x3u8, 0x6u8, 0xcu8, 0xbu8, 0x5u8, 0xau8, 0x7u8, 0xeu8, 0xfu8,
    0xdu8, 0x9u8, 0x1u8,
];

static mut gf16_log: [u8; 16] = [
    0x0u8, 0xfu8, 0x1u8, 0x4u8, 0x2u8, 0x8u8, 0x5u8, 0xau8, 0x3u8, 0xeu8, 0x9u8, 0x7u8, 0x6u8,
    0xdu8, 0xbu8, 0xcu8,
];

#[derive(Copy)]
#[repr(C)]
pub struct galois_field {
    pub p: i32,
    pub log: &'static [u8],
    pub exp: &'static [u8],
}

impl Clone for galois_field {
    fn clone(&self) -> Self {
        *self
    }
}

static gf16: galois_field = unsafe {
    galois_field {
        p: 15i32,
        log: &gf16_log,
        exp: &gf16_exp,
    }
};

static mut gf256_exp: [u8; 256] = [
    0x1u8, 0x2u8, 0x4u8, 0x8u8, 0x10u8, 0x20u8, 0x40u8, 0x80u8, 0x1du8, 0x3au8, 0x74u8, 0xe8u8,
    0xcdu8, 0x87u8, 0x13u8, 0x26u8, 0x4cu8, 0x98u8, 0x2du8, 0x5au8, 0xb4u8, 0x75u8, 0xeau8, 0xc9u8,
    0x8fu8, 0x3u8, 0x6u8, 0xcu8, 0x18u8, 0x30u8, 0x60u8, 0xc0u8, 0x9du8, 0x27u8, 0x4eu8, 0x9cu8,
    0x25u8, 0x4au8, 0x94u8, 0x35u8, 0x6au8, 0xd4u8, 0xb5u8, 0x77u8, 0xeeu8, 0xc1u8, 0x9fu8, 0x23u8,
    0x46u8, 0x8cu8, 0x5u8, 0xau8, 0x14u8, 0x28u8, 0x50u8, 0xa0u8, 0x5du8, 0xbau8, 0x69u8, 0xd2u8,
    0xb9u8, 0x6fu8, 0xdeu8, 0xa1u8, 0x5fu8, 0xbeu8, 0x61u8, 0xc2u8, 0x99u8, 0x2fu8, 0x5eu8, 0xbcu8,
    0x65u8, 0xcau8, 0x89u8, 0xfu8, 0x1eu8, 0x3cu8, 0x78u8, 0xf0u8, 0xfdu8, 0xe7u8, 0xd3u8, 0xbbu8,
    0x6bu8, 0xd6u8, 0xb1u8, 0x7fu8, 0xfeu8, 0xe1u8, 0xdfu8, 0xa3u8, 0x5bu8, 0xb6u8, 0x71u8, 0xe2u8,
    0xd9u8, 0xafu8, 0x43u8, 0x86u8, 0x11u8, 0x22u8, 0x44u8, 0x88u8, 0xdu8, 0x1au8, 0x34u8, 0x68u8,
    0xd0u8, 0xbdu8, 0x67u8, 0xceu8, 0x81u8, 0x1fu8, 0x3eu8, 0x7cu8, 0xf8u8, 0xedu8, 0xc7u8, 0x93u8,
    0x3bu8, 0x76u8, 0xecu8, 0xc5u8, 0x97u8, 0x33u8, 0x66u8, 0xccu8, 0x85u8, 0x17u8, 0x2eu8, 0x5cu8,
    0xb8u8, 0x6du8, 0xdau8, 0xa9u8, 0x4fu8, 0x9eu8, 0x21u8, 0x42u8, 0x84u8, 0x15u8, 0x2au8, 0x54u8,
    0xa8u8, 0x4du8, 0x9au8, 0x29u8, 0x52u8, 0xa4u8, 0x55u8, 0xaau8, 0x49u8, 0x92u8, 0x39u8, 0x72u8,
    0xe4u8, 0xd5u8, 0xb7u8, 0x73u8, 0xe6u8, 0xd1u8, 0xbfu8, 0x63u8, 0xc6u8, 0x91u8, 0x3fu8, 0x7eu8,
    0xfcu8, 0xe5u8, 0xd7u8, 0xb3u8, 0x7bu8, 0xf6u8, 0xf1u8, 0xffu8, 0xe3u8, 0xdbu8, 0xabu8, 0x4bu8,
    0x96u8, 0x31u8, 0x62u8, 0xc4u8, 0x95u8, 0x37u8, 0x6eu8, 0xdcu8, 0xa5u8, 0x57u8, 0xaeu8, 0x41u8,
    0x82u8, 0x19u8, 0x32u8, 0x64u8, 0xc8u8, 0x8du8, 0x7u8, 0xeu8, 0x1cu8, 0x38u8, 0x70u8, 0xe0u8,
    0xddu8, 0xa7u8, 0x53u8, 0xa6u8, 0x51u8, 0xa2u8, 0x59u8, 0xb2u8, 0x79u8, 0xf2u8, 0xf9u8, 0xefu8,
    0xc3u8, 0x9bu8, 0x2bu8, 0x56u8, 0xacu8, 0x45u8, 0x8au8, 0x9u8, 0x12u8, 0x24u8, 0x48u8, 0x90u8,
    0x3du8, 0x7au8, 0xf4u8, 0xf5u8, 0xf7u8, 0xf3u8, 0xfbu8, 0xebu8, 0xcbu8, 0x8bu8, 0xbu8, 0x16u8,
    0x2cu8, 0x58u8, 0xb0u8, 0x7du8, 0xfau8, 0xe9u8, 0xcfu8, 0x83u8, 0x1bu8, 0x36u8, 0x6cu8, 0xd8u8,
    0xadu8, 0x47u8, 0x8eu8, 0x1u8,
];

static mut gf256_log: [u8; 256] = [
    0x0u8, 0xffu8, 0x1u8, 0x19u8, 0x2u8, 0x32u8, 0x1au8, 0xc6u8, 0x3u8, 0xdfu8, 0x33u8, 0xeeu8,
    0x1bu8, 0x68u8, 0xc7u8, 0x4bu8, 0x4u8, 0x64u8, 0xe0u8, 0xeu8, 0x34u8, 0x8du8, 0xefu8, 0x81u8,
    0x1cu8, 0xc1u8, 0x69u8, 0xf8u8, 0xc8u8, 0x8u8, 0x4cu8, 0x71u8, 0x5u8, 0x8au8, 0x65u8, 0x2fu8,
    0xe1u8, 0x24u8, 0xfu8, 0x21u8, 0x35u8, 0x93u8, 0x8eu8, 0xdau8, 0xf0u8, 0x12u8, 0x82u8, 0x45u8,
    0x1du8, 0xb5u8, 0xc2u8, 0x7du8, 0x6au8, 0x27u8, 0xf9u8, 0xb9u8, 0xc9u8, 0x9au8, 0x9u8, 0x78u8,
    0x4du8, 0xe4u8, 0x72u8, 0xa6u8, 0x6u8, 0xbfu8, 0x8bu8, 0x62u8, 0x66u8, 0xddu8, 0x30u8, 0xfdu8,
    0xe2u8, 0x98u8, 0x25u8, 0xb3u8, 0x10u8, 0x91u8, 0x22u8, 0x88u8, 0x36u8, 0xd0u8, 0x94u8, 0xceu8,
    0x8fu8, 0x96u8, 0xdbu8, 0xbdu8, 0xf1u8, 0xd2u8, 0x13u8, 0x5cu8, 0x83u8, 0x38u8, 0x46u8, 0x40u8,
    0x1eu8, 0x42u8, 0xb6u8, 0xa3u8, 0xc3u8, 0x48u8, 0x7eu8, 0x6eu8, 0x6bu8, 0x3au8, 0x28u8, 0x54u8,
    0xfau8, 0x85u8, 0xbau8, 0x3du8, 0xcau8, 0x5eu8, 0x9bu8, 0x9fu8, 0xau8, 0x15u8, 0x79u8, 0x2bu8,
    0x4eu8, 0xd4u8, 0xe5u8, 0xacu8, 0x73u8, 0xf3u8, 0xa7u8, 0x57u8, 0x7u8, 0x70u8, 0xc0u8, 0xf7u8,
    0x8cu8, 0x80u8, 0x63u8, 0xdu8, 0x67u8, 0x4au8, 0xdeu8, 0xedu8, 0x31u8, 0xc5u8, 0xfeu8, 0x18u8,
    0xe3u8, 0xa5u8, 0x99u8, 0x77u8, 0x26u8, 0xb8u8, 0xb4u8, 0x7cu8, 0x11u8, 0x44u8, 0x92u8, 0xd9u8,
    0x23u8, 0x20u8, 0x89u8, 0x2eu8, 0x37u8, 0x3fu8, 0xd1u8, 0x5bu8, 0x95u8, 0xbcu8, 0xcfu8, 0xcdu8,
    0x90u8, 0x87u8, 0x97u8, 0xb2u8, 0xdcu8, 0xfcu8, 0xbeu8, 0x61u8, 0xf2u8, 0x56u8, 0xd3u8, 0xabu8,
    0x14u8, 0x2au8, 0x5du8, 0x9eu8, 0x84u8, 0x3cu8, 0x39u8, 0x53u8, 0x47u8, 0x6du8, 0x41u8, 0xa2u8,
    0x1fu8, 0x2du8, 0x43u8, 0xd8u8, 0xb7u8, 0x7bu8, 0xa4u8, 0x76u8, 0xc4u8, 0x17u8, 0x49u8, 0xecu8,
    0x7fu8, 0xcu8, 0x6fu8, 0xf6u8, 0x6cu8, 0xa1u8, 0x3bu8, 0x52u8, 0x29u8, 0x9du8, 0x55u8, 0xaau8,
    0xfbu8, 0x60u8, 0x86u8, 0xb1u8, 0xbbu8, 0xccu8, 0x3eu8, 0x5au8, 0xcbu8, 0x59u8, 0x5fu8, 0xb0u8,
    0x9cu8, 0xa9u8, 0xa0u8, 0x51u8, 0xbu8, 0xf5u8, 0x16u8, 0xebu8, 0x7au8, 0x75u8, 0x2cu8, 0xd7u8,
    0x4fu8, 0xaeu8, 0xd5u8, 0xe9u8, 0xe6u8, 0xe7u8, 0xadu8, 0xe8u8, 0x74u8, 0xd6u8, 0xf4u8, 0xeau8,
    0xa8u8, 0x50u8, 0x58u8, 0xafu8,
];

static gf256: galois_field = unsafe {
    galois_field {
        p: 255i32,
        log: &gf256_log,
        exp: &gf256_exp,
    }
};

#[derive(Copy)]
#[repr(C)]
pub struct quirc_code {
    pub corners: [quirc_point; 4],
    pub size: i32,
    pub cell_bitmap: [u8; 3917],
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
    pub payload: [u8; QUIRC_MAX_PAYLOAD],
    pub payload_len: i32,
    pub eci: u32,
}

impl Clone for quirc_data {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct datastream {
    pub raw: [u8; QUIRC_MAX_PAYLOAD],
    pub data_bits: i32,
    pub ptr: i32,
    pub data: [u8; QUIRC_MAX_PAYLOAD],
}

impl Clone for datastream {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe extern "C" fn grid_bit(mut code: *const quirc_code, mut x: i32, mut y: i32) -> i32 {
    let mut p: i32 = y * (*code).size + x;
    (*code).cell_bitmap[(p >> 3i32) as (usize)] as (i32) >> (p & 7i32) & 1i32
}

unsafe extern "C" fn format_syndromes(mut u: u16, mut s: *mut u8) -> i32 {
    let mut i: i32;
    let mut nonzero: i32 = 0i32;
    memset(s as (*mut ::std::os::raw::c_void), 0i32, 64usize);
    i = 0i32;
    'loop1: loop {
        if !(i < 3i32 * 2i32) {
            break;
        }
        let mut j: i32;
        *s.offset(i as (isize)) = 0u8;
        j = 0i32;
        'loop4: loop {
            if !(j < 15i32) {
                break;
            }
            if u as (i32) & 1i32 << j != 0 {
                let _rhs = gf16_exp[((i + 1i32) * j % 15i32) as (usize)];
                let _lhs = &mut *s.offset(i as (isize));
                *_lhs = (*_lhs as (i32) ^ _rhs as (i32)) as (u8);
            }
            j = j + 1;
        }
        if *s.offset(i as (isize)) != 0 {
            nonzero = 1i32;
        }
        i = i + 1;
    }
    nonzero
}

unsafe extern "C" fn poly_add(
    mut dst: *mut u8,
    mut src: *const u8,
    mut c: u8,
    mut shift: i32,
    mut gf: *const galois_field,
) {
    let mut i: i32;
    let mut log_c: i32 = (*gf).log[c as (usize)] as (i32);
    if c == 0 {
    } else {
        i = 0i32;
        'loop2: loop {
            if !(i < 64i32) {
                break;
            }
            let mut p: i32 = i + shift;
            let mut v: u8 = *src.offset(i as (isize));
            if !(p < 0i32 || p >= 64i32) {
                if !(v == 0) {
                    let _rhs = (*gf).exp
                        [(((*gf).log[v as (usize)] as (i32) + log_c) % (*gf).p) as (usize)];
                    let _lhs = &mut *dst.offset(p as (isize));
                    *_lhs = (*_lhs as (i32) ^ _rhs as (i32)) as (u8);
                }
            }
            i = i + 1;
        }
    }
}

unsafe extern "C" fn berlekamp_massey(
    mut s: *const u8,
    mut N: i32,
    mut gf: *const galois_field,
    mut sigma: *mut u8,
) {
    let mut C: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut B: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut L: i32 = 0i32;
    let mut m: i32 = 1i32;
    let mut b: u8 = 1u8;
    let mut n: i32;
    B[0usize] = 1u8;
    C[0usize] = 1u8;
    n = 0i32;
    'loop1: loop {
        if !(n < N) {
            break;
        }
        let mut d: u8 = *s.offset(n as (isize));
        let mut mult: u8;
        let mut i: i32;
        i = 1i32;
        'loop4: loop {
            if !(i <= L) {
                break;
            }
            if !!(C[i as (usize)] != 0 && (*s.offset((n - i) as (isize)) != 0)) {
                d = (d as (i32)
                    ^ (*gf).exp[(((*gf).log[C[i as (usize)] as (usize)] as (i32)
                        + (*gf).log[*s.offset((n - i) as (isize)) as (usize)] as (i32))
                        % (*gf).p) as (usize)] as (i32)) as (u8);
            }
            i = i + 1;
        }
        mult = (*gf).exp[(((*gf).p - (*gf).log[b as (usize)] as (i32)
            + (*gf).log[d as (usize)] as (i32))
            % (*gf).p) as (usize)];
        if d == 0 {
            m = m + 1;
        } else if L * 2i32 <= n {
            let mut T: [u8; MAX_POLY] = [0u8; MAX_POLY];
            memcpy(
                T.as_mut_ptr() as (*mut ::std::os::raw::c_void),
                C.as_mut_ptr() as (*const ::std::os::raw::c_void),
                ::std::mem::size_of::<[u8; 64]>(),
            );
            poly_add(C.as_mut_ptr(), B.as_mut_ptr() as (*const u8), mult, m, gf);
            memcpy(
                B.as_mut_ptr() as (*mut ::std::os::raw::c_void),
                T.as_mut_ptr() as (*const ::std::os::raw::c_void),
                ::std::mem::size_of::<[u8; 64]>(),
            );
            L = n + 1i32 - L;
            b = d;
            m = 1i32;
        } else {
            poly_add(C.as_mut_ptr(), B.as_mut_ptr() as (*const u8), mult, m, gf);
            m = m + 1;
        }
        n = n + 1;
    }
    memcpy(
        sigma as (*mut ::std::os::raw::c_void),
        C.as_mut_ptr() as (*const ::std::os::raw::c_void),
        64usize,
    );
}

unsafe extern "C" fn poly_eval(mut s: *const u8, mut x: u8, mut gf: *const galois_field) -> u8 {
    let mut i: i32;
    let mut sum: u8 = 0u8;
    let mut log_x: u8 = (*gf).log[x as (usize)];
    if x == 0 {
        *s.offset(0isize)
    } else {
        i = 0i32;
        'loop2: loop {
            if !(i < 64i32) {
                break;
            }
            let mut c: u8 = *s.offset(i as (isize));
            if !(c == 0) {
                sum = (sum as (i32)
                    ^ (*gf).exp[(((*gf).log[c as (usize)] as (i32) + log_x as (i32) * i) % (*gf).p)
                        as (usize)] as (i32)) as (u8);
            }
            i = i + 1;
        }
        sum
    }
}

unsafe extern "C" fn correct_format(mut f_ret: *mut u16) -> Enum1 {
    let mut u: u16 = *f_ret;
    let mut i: i32;
    let mut s: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut sigma: [u8; MAX_POLY] = [0u8; MAX_POLY];
    if format_syndromes(u, s.as_mut_ptr()) == 0 {
        Enum1::QUIRC_SUCCESS
    } else {
        berlekamp_massey(
            s.as_mut_ptr() as (*const u8),
            3i32 * 2i32,
            &gf16 as (*const galois_field),
            sigma.as_mut_ptr(),
        );
        i = 0i32;
        'loop2: loop {
            if !(i < 15i32) {
                break;
            }
            if poly_eval(
                sigma.as_mut_ptr() as (*const u8),
                gf16_exp[(15i32 - i) as (usize)],
                &gf16 as (*const galois_field),
            ) == 0
            {
                u = (u as (i32) ^ 1i32 << i) as (u16);
            }
            i = i + 1;
        }
        (if format_syndromes(u, s.as_mut_ptr()) != 0 {
            Enum1::QUIRC_ERROR_FORMAT_ECC
        } else {
            *f_ret = u;
            Enum1::QUIRC_SUCCESS
        })
    }
}

unsafe extern "C" fn read_format(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut which: i32,
) -> Enum1 {
    let mut i: i32;
    let mut format: u16 = 0u16;
    let mut fdata: u16;
    let mut err: Enum1;
    if which != 0 {
        i = 0i32;
        'loop6: loop {
            if !(i < 7i32) {
                break;
            }
            format =
                (format as (i32) << 1i32 | grid_bit(code, 8i32, (*code).size - 1i32 - i)) as (u16);
            i = i + 1;
        }
        i = 0i32;
        'loop8: loop {
            if !(i < 8i32) {
                break;
            }
            format =
                (format as (i32) << 1i32 | grid_bit(code, (*code).size - 8i32 + i, 8i32)) as (u16);
            i = i + 1;
        }
    } else {
        static mut xs: [i32; 15] = [
            8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 7i32, 5i32, 4i32, 3i32, 2i32, 1i32,
            0i32,
        ];
        static mut ys: [i32; 15] = [
            0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 7i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32,
            8i32,
        ];
        i = 14i32;
        'loop2: loop {
            if !(i >= 0i32) {
                break;
            }
            format = (format as (i32) << 1i32 | grid_bit(code, xs[i as (usize)], ys[i as (usize)]))
                as (u16);
            i = i - 1;
        }
    }
    format = (format as (i32) ^ 0x5412i32) as (u16);
    err = correct_format(&mut format as (*mut u16));
    if err != Enum1::QUIRC_SUCCESS {
        err
    } else {
        fdata = (format as (i32) >> 10i32) as (u16);
        (*data).ecc_level = fdata as (i32) >> 3i32;
        (*data).mask = fdata as (i32) & 7i32;
        Enum1::QUIRC_SUCCESS
    }
}

unsafe extern "C" fn reserved_cell(mut version: i32, mut i: i32, mut j: i32) -> i32 {
    let mut ver: *const quirc_version_info =
        &quirc_version_db[version as (usize)] as (*const quirc_version_info);
    let mut size: i32 = version * 4i32 + 17i32;
    let mut ai: i32 = -1i32;
    let mut aj: i32 = -1i32;
    let mut a: i32;
    if i < 9i32 && (j < 9i32) {
        1i32
    } else if i + 8i32 >= size && (j < 9i32) {
        1i32
    } else if i < 9i32 && (j + 8i32 >= size) {
        1i32
    } else if i == 6i32 || j == 6i32 {
        1i32
    } else {
        if version >= 7i32 {
            if i < 6i32 && (j + 11i32 >= size) {
                return 1i32;
            } else if i + 11i32 >= size && (j < 6i32) {
                return 1i32;
            }
        }
        a = 0i32;
        'loop8: loop {
            if !(a < 7i32 && ((*ver).apat[a as (usize)] != 0)) {
                break;
            }
            let mut p: i32 = (*ver).apat[a as (usize)];
            if abs(p - i) < 3i32 {
                ai = a;
            }
            if abs(p - j) < 3i32 {
                aj = a;
            }
            a = a + 1;
        }
        if ai >= 0i32 && (aj >= 0i32) {
            a = a - 1;
            if ai > 0i32 && (ai < a) {
                return 1i32;
            } else if aj > 0i32 && (aj < a) {
                return 1i32;
            } else if aj == a && (ai == a) {
                return 1i32;
            }
        }
        0i32
    }
}

unsafe extern "C" fn mask_bit(mut mask: i32, mut i: i32, mut j: i32) -> i32 {
    if mask == 7i32 {
        ((i * j % 3i32 + (i + j) % 2i32) % 2i32 == 0) as (i32)
    } else if mask == 6i32 {
        ((i * j % 2i32 + i * j % 3i32) % 2i32 == 0) as (i32)
    } else if mask == 5i32 {
        (i * j % 2i32 + i * j % 3i32 == 0) as (i32)
    } else if mask == 4i32 {
        ((i / 2i32 + j / 3i32) % 2i32 == 0) as (i32)
    } else if mask == 3i32 {
        ((i + j) % 3i32 == 0) as (i32)
    } else if mask == 2i32 {
        (j % 3i32 == 0) as (i32)
    } else if mask == 1i32 {
        (i % 2i32 == 0) as (i32)
    } else if mask == 0i32 {
        ((i + j) % 2i32 == 0) as (i32)
    } else {
        0i32
    }
}

unsafe extern "C" fn read_bit(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut i: i32,
    mut j: i32,
) {
    let mut bitpos: i32 = (*ds).data_bits & 7i32;
    let mut bytepos: i32 = (*ds).data_bits >> 3i32;
    let mut v: i32 = grid_bit(code, j, i);
    if mask_bit((*data).mask, i, j) != 0 {
        v = v ^ 1i32;
    }
    if v != 0 {
        let _rhs = 0x80i32 >> bitpos;
        let _lhs = &mut (*ds).raw[bytepos as (usize)];
        *_lhs = (*_lhs as (i32) | _rhs) as (u8);
    }
    (*ds).data_bits = (*ds).data_bits + 1;
}

unsafe extern "C" fn read_data(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) {
    let mut y: i32 = (*code).size - 1i32;
    let mut x: i32 = (*code).size - 1i32;
    let mut dir: i32 = -1i32;
    'loop1: loop {
        if !(x > 0i32) {
            break;
        }
        if x == 6i32 {
            x = x - 1;
        }
        if reserved_cell((*data).version, y, x) == 0 {
            read_bit(code, data, ds, y, x);
        }
        if reserved_cell((*data).version, y, x - 1i32) == 0 {
            read_bit(code, data, ds, y, x - 1i32);
        }
        y = y + dir;
        if !(y < 0i32 || y >= (*code).size) {
            continue;
        }
        dir = -dir;
        x = x - 2i32;
        y = y + dir;
    }
}

unsafe extern "C" fn block_syndromes(
    mut data: *const u8,
    mut bs: i32,
    mut npar: i32,
    mut s: *mut u8,
) -> i32 {
    let mut nonzero: i32 = 0i32;
    let mut i: i32;
    memset(s as (*mut ::std::os::raw::c_void), 0i32, 64usize);
    i = 0i32;
    'loop1: loop {
        if !(i < npar) {
            break;
        }
        let mut j: i32;
        j = 0i32;
        'loop4: loop {
            if !(j < bs) {
                break;
            }
            let mut c: u8 = *data.offset((bs - j - 1i32) as (isize));
            if !(c == 0) {
                let _rhs =
                    gf256_exp[((gf256_log[c as (usize)] as (i32) + i * j) % 255i32) as (usize)];
                let _lhs = &mut *s.offset(i as (isize));
                *_lhs = (*_lhs as (i32) ^ _rhs as (i32)) as (u8);
            }
            j = j + 1;
        }
        if *s.offset(i as (isize)) != 0 {
            nonzero = 1i32;
        }
        i = i + 1;
    }
    nonzero
}

unsafe extern "C" fn eloc_poly(
    mut omega: *mut u8,
    mut s: *const u8,
    mut sigma: *const u8,
    mut npar: i32,
) {
    let mut i: i32;
    memset(omega as (*mut ::std::os::raw::c_void), 0i32, 64usize);
    i = 0i32;
    'loop1: loop {
        if !(i < npar) {
            break;
        }
        let a: u8 = *sigma.offset(i as (isize));
        let log_a: u8 = gf256_log[a as (usize)];
        let mut j: i32;
        if !(a == 0) {
            j = 0i32;
            'loop5: loop {
                if !(j + 1i32 < 64i32) {
                    break;
                }
                let b: u8 = *s.offset((j + 1i32) as (isize));
                if i + j >= npar {
                    break;
                }
                if !(b == 0) {
                    let _rhs = gf256_exp
                        [((log_a as (i32) + gf256_log[b as (usize)] as (i32)) % 255i32) as (usize)];
                    let _lhs = &mut *omega.offset((i + j) as (isize));
                    *_lhs = (*_lhs as (i32) ^ _rhs as (i32)) as (u8);
                }
                j = j + 1;
            }
        }
        i = i + 1;
    }
}

unsafe extern "C" fn correct_block(mut data: *mut u8, mut ecc: *const quirc_rs_params) -> Enum1 {
    let mut npar: i32 = (*ecc).bs - (*ecc).dw;
    let mut s: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut sigma: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut sigma_deriv: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut omega: [u8; MAX_POLY] = [0u8; MAX_POLY];
    let mut i: i32;

    /* Compute syndrome vector */
    if block_syndromes(data as (*const u8), (*ecc).bs, npar, s.as_mut_ptr()) == 0 {
        Enum1::QUIRC_SUCCESS
    } else {
        berlekamp_massey(
            s.as_mut_ptr() as (*const u8),
            npar,
            &gf256 as (*const galois_field),
            sigma.as_mut_ptr(),
        );

        /* Compute derivative of sigma */
        i = 0i32;
        'loop2: loop {
            if !(i + 1i32 < 64i32) {
                break;
            }
            sigma_deriv[i as (usize)] = sigma[(i + 1i32) as (usize)];
            i = i + 2i32;
        }

        /* Compute error evaluator polynomial */
        eloc_poly(
            omega.as_mut_ptr(),
            s.as_mut_ptr() as (*const u8),
            sigma.as_mut_ptr() as (*const u8),
            npar - 1i32,
        );

        /* Find error locations and magnitudes */
        i = 0i32;
        'loop4: loop {
            if !(i < (*ecc).bs) {
                break;
            }
            let mut xinv: u8 = gf256_exp[(255i32 - i) as (usize)];
            if poly_eval(
                sigma.as_mut_ptr() as (*const u8),
                xinv,
                &gf256 as (*const galois_field),
            ) == 0
            {
                let mut sd_x: u8 = poly_eval(
                    sigma_deriv.as_mut_ptr() as (*const u8),
                    xinv,
                    &gf256 as (*const galois_field),
                );
                let mut omega_x: u8 = poly_eval(
                    omega.as_mut_ptr() as (*const u8),
                    xinv,
                    &gf256 as (*const galois_field),
                );
                let mut error: u8 = gf256_exp[((255i32 - gf256_log[sd_x as (usize)] as (i32)
                    + gf256_log[omega_x as (usize)] as (i32))
                    % 255i32) as (usize)];
                let _rhs = error;
                let _lhs = &mut *data.offset(((*ecc).bs - i - 1i32) as (isize));
                *_lhs = (*_lhs as (i32) ^ _rhs as (i32)) as (u8);
            }
            i = i + 1;
        }
        (if block_syndromes(data as (*const u8), (*ecc).bs, npar, s.as_mut_ptr()) != 0 {
            Enum1::QUIRC_ERROR_DATA_ECC
        } else {
            Enum1::QUIRC_SUCCESS
        })
    }
}

unsafe extern "C" fn codestream_ecc(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut ver: *const quirc_version_info =
        &quirc_version_db[(*data).version as (usize)] as (*const quirc_version_info);
    let mut sb_ecc: *const quirc_rs_params =
        &(*ver).ecc[(*data).ecc_level as (usize)] as (*const quirc_rs_params);
    let mut lb_ecc: quirc_rs_params;
    let lb_count: i32 = ((*ver).data_bytes - (*sb_ecc).bs * (*sb_ecc).ns) / ((*sb_ecc).bs + 1i32);
    let bc: i32 = lb_count + (*sb_ecc).ns;
    let ecc_offset: i32 = (*sb_ecc).dw * bc + lb_count;
    let mut dst_offset: i32 = 0i32;
    let mut i: i32;
    lb_ecc = *sb_ecc;
    lb_ecc.dw = lb_ecc.dw + 1;
    lb_ecc.bs = lb_ecc.bs + 1;

    for i in 0..bc {
        let mut dst: *mut u8 = (*ds).data.as_mut_ptr().offset(dst_offset as (isize));
        let mut ecc: *const quirc_rs_params = if i < (*sb_ecc).ns {
            sb_ecc
        } else {
            &mut lb_ecc as (*mut quirc_rs_params) as (*const quirc_rs_params)
        };
        let num_ec: i32 = (*ecc).bs - (*ecc).dw;
        let mut err: Enum1;
        let mut j: i32;

        for j in 0..(*ecc).dw {
            *dst.offset(j as (isize)) = (*ds).raw[(j * bc + i) as (usize)];
        }
        for j in 0..num_ec {
            *dst.offset(((*ecc).dw + j) as (isize)) =
                (*ds).raw[(ecc_offset + j * bc + i) as (usize)];
        }

        err = correct_block(dst, ecc);
        if err != Enum1::QUIRC_SUCCESS {
            return err;
        }

        dst_offset = dst_offset + (*ecc).dw;
    }

    (*ds).data_bits = dst_offset * 8i32;
    Enum1::QUIRC_SUCCESS
}

unsafe extern "C" fn bits_remaining(mut ds: *const datastream) -> i32 {
    (*ds).data_bits - (*ds).ptr
}

unsafe extern "C" fn take_bits(mut ds: *mut datastream, mut len: i32) -> i32 {
    let mut ret: i32 = 0i32;
    'loop1: loop {
        if !(len != 0 && ((*ds).ptr < (*ds).data_bits)) {
            break;
        }
        let mut b: u8 = (*ds).data[((*ds).ptr >> 3i32) as (usize)];
        let mut bitpos: i32 = (*ds).ptr & 7i32;
        ret = ret << 1i32;
        if b as (i32) << bitpos & 0x80i32 != 0 {
            ret = ret | 1i32;
        }
        (*ds).ptr = (*ds).ptr + 1;
        len = len - 1;
    }
    ret
}

unsafe extern "C" fn numeric_tuple(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut bits: i32,
    mut digits: i32,
) -> i32 {
    let mut tuple: i32;
    let mut i: i32;
    if bits_remaining(ds as (*const datastream)) < bits {
        -1i32
    } else {
        tuple = take_bits(ds, bits);
        i = digits - 1i32;
        'loop2: loop {
            if !(i >= 0i32) {
                break;
            }
            (*data).payload[((*data).payload_len + i) as (usize)] =
                (tuple % 10i32 + b'0' as (i32)) as (u8);
            tuple = tuple / 10i32;
            i = i - 1;
        }
        (*data).payload_len = (*data).payload_len + digits;
        0i32
    }
}

unsafe extern "C" fn decode_numeric(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut _currentBlock;
    let mut bits: i32 = 14i32;
    let mut count: i32;
    if (*data).version < 10i32 {
        bits = 10i32;
    } else if (*data).version < 27i32 {
        bits = 12i32;
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        Enum1::QUIRC_ERROR_DATA_OVERFLOW
    } else {
        'loop5: loop {
            if !(count >= 3i32) {
                _currentBlock = 6;
                break;
            }
            if numeric_tuple(data, ds, 10i32, 3i32) < 0i32 {
                _currentBlock = 17;
                break;
            }
            count = count - 3i32;
        }
        (if _currentBlock == 6 {
            if count >= 2i32 {
                if numeric_tuple(data, ds, 7i32, 2i32) < 0i32 {
                    return Enum1::QUIRC_ERROR_DATA_UNDERFLOW;
                } else {
                    count = count - 2i32;
                }
            }
            if count != 0 {
                if numeric_tuple(data, ds, 4i32, 1i32) < 0i32 {
                    return Enum1::QUIRC_ERROR_DATA_UNDERFLOW;
                } else {
                    count = count - 1;
                }
            }
            Enum1::QUIRC_SUCCESS
        } else {
            Enum1::QUIRC_ERROR_DATA_UNDERFLOW
        })
    }
}

unsafe extern "C" fn alpha_tuple(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut bits: i32,
    mut digits: i32,
) -> i32 {
    let mut tuple: i32;
    let mut i: i32;
    if bits_remaining(ds as (*const datastream)) < bits {
        -1i32
    } else {
        tuple = take_bits(ds, bits);
        i = 0i32;
        'loop2: loop {
            if !(i < digits) {
                break;
            }
            static mut alpha_map: *const u8 =
                (*b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:\0").as_ptr();
            (*data).payload[((*data).payload_len + digits - i - 1i32) as (usize)] =
                *alpha_map.offset((tuple % 45i32) as (isize));
            tuple = tuple / 45i32;
            i = i + 1;
        }
        (*data).payload_len = (*data).payload_len + digits;
        0i32
    }
}

unsafe extern "C" fn decode_alpha(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut _currentBlock;
    let mut bits: i32 = 13i32;
    let mut count: i32;
    if (*data).version < 10i32 {
        bits = 9i32;
    } else if (*data).version < 27i32 {
        bits = 11i32;
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        Enum1::QUIRC_ERROR_DATA_OVERFLOW
    } else {
        'loop5: loop {
            if !(count >= 2i32) {
                _currentBlock = 6;
                break;
            }
            if alpha_tuple(data, ds, 11i32, 2i32) < 0i32 {
                _currentBlock = 13;
                break;
            }
            count = count - 2i32;
        }
        (if _currentBlock == 6 {
            if count != 0 {
                if alpha_tuple(data, ds, 6i32, 1i32) < 0i32 {
                    return Enum1::QUIRC_ERROR_DATA_UNDERFLOW;
                } else {
                    count = count - 1;
                }
            }
            Enum1::QUIRC_SUCCESS
        } else {
            Enum1::QUIRC_ERROR_DATA_UNDERFLOW
        })
    }
}

unsafe extern "C" fn decode_byte(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut bits: i32 = 16i32;
    let mut count: i32;
    let mut i: i32;
    if (*data).version < 10i32 {
        bits = 8i32;
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        Enum1::QUIRC_ERROR_DATA_OVERFLOW
    } else if bits_remaining(ds as (*const datastream)) < count * 8i32 {
        Enum1::QUIRC_ERROR_DATA_UNDERFLOW
    } else {
        i = 0i32;
        'loop5: loop {
            if !(i < count) {
                break;
            }
            (*data).payload[{
                let _old = (*data).payload_len;
                (*data).payload_len = (*data).payload_len + 1;
                _old
            } as (usize)] = take_bits(ds, 8i32) as (u8);
            i = i + 1;
        }
        Enum1::QUIRC_SUCCESS
    }
}

unsafe extern "C" fn decode_kanji(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut bits: i32 = 12i32;
    let mut count: i32;
    let mut i: i32;
    if (*data).version < 10i32 {
        bits = 8i32;
    } else if (*data).version < 27i32 {
        bits = 10i32;
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count * 2i32 + 1i32 > 8896i32 {
        Enum1::QUIRC_ERROR_DATA_OVERFLOW
    } else if bits_remaining(ds as (*const datastream)) < count * 13i32 {
        Enum1::QUIRC_ERROR_DATA_UNDERFLOW
    } else {
        i = 0i32;
        'loop7: loop {
            if !(i < count) {
                break;
            }
            let mut d: i32 = take_bits(ds, 13i32);
            let mut msB: i32 = d / 0xc0i32;
            let mut lsB: i32 = d % 0xc0i32;
            let mut intermediate: i32 = msB << 8i32 | lsB;
            let mut sjw: u16;
            if intermediate + 0x8140i32 <= 0x9ffci32 {
                sjw = (intermediate + 0x8140i32) as (u16);
            } else {
                sjw = (intermediate + 0xc140i32) as (u16);
            }
            (*data).payload[{
                let _old = (*data).payload_len;
                (*data).payload_len = (*data).payload_len + 1;
                _old
            } as (usize)] = (sjw as (i32) >> 8i32) as (u8);
            (*data).payload[{
                let _old = (*data).payload_len;
                (*data).payload_len = (*data).payload_len + 1;
                _old
            } as (usize)] = (sjw as (i32) & 0xffi32) as (u8);
            i = i + 1;
        }
        Enum1::QUIRC_SUCCESS
    }
}

unsafe extern "C" fn decode_eci(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    if bits_remaining(ds as (*const datastream)) < 8i32 {
        Enum1::QUIRC_ERROR_DATA_UNDERFLOW
    } else {
        (*data).eci = take_bits(ds, 8i32) as (u32);
        if (*data).eci & 0xc0u32 == 0x80u32 {
            if bits_remaining(ds as (*const datastream)) < 8i32 {
                return Enum1::QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                (*data).eci = (*data).eci << 8i32 | take_bits(ds, 8i32) as (u32);
            }
        } else if (*data).eci & 0xe0u32 == 0xc0u32 {
            if bits_remaining(ds as (*const datastream)) < 16i32 {
                return Enum1::QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                (*data).eci = (*data).eci << 16i32 | take_bits(ds, 16i32) as (u32);
            }
        }
        Enum1::QUIRC_SUCCESS
    }
}

unsafe extern "C" fn decode_payload(mut data: *mut quirc_data, mut ds: *mut datastream) -> Enum1 {
    let mut _currentBlock;
    let mut err: Enum1 = Enum1::QUIRC_SUCCESS;
    'loop0: loop {
        if !(bits_remaining(ds as (*const datastream)) >= 4i32) {
            _currentBlock = 7;
            break;
        }
        let mut type_: i32 = take_bits(ds, 4i32);
        if type_ == 7i32 {
            err = decode_eci(data, ds);
        } else if type_ == 8i32 {
            err = decode_kanji(data, ds);
        } else if type_ == 4i32 {
            err = decode_byte(data, ds);
        } else if type_ == 2i32 {
            err = decode_alpha(data, ds);
        } else {
            if !(type_ == 1i32) {
                _currentBlock = 7;
                break;
            }
            err = decode_numeric(data, ds);
        }
        if err != Enum1::QUIRC_SUCCESS {
            _currentBlock = 18;
            break;
        }
        if !(type_ & type_ - 1i32 == 0 && (type_ > (*data).data_type)) {
            continue;
        }
        (*data).data_type = type_;
    }
    if _currentBlock == 7 {
        if (*data).payload_len as (usize) >= ::std::mem::size_of::<[u8; 8896]>() {
            (*data).payload_len = (*data).payload_len - 1;
        }
        (*data).payload[(*data).payload_len as (usize)] = 0u8;
        Enum1::QUIRC_SUCCESS
    } else {
        err
    }
}

pub unsafe extern "C" fn quirc_decode(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
) -> Enum1 {
    let mut err: Enum1;
    let mut ds: datastream = datastream {
        raw: [0u8; QUIRC_MAX_PAYLOAD],
        data_bits: 0,
        ptr: 0,
        data: [0u8; QUIRC_MAX_PAYLOAD],
    };
    if ((*code).size - 17i32) % 4i32 != 0 {
        Enum1::QUIRC_ERROR_INVALID_GRID_SIZE
    } else {
        memset(
            data as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<quirc_data>(),
        );
        (*data).version = ((*code).size - 17i32) / 4i32;
        (if (*data).version < 1i32 || (*data).version > 40i32 {
            Enum1::QUIRC_ERROR_INVALID_VERSION
        } else {
            err = read_format(code, data, 0i32);
            if err != Enum1::QUIRC_SUCCESS {
                err = read_format(code, data, 1i32);
            }
            (if err != Enum1::QUIRC_SUCCESS {
                err
            } else {
                read_data(code, data, &mut ds as (*mut datastream));
                err = codestream_ecc(data, &mut ds as (*mut datastream));
                (if err != Enum1::QUIRC_SUCCESS {
                    err
                } else {
                    err = decode_payload(data, &mut ds as (*mut datastream));
                    (if err != Enum1::QUIRC_SUCCESS {
                        err
                    } else {
                        Enum1::QUIRC_SUCCESS
                    })
                })
            })
        })
    }
}
