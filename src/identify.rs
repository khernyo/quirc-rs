/* quirc - QR-code recognition library
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
use std::os::raw::c_double;

extern "C" {
    fn abs(__x: i32) -> i32;
    fn memcpy(
        __dest: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
    fn memmove(
        __dest: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
    fn memset(
        __s: *mut ::std::os::raw::c_void,
        __c: i32,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
    fn rint(x: c_double) -> c_double;
}

/************************************************************************
 * Linear algebra routines
 */

fn line_intersect(p0: &Point, p1: &Point, q0: &Point, q1: &Point, r: &mut Point) -> i32 {
    // (a, b) is perpendicular to line p
    let a: i32 = -(p1.y - p0.y);
    let b: i32 = p1.x - p0.x;

    // (c, d) is perpendicular to line q
    let c: i32 = -(q1.y - q0.y);
    let d: i32 = q1.x - q0.x;

    // e and f are dot products of the respective vectors with p and q
    let e: i32 = a * p1.x + b * p1.y;
    let f: i32 = c * q1.x + d * q1.y;

    // Now we need to solve:
    //     [a b] [rx]   [e]
    //     [c d] [ry] = [f]
    //
    // We do this by inverting the matrix and applying it to (e, f):
    //       [ d -b] [e]   [rx]
    // 1/det [-c  a] [f] = [ry]
    let det: i32 = a * d - b * c;

    if det == 0 {
        0i32
    } else {
        r.x = (d * e - b * f) / det;
        r.y = (-c * e + a * f) / det;
        1i32
    }
}

fn perspective_setup(rect: &[Point; 4], w: f64, h: f64) -> [f64; 8] {
    let x0: f64 = rect[0].x as (f64);
    let y0: f64 = rect[0].y as (f64);
    let x1: f64 = rect[1].x as (f64);
    let y1: f64 = rect[1].y as (f64);
    let x2: f64 = rect[2].x as (f64);
    let y2: f64 = rect[2].y as (f64);
    let x3: f64 = rect[3].x as (f64);
    let y3: f64 = rect[3].y as (f64);
    let wden: f64 = w * (x2 * y3 - x3 * y2 + (x3 - x2) * y1 + x1 * (y2 - y3));
    let hden: f64 = h * (x2 * y3 + x1 * (y2 - y3) - x3 * y2 + (x3 - x2) * y1);
    [
        (x1 * (x2 * y3 - x3 * y2)
            + x0 * (-x2 * y3 + x3 * y2 + (x2 - x3) * y1)
            + x1 * (x3 - x2) * y0)
            / wden,
        -(x0 * (x2 * y3 + x1 * (y2 - y3) - x2 * y1) - x1 * x3 * y2
            + x2 * x3 * y1
            + (x1 * x3 - x2 * x3) * y0)
            / hden,
        x0,
        (y0 * (x1 * (y3 - y2) - x2 * y3 + x3 * y2)
            + y1 * (x2 * y3 - x3 * y2)
            + x0 * y1 * (y2 - y3))
            / wden,
        (x0 * (y1 * y3 - y2 * y3) + x1 * y2 * y3 - x2 * y1 * y3
            + y0 * (x3 * y2 - x1 * y2 + (x2 - x3) * y1))
            / hden,
        y0,
        (x1 * (y3 - y2) + x0 * (y2 - y3) + (x2 - x3) * y1 + (x3 - x2) * y0) / wden,
        (-x2 * y3 + x1 * y3 + x3 * y2 + x0 * (y1 - y2) - x3 * y1 + (x2 - x1) * y0) / hden,
    ]
}

unsafe fn perspective_map(c: &[f64; consts::PERSPECTIVE_PARAMS], u: f64, v: f64) -> Point {
    let den: f64 = c[6] * u + c[7] * v + 1.0f64;
    let x: f64 = (c[0] * u + c[1] * v + c[2]) / den;
    let y: f64 = (c[3] * u + c[4] * v + c[5]) / den;

    Point {
        x: rint(x) as i32,
        y: rint(y) as i32,
    }
}

fn perspective_unmap(c: &[f64; consts::PERSPECTIVE_PARAMS], in_: &Point) -> (f64, f64) {
    let x: f64 = in_.x as f64;
    let y: f64 = in_.y as f64;
    let den: f64 =
        -c[0] * c[7] * y + c[1] * c[6] * y + (c[3] * c[7] - c[4] * c[6]) * x + c[0] * c[4]
            - c[1] * c[3];

    let u = -(c[1] * (y - c[5]) - c[2] * c[7] * y + (c[5] * c[7] - c[4]) * x + c[2] * c[4]) / den;
    let v = (c[0] * (y - c[5]) - c[2] * c[6] * y + (c[5] * c[6] - c[3]) * x + c[2] * c[3]) / den;

    (u, v)
}

const FLOOD_FILL_MAX_DEPTH: i32 = 4096;

type SpanFunc =
    unsafe extern "C" fn(user_data: *mut ::std::os::raw::c_void, y: i32, left: i32, right: i32);

/// Span-based floodfill routine
pub unsafe extern "C" fn flood_fill_seed(
    q: &mut Quirc,
    x: i32,
    y: i32,
    from: i32,
    to: i32,
    func: Option<SpanFunc>,
    user_data: *mut ::std::os::raw::c_void,
    depth: i32,
) {
    let mut left: i32 = x;
    let mut right: i32 = x;
    let mut i: i32;
    let mut row: *mut u8 = q.pixels.as_mut_ptr().offset((y * q.w) as isize);
    if depth >= FLOOD_FILL_MAX_DEPTH {
    } else {
        'loop1: loop {
            if !(left > 0i32 && (*row.offset((left - 1i32) as (isize)) as (i32) == from)) {
                break;
            }
            left = left - 1;
        }
        'loop2: loop {
            if !(right < q.w - 1i32 && (*row.offset((right + 1i32) as (isize)) as (i32) == from)) {
                break;
            }
            right = right + 1;
        }

        // Fill the extent
        i = left;
        'loop4: loop {
            if !(i <= right) {
                break;
            }
            *row.offset(i as (isize)) = to as (u8);
            i = i + 1;
        }
        if let Some(f) = func {
            f(user_data, y, left, right);
        }

        // Seed new flood-fills
        if y > 0i32 {
            row = q.pixels.as_mut_ptr().offset(((y - 1i32) * q.w) as isize);
            i = left;
            'loop9: loop {
                if !(i <= right) {
                    break;
                }
                if *row.offset(i as (isize)) as (i32) == from {
                    flood_fill_seed(q, i, y - 1i32, from, to, func, user_data, depth + 1i32);
                }
                i = i + 1;
            }
        }
        if y < q.h - 1i32 {
            row = q.pixels.as_mut_ptr().offset(((y + 1i32) * q.w) as isize);
            i = left;
            'loop12: loop {
                if !(i <= right) {
                    break;
                }
                if *row.offset(i as (isize)) as (i32) == from {
                    flood_fill_seed(q, i, y + 1i32, from, to, func, user_data, depth + 1i32);
                }
                i = i + 1;
            }
        }
    }
}

const THRESHOLD_S_MIN: i32 = 1;
const THRESHOLD_S_DEN: i32 = 8;
const THRESHOLD_T: i32 = 5;

/// Adaptive thresholding
pub unsafe extern "C" fn threshold(q: &mut Quirc) {
    let mut x: i32;
    let mut y: i32;
    let mut avg_w: i32 = 0i32;
    let mut avg_u: i32 = 0i32;
    let mut threshold_s: i32 = q.w / THRESHOLD_S_DEN;
    let mut row: *mut u8 = q.pixels.as_mut_ptr();

    // Ensure a sane, non-zero value for threshold_s.
    //
    // threshold_s can be zero if the image width is small. We need to avoid
    // SIGFPE as it will be used as divisor.
    if threshold_s < THRESHOLD_S_MIN {
        threshold_s = THRESHOLD_S_MIN;
    }
    y = 0i32;
    'loop3: loop {
        if !(y < q.h) {
            break;
        }
        q.row_average.iter_mut().for_each(|x| *x = 0);
        x = 0i32;
        'loop6: loop {
            if !(x < q.w) {
                break;
            }
            let w: i32;
            let u: i32;
            if y & 1i32 != 0 {
                w = x;
                u = q.w - 1i32 - x;
            } else {
                w = q.w - 1i32 - x;
                u = x;
            }
            avg_w = avg_w * (threshold_s - 1i32) / threshold_s + *row.offset(w as (isize)) as (i32);
            avg_u = avg_u * (threshold_s - 1i32) / threshold_s + *row.offset(u as (isize)) as (i32);
            let _rhs = avg_w;
            let _lhs = &mut q.row_average[w as usize];
            *_lhs = *_lhs + _rhs;
            let _rhs = avg_u;
            let _lhs = &mut q.row_average[u as usize];
            *_lhs = *_lhs + _rhs;
            x = x + 1;
        }
        x = 0i32;
        'loop8: loop {
            if !(x < q.w) {
                break;
            }
            if *row.offset(x as (isize)) as (i32)
                < q.row_average[x as usize] * (100i32 - THRESHOLD_T) / (200i32 * threshold_s)
            {
                *row.offset(x as (isize)) = PIXEL_BLACK as u8;
            } else {
                *row.offset(x as (isize)) = PIXEL_WHITE as u8;
            }
            x = x + 1;
        }
        row = row.offset(q.w as (isize));
        y = y + 1;
    }
}

pub unsafe extern "C" fn area_count(
    user_data: *mut ::std::os::raw::c_void,
    _y: i32,
    left: i32,
    right: i32,
) {
    let _rhs = right - left + 1i32;
    let _lhs = &mut (*(user_data as (*mut Region))).count;
    *_lhs = *_lhs + _rhs;
}

pub unsafe extern "C" fn region_code(q: &mut Quirc, x: i32, y: i32) -> i32 {
    let pixel: i32;
    let mut r#box: *mut Region;
    let region: i32;
    if x < 0i32 || y < 0i32 || x >= q.w || y >= q.h {
        -1i32
    } else {
        pixel = *q.pixels.as_mut_ptr().offset((y * q.w + x) as isize) as i32;
        (if pixel >= PIXEL_REGION {
            pixel
        } else if pixel == PIXEL_WHITE {
            -1i32
        } else if q.num_regions >= MAX_REGIONS {
            -1i32
        } else {
            region = q.num_regions;
            r#box = &mut q.regions[{
                let _old = q.num_regions;
                q.num_regions = q.num_regions + 1;
                _old
            } as (usize)];
            memset(
                r#box as (*mut ::std::os::raw::c_void),
                0i32,
                ::std::mem::size_of::<Region>(),
            );
            (*r#box).seed.x = x;
            (*r#box).seed.y = y;
            (*r#box).capstone = -1i32;
            flood_fill_seed(
                q,
                x,
                y,
                pixel,
                region,
                Some(area_count),
                r#box as (*mut ::std::os::raw::c_void),
                0i32,
            );
            region
        })
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct PolygonScoreData {
    pub r#ref: Point,
    pub scores: [i32; 4],
    pub corners: *mut Point,
}

impl Clone for PolygonScoreData {
    fn clone(&self) -> Self {
        *self
    }
}

pub unsafe extern "C" fn find_one_corner(
    user_data: *mut ::std::os::raw::c_void,
    y: i32,
    left: i32,
    right: i32,
) {
    let mut psd: *mut PolygonScoreData = user_data as (*mut PolygonScoreData);
    let xs: [i32; 2] = [left, right];
    let dy: i32 = y - (*psd).r#ref.y;
    let mut i: i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 2i32) {
            break;
        }
        let dx: i32 = xs[i as (usize)] - (*psd).r#ref.x;
        let d: i32 = dx * dx + dy * dy;
        if d > (*psd).scores[0usize] {
            (*psd).scores[0usize] = d;
            (*(*psd).corners.offset(0isize)).x = xs[i as (usize)];
            (*(*psd).corners.offset(0isize)).y = y;
        }
        i = i + 1;
    }
}

pub unsafe extern "C" fn find_other_corners(
    user_data: *mut ::std::os::raw::c_void,
    y: i32,
    left: i32,
    right: i32,
) {
    let mut psd: *mut PolygonScoreData = user_data as (*mut PolygonScoreData);
    let xs: [i32; 2] = [left, right];
    let mut i: i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 2i32) {
            break;
        }
        let up: i32 = xs[i as (usize)] * (*psd).r#ref.x + y * (*psd).r#ref.y;
        let right: i32 = xs[i as (usize)] * -(*psd).r#ref.y + y * (*psd).r#ref.x;
        let scores: [i32; 4] = [up, right, -up, -right];
        let mut j: i32;
        j = 0i32;
        'loop4: loop {
            if !(j < 4i32) {
                break;
            }
            if scores[j as (usize)] > (*psd).scores[j as (usize)] {
                (*psd).scores[j as (usize)] = scores[j as (usize)];
                (*(*psd).corners.offset(j as (isize))).x = xs[i as (usize)];
                (*(*psd).corners.offset(j as (isize))).y = y;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

pub unsafe extern "C" fn find_region_corners(
    q: &mut Quirc,
    rcode: i32,
    r#ref: *const Point,
    corners: *mut Point,
) {
    let region: *mut Region = &mut q.regions[rcode as (usize)];
    let mut psd: PolygonScoreData = PolygonScoreData {
        r#ref: *r#ref,
        scores: [-1i32, 0i32, 0i32, 0i32],
        corners,
    };
    let mut i: i32;
    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        rcode,
        PIXEL_BLACK,
        Some(find_one_corner),
        &mut psd as (*mut PolygonScoreData) as (*mut ::std::os::raw::c_void),
        0i32,
    );
    psd.r#ref.x = (*psd.corners.offset(0isize)).x - psd.r#ref.x;
    psd.r#ref.y = (*psd.corners.offset(0isize)).y - psd.r#ref.y;
    i = 0i32;
    'loop1: loop {
        if !(i < 4i32) {
            break;
        }
        memcpy(
            &mut *psd.corners.offset(i as (isize)) as (*mut Point) as (*mut ::std::os::raw::c_void),
            &mut (*region).seed as (*mut Point) as (*const ::std::os::raw::c_void),
            ::std::mem::size_of::<Point>(),
        );
        i = i + 1;
    }
    i = (*region).seed.x * psd.r#ref.x + (*region).seed.y * psd.r#ref.y;
    psd.scores[0usize] = i;
    psd.scores[2usize] = -i;
    i = (*region).seed.x * -psd.r#ref.y + (*region).seed.y * psd.r#ref.x;
    psd.scores[1usize] = i;
    psd.scores[3usize] = -i;
    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        PIXEL_BLACK,
        rcode,
        Some(find_other_corners),
        &mut psd as (*mut PolygonScoreData) as (*mut ::std::os::raw::c_void),
        0i32,
    );
}

pub unsafe extern "C" fn record_capstone(q: &mut Quirc, ring: i32, stone: i32) {
    let mut stone_reg: *mut Region = &mut q.regions[stone as (usize)];
    let mut ring_reg: *mut Region = &mut q.regions[ring as (usize)];
    let mut capstone: *mut Capstone;
    let cs_index: i32;
    if q.num_capstones >= MAX_CAPSTONES as i32 {
    } else {
        cs_index = q.num_capstones;
        capstone = &mut q.capstones[{
            let _old = q.num_capstones;
            q.num_capstones = q.num_capstones + 1;
            _old
        } as (usize)];
        memset(
            capstone as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<Capstone>(),
        );
        (*capstone).qr_grid = -1i32;
        (*capstone).ring = ring;
        (*capstone).stone = stone;
        (*stone_reg).capstone = cs_index;
        (*ring_reg).capstone = cs_index;

        // Find the corners of the ring
        find_region_corners(
            q,
            ring,
            &mut (*stone_reg).seed,
            (*capstone).corners.as_mut_ptr(),
        );

        // Set up the perspective transform and find the center
        (*capstone).c = perspective_setup(&(*capstone).corners, 7.0f64, 7.0f64);
        (*capstone).center = perspective_map(&(*capstone).c, 3.5f64, 3.5f64);
    }
}

pub unsafe extern "C" fn test_capstone(q: &mut Quirc, x: i32, y: i32, pb: *mut i32) {
    let ring_right: i32 = region_code(q, x - *pb.offset(4isize), y);
    let stone: i32 = region_code(
        q,
        x - *pb.offset(4isize) - *pb.offset(3isize) - *pb.offset(2isize),
        y,
    );
    let ring_left: i32 = region_code(
        q,
        x - *pb.offset(4isize)
            - *pb.offset(3isize)
            - *pb.offset(2isize)
            - *pb.offset(1isize)
            - *pb.offset(0isize),
        y,
    );
    let stone_reg: *mut Region;
    let ring_reg: *mut Region;
    let ratio: i32;
    if ring_left < 0i32 || ring_right < 0i32 || stone < 0i32 {
        return;
    }

    // Left and ring of ring should be connected
    if ring_left != ring_right {
        return;
    }

    // Ring should be disconnected from stone
    if ring_left == stone {
        return;
    }

    stone_reg = &mut q.regions[stone as (usize)] as (*mut Region);
    ring_reg = &mut q.regions[ring_left as (usize)] as (*mut Region);

    // Already detected
    if (*stone_reg).capstone >= 0i32 || (*ring_reg).capstone >= 0i32 {
        return;
    }

    // Ratio should ideally be 37.5
    ratio = (*stone_reg).count * 100i32 / (*ring_reg).count;
    if ratio < 10i32 || ratio > 70i32 {
        return;
    }

    record_capstone(q, ring_left, stone);
}

pub unsafe extern "C" fn finder_scan(q: &mut Quirc, y: i32) {
    let row: *mut u8 = q.pixels.as_mut_ptr().offset((y * q.w) as isize);
    let mut x: i32;
    let mut last_color: i32 = 0i32;
    let mut run_length: i32 = 0i32;
    let mut run_count: i32 = 0i32;
    let mut pb: [i32; 5] = [0i32; 5];
    x = 0i32;
    'loop1: loop {
        if !(x < q.w) {
            break;
        }
        let color: i32 = if *row.offset(x as (isize)) != 0 {
            1i32
        } else {
            0i32
        };
        if x != 0 && (color != last_color) {
            memmove(
                pb.as_mut_ptr() as (*mut ::std::os::raw::c_void),
                pb.as_mut_ptr().offset(1isize) as (*const ::std::os::raw::c_void),
                ::std::mem::size_of::<i32>().wrapping_mul(4usize),
            );
            pb[4usize] = run_length;
            run_length = 0i32;
            run_count = run_count + 1;
            if color == 0 && (run_count >= 5i32) {
                static mut CHECK: [i32; 5] = [1i32, 1i32, 3i32, 1i32, 1i32];
                let avg: i32;
                let err: i32;
                let mut i: i32;
                let mut ok: i32 = 1i32;
                avg = (pb[0usize] + pb[1usize] + pb[3usize] + pb[4usize]) / 4i32;
                err = avg * 3i32 / 4i32;
                i = 0i32;
                'loop6: loop {
                    if !(i < 5i32) {
                        break;
                    }
                    if pb[i as (usize)] < CHECK[i as (usize)] * avg - err
                        || pb[i as (usize)] > CHECK[i as (usize)] * avg + err
                    {
                        ok = 0i32;
                    }
                    i = i + 1;
                }
                if ok != 0 {
                    test_capstone(q, x, y, pb.as_mut_ptr());
                }
            }
        }
        run_length = run_length + 1;
        last_color = color;
        x = x + 1;
    }
}

pub unsafe extern "C" fn find_alignment_pattern(q: &mut Quirc, index: i32) {
    let mut qr: *mut Grid = &mut q.grids[index as (usize)];
    let c0: *mut Capstone = &mut q.capstones[(*qr).caps[0usize] as (usize)];
    let c2: *mut Capstone = &mut q.capstones[(*qr).caps[2usize] as (usize)];
    let mut b: Point = std::mem::uninitialized();
    let size_estimate: i32;
    let mut step_size: i32 = 1i32;
    let mut dir: i32 = 0i32;

    // Grab our previous estimate of the alignment pattern corner
    memcpy(
        &mut b as (*mut Point) as (*mut ::std::os::raw::c_void),
        &mut (*qr).align as (*mut Point) as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<Point>(),
    );

    // Guess another two corners of the alignment pattern so that we
    // can estimate its size.
    let (u, v) = perspective_unmap(&(*c0).c, &mut b);
    let a = perspective_map(&(*c0).c, u, v + 1.0f64);
    let (u, v) = perspective_unmap(&(*c2).c, &mut b);
    let c = perspective_map(&(*c2).c, u + 1.0f64, v);

    size_estimate = abs((a.x - b.x) * -(c.y - b.y) + (a.y - b.y) * (c.x - b.x));

    // Spiral outwards from the estimate point until we find something
    // roughly the right size. Don't look too far from the estimate
    // point.
    while step_size * step_size < size_estimate * 100 {
        static mut DX_MAP: [i32; 4] = [1, 0, -1, 0];
        static mut DY_MAP: [i32; 4] = [0, -1, 0, 1];

        for _ in 0..step_size {
            let code: i32 = region_code(q, b.x, b.y);
            if code >= 0i32 {
                let reg: *mut Region = &mut q.regions[code as (usize)];
                if (*reg).count >= size_estimate / 2i32 && ((*reg).count <= size_estimate * 2i32) {
                    (*qr).align_region = code;
                    return;
                }
            }
            b.x = b.x + DX_MAP[dir as (usize)];
            b.y = b.y + DY_MAP[dir as (usize)];
        }
        dir = (dir + 1i32) % 4i32;
        if dir & 1i32 == 0 {
            step_size = step_size + 1;
        }
    }
}

pub unsafe extern "C" fn find_leftmost_to_line(
    user_data: *mut ::std::os::raw::c_void,
    y: i32,
    left: i32,
    right: i32,
) {
    let mut psd: *mut PolygonScoreData = user_data as (*mut PolygonScoreData);
    let xs: [i32; 2] = [left, right];
    let mut i: i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 2i32) {
            break;
        }
        let d: i32 = -(*psd).r#ref.y * xs[i as (usize)] + (*psd).r#ref.x * y;
        if d < (*psd).scores[0usize] {
            (*psd).scores[0usize] = d;
            (*(*psd).corners.offset(0isize)).x = xs[i as (usize)];
            (*(*psd).corners.offset(0isize)).y = y;
        }
        i = i + 1;
    }
}

/// Do a Bresenham scan from one point to another and count the number
/// of black/white transitions.
pub unsafe extern "C" fn timing_scan(q: &Quirc, p0: *const Point, p1: *const Point) -> i32 {
    let mut n: i32 = (*p1).x - (*p0).x;
    let mut d: i32 = (*p1).y - (*p0).y;
    let mut x: i32 = (*p0).x;
    let mut y: i32 = (*p0).y;
    let dom: *mut i32;
    let nondom: *mut i32;
    let dom_step: i32;
    let nondom_step: i32;
    let mut a: i32 = 0i32;
    let mut i: i32;
    let mut run_length: i32 = 0i32;
    let mut count: i32 = 0i32;
    if (*p0).x < 0i32 || (*p0).y < 0i32 || (*p0).x >= q.w || (*p0).y >= q.h {
        -1i32
    } else if (*p1).x < 0i32 || (*p1).y < 0i32 || (*p1).x >= q.w || (*p1).y >= q.h {
        -1i32
    } else {
        if abs(n) > abs(d) {
            let swap: i32 = n;
            n = d;
            d = swap;
            dom = &mut x as (*mut i32);
            nondom = &mut y as (*mut i32);
        } else {
            dom = &mut y as (*mut i32);
            nondom = &mut x as (*mut i32);
        }
        if n < 0i32 {
            n = -n;
            nondom_step = -1i32;
        } else {
            nondom_step = 1i32;
        }
        if d < 0i32 {
            d = -d;
            dom_step = -1i32;
        } else {
            dom_step = 1i32;
        }
        x = (*p0).x;
        y = (*p0).y;
        i = 0i32;
        'loop12: loop {
            if !(i <= d) {
                break;
            }
            let pixel: i32;
            if y < 0i32 || y >= q.h || x < 0i32 || x >= q.w {
                break;
            }
            pixel = *q.pixels.as_ptr().offset((y * q.w + x) as isize) as i32;
            if pixel != 0 {
                if run_length >= 2i32 {
                    count = count + 1;
                }
                run_length = 0i32;
            } else {
                run_length = run_length + 1;
            }
            a = a + n;
            *dom = *dom + dom_step;
            if a >= d {
                *nondom = *nondom + nondom_step;
                a = a - d;
            }
            i = i + 1;
        }
        count
    }
}

/// Try the measure the timing pattern for a given QR code. This does
/// not require the global perspective to have been set up, but it
/// does require that the capstone corners have been set to their
/// canonical rotation.
///
/// For each capstone, we find a point in the middle of the ring band
/// which is nearest the centre of the code. Using these points, we do
/// a horizontal and a vertical timing scan.
pub unsafe extern "C" fn measure_timing_pattern(q: &mut Quirc, index: i32) -> i32 {
    let mut qr: *mut Grid = &mut q.grids[index as (usize)];
    let mut i: i32;
    let mut scan: i32;
    let ver: i32;
    let size: i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 3i32) {
            break;
        }
        static mut US: [f64; 3] = [6.5, 6.5, 0.5];
        static mut VS: [f64; 3] = [0.5, 6.5, 6.5];
        let cap: *mut Capstone = &mut q.capstones[(*qr).caps[i as (usize)] as (usize)];
        (*qr).tpep[i as (usize)] = perspective_map(&(*cap).c, US[i as (usize)], VS[i as (usize)]);
        i = i + 1;
    }
    (*qr).hscan = timing_scan(q, &mut (*qr).tpep[1usize], &mut (*qr).tpep[2usize]);
    (*qr).vscan = timing_scan(q, &mut (*qr).tpep[1usize], &mut (*qr).tpep[0usize]);
    scan = (*qr).hscan;
    if (*qr).vscan > scan {
        scan = (*qr).vscan;
    }

    // If neither scan worked, we can't go any further.
    if scan < 0i32 {
        return -1i32;
    }

    // Choose the nearest allowable grid size
    size = scan * 2i32 + 13i32;
    ver = (size - 15i32) / 4i32;
    (*qr).grid_size = ver * 4i32 + 17i32;

    return 0i32;
}

/// Read a cell from a grid using the currently set perspective
/// transform. Returns +/- 1 for black/white, 0 for cells which are
/// out of image bounds.
pub unsafe extern "C" fn read_cell(q: &mut Quirc, index: i32, x: i32, y: i32) -> i32 {
    let qr: *mut Grid = &mut q.grids[index as (usize)];
    let p = perspective_map(&(*qr).c, x as (f64) + 0.5f64, y as (f64) + 0.5f64);
    if p.y < 0i32 || p.y >= q.h || p.x < 0i32 || p.x >= q.w {
        0i32
    } else if *q.pixels.as_mut_ptr().offset((p.y * q.w + p.x) as isize) != 0 {
        1i32
    } else {
        -1i32
    }
}

pub unsafe extern "C" fn fitness_cell(q: &mut Quirc, index: i32, x: i32, y: i32) -> i32 {
    let qr: *mut Grid = &mut q.grids[index as (usize)];
    let mut score: i32 = 0i32;
    let mut u: i32;
    let mut v: i32;
    v = 0i32;
    'loop1: loop {
        if !(v < 3i32) {
            break;
        }
        u = 0i32;
        'loop4: loop {
            if !(u < 3i32) {
                break;
            }
            static mut OFFSETS: [f64; 3] = [0.3, 0.5, 0.7];
            let p = perspective_map(
                &(*qr).c,
                x as (f64) + OFFSETS[u as (usize)],
                y as (f64) + OFFSETS[v as (usize)],
            );
            if !(p.y < 0i32 || p.y >= q.h || p.x < 0i32 || p.x >= q.w) {
                if *q.pixels.as_mut_ptr().offset((p.y * q.w + p.x) as isize) != 0 {
                    score = score + 1;
                } else {
                    score = score - 1;
                }
            }
            u = u + 1;
        }
        v = v + 1;
    }
    score
}

pub unsafe extern "C" fn fitness_ring(
    q: &mut Quirc,
    index: i32,
    cx: i32,
    cy: i32,
    radius: i32,
) -> i32 {
    let mut i: i32;
    let mut score: i32 = 0i32;
    i = 0i32;
    'loop1: loop {
        if !(i < radius * 2i32) {
            break;
        }
        score = score + fitness_cell(q, index, cx - radius + i, cy - radius);
        score = score + fitness_cell(q, index, cx - radius, cy + radius - i);
        score = score + fitness_cell(q, index, cx + radius, cy - radius + i);
        score = score + fitness_cell(q, index, cx + radius - i, cy + radius);
        i = i + 1;
    }
    score
}

pub unsafe extern "C" fn fitness_apat(q: &mut Quirc, index: i32, cx: i32, cy: i32) -> i32 {
    fitness_cell(q, index, cx, cy) - fitness_ring(q, index, cx, cy, 1i32)
        + fitness_ring(q, index, cx, cy, 2i32)
}

pub unsafe extern "C" fn fitness_capstone(
    q: &mut Quirc,
    index: i32,
    mut x: i32,
    mut y: i32,
) -> i32 {
    x = x + 3i32;
    y = y + 3i32;
    fitness_cell(q, index, x, y) + fitness_ring(q, index, x, y, 1i32)
        - fitness_ring(q, index, x, y, 2i32)
        + fitness_ring(q, index, x, y, 3i32)
}

/// Compute a fitness score for the currently configured perspective
/// transform, using the features we expect to find by scanning the
/// grid.
pub unsafe extern "C" fn fitness_all(q: &mut Quirc, index: i32) -> i32 {
    let qr: *const Grid = &mut q.grids[index as (usize)];
    let version: i32 = ((*qr).grid_size - 17i32) / 4i32;
    let info: *const VersionInfo = &VERSION_DB[version as (usize)];
    let mut score: i32 = 0i32;
    let mut i: i32;
    let mut j: i32;
    let mut ap_count: i32;

    // Check the timing pattern
    i = 0i32;
    'loop1: loop {
        if !(i < (*qr).grid_size - 14i32) {
            break;
        }
        let expect: i32 = if i & 1i32 != 0 { 1i32 } else { -1i32 };
        score = score + fitness_cell(q, index, i + 7i32, 6i32) * expect;
        score = score + fitness_cell(q, index, 6i32, i + 7i32) * expect;
        i = i + 1;
    }

    // Check capstones
    score = score + fitness_capstone(q, index, 0i32, 0i32);
    score = score + fitness_capstone(q, index, (*qr).grid_size - 7i32, 0i32);
    score = score + fitness_capstone(q, index, 0i32, (*qr).grid_size - 7i32);

    if version < 0i32 || version > QUIRC_MAX_VERSION as i32 {
        score
    } else {
        // Check alignment patterns
        ap_count = 0i32;
        'loop4: loop {
            if !(ap_count < QUIRC_MAX_ALIGNMENT as i32 && ((*info).apat[ap_count as (usize)] != 0))
            {
                break;
            }
            ap_count = ap_count + 1;
        }
        i = 1i32;
        'loop6: loop {
            if !(i + 1i32 < ap_count) {
                break;
            }
            score = score + fitness_apat(q, index, 6i32, (*info).apat[i as (usize)]);
            score = score + fitness_apat(q, index, (*info).apat[i as (usize)], 6i32);
            i = i + 1;
        }
        i = 1i32;
        'loop8: loop {
            if !(i < ap_count) {
                break;
            }
            j = 1i32;
            'loop11: loop {
                if !(j < ap_count) {
                    break;
                }
                score = score
                    + fitness_apat(
                        q,
                        index,
                        (*info).apat[i as (usize)],
                        (*info).apat[j as (usize)],
                    );
                j = j + 1;
            }
            i = i + 1;
        }
        score
    }
}

pub unsafe extern "C" fn jiggle_perspective(q: &mut Quirc, index: i32) {
    let mut qr: *mut Grid = &mut q.grids[index as (usize)];
    let mut best: i32 = fitness_all(q, index);
    let mut pass: i32;
    let mut adjustments: [f64; 8] = [0f64; 8];
    let mut i: i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 8i32) {
            break;
        }
        adjustments[i as (usize)] = (*qr).c[i as (usize)] * 0.02f64;
        i = i + 1;
    }
    pass = 0i32;
    'loop3: loop {
        if !(pass < 5i32) {
            break;
        }
        i = 0i32;
        'loop6: loop {
            if !(i < 16i32) {
                break;
            }
            let j: i32 = i >> 1i32;
            let test: i32;
            let old: f64 = (*qr).c[j as (usize)];
            let step: f64 = adjustments[j as (usize)];
            let new: f64;
            if i & 1i32 != 0 {
                new = old + step;
            } else {
                new = old - step;
            }
            (*qr).c[j as (usize)] = new;
            test = fitness_all(q, index);
            if test > best {
                best = test;
            } else {
                (*qr).c[j as (usize)] = old;
            }
            i = i + 1;
        }
        i = 0i32;
        'loop8: loop {
            if !(i < 8i32) {
                break;
            }
            let _rhs = 0.5f64;
            let _lhs = &mut adjustments[i as (usize)];
            *_lhs = *_lhs * _rhs;
            i = i + 1;
        }
        pass = pass + 1;
    }
}

/// Once the capstones are in place and an alignment point has been
/// chosen, we call this function to set up a grid-reading perspective
/// transform.
pub unsafe extern "C" fn setup_qr_perspective(q: &mut Quirc, index: i32) {
    let qr: *mut Grid = &mut q.grids[index as (usize)];
    let mut rect: [Point; 4] = std::mem::uninitialized();

    // Set up the perspective map for reading the grid
    memcpy(
        &mut rect[0usize] as (*mut Point) as (*mut ::std::os::raw::c_void),
        &mut q.capstones[(*qr).caps[1usize] as (usize)].corners[0usize] as (*mut Point)
            as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<Point>(),
    );
    memcpy(
        &mut rect[1usize] as (*mut Point) as (*mut ::std::os::raw::c_void),
        &mut q.capstones[(*qr).caps[2usize] as (usize)].corners[0usize] as (*mut Point)
            as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<Point>(),
    );
    memcpy(
        &mut rect[2usize] as (*mut Point) as (*mut ::std::os::raw::c_void),
        &mut (*qr).align as (*mut Point) as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<Point>(),
    );
    memcpy(
        &mut rect[3usize] as (*mut Point) as (*mut ::std::os::raw::c_void),
        &mut q.capstones[(*qr).caps[0usize] as (usize)].corners[0usize] as (*mut Point)
            as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<Point>(),
    );
    (*qr).c = perspective_setup(
        &rect,
        ((*qr).grid_size - 7i32) as (f64),
        ((*qr).grid_size - 7i32) as (f64),
    );
    jiggle_perspective(q, index);
}

/// Rotate the capstone with so that corner 0 is the leftmost with respect
/// to the given reference line.
pub unsafe extern "C" fn rotate_capstone(cap: *mut Capstone, h0: *const Point, hd: *const Point) {
    let mut copy: [Point; 4] = std::mem::uninitialized();
    let mut j: i32;
    let mut best: i32 = std::mem::uninitialized();
    let mut best_score: i32 = std::mem::uninitialized();
    j = 0i32;
    'loop1: loop {
        if !(j < 4i32) {
            break;
        }
        let p: *mut Point = &mut (*cap).corners[j as (usize)];
        let score: i32 = ((*p).x - (*h0).x) * -(*hd).y + ((*p).y - (*h0).y) * (*hd).x;
        if j == 0 || score < best_score {
            best = j;
            best_score = score;
        }
        j = j + 1;
    }

    // Rotate the capstone
    j = 0i32;
    'loop3: loop {
        if !(j < 4i32) {
            break;
        }
        memcpy(
            &mut copy[j as (usize)] as (*mut Point) as (*mut ::std::os::raw::c_void),
            &mut (*cap).corners[((j + best) % 4i32) as (usize)] as (*mut Point)
                as (*const ::std::os::raw::c_void),
            ::std::mem::size_of::<Point>(),
        );
        j = j + 1;
    }
    memcpy(
        (*cap).corners.as_mut_ptr() as (*mut ::std::os::raw::c_void),
        copy.as_mut_ptr() as (*const ::std::os::raw::c_void),
        ::std::mem::size_of::<[Point; 4]>(),
    );
    (*cap).c = perspective_setup(&(*cap).corners, 7.0f64, 7.0f64);
}

pub unsafe extern "C" fn record_qr_grid(mut q: &mut Quirc, mut a: i32, b: i32, mut c: i32) {
    let mut h0: Point = std::mem::uninitialized();
    let mut hd: Point = std::mem::uninitialized();
    let mut i: i32;
    let qr_index: i32;
    let mut qr: *mut Grid;
    if q.num_grids >= MAX_GRIDS as i32 {
    } else {
        // Construct the hypotenuse line from A to C. B should be to
        // the left of this line.
        memcpy(
            &mut h0 as (*mut Point) as (*mut ::std::os::raw::c_void),
            &mut q.capstones[a as (usize)].center as (*mut Point)
                as (*const ::std::os::raw::c_void),
            ::std::mem::size_of::<Point>(),
        );
        hd.x = q.capstones[c as (usize)].center.x - q.capstones[a as (usize)].center.x;
        hd.y = q.capstones[c as (usize)].center.y - q.capstones[a as (usize)].center.y;

        // Make sure A-B-C is clockwise
        if (q.capstones[b as (usize)].center.x - h0.x) * -hd.y
            + (q.capstones[b as (usize)].center.y - h0.y) * hd.x
            > 0i32
        {
            let swap: i32 = a;
            a = c;
            c = swap;
            hd.x = -hd.x;
            hd.y = -hd.y;
        }

        // Record the grid and its components
        qr_index = q.num_grids;
        qr = &mut q.grids[{
            let _old = q.num_grids;
            q.num_grids = q.num_grids + 1;
            _old
        } as (usize)];
        memset(
            qr as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<Grid>(),
        );
        (*qr).caps[0usize] = a;
        (*qr).caps[1usize] = b;
        (*qr).caps[2usize] = c;
        (*qr).align_region = -1i32;

        // Rotate each capstone so that corner 0 is top-left with respect
        // to the grid.
        i = 0i32;
        'loop4: loop {
            if !(i < 3i32) {
                break;
            }
            let mut cap: *mut Capstone = &mut q.capstones[(*qr).caps[i as (usize)] as (usize)];
            rotate_capstone(cap, &mut h0, &mut hd);
            (*cap).qr_grid = qr_index;
            i = i + 1;
        }

        // Check the timing pattern. This doesn't require a perspective
        // transform.
        if !(measure_timing_pattern(q, qr_index) < 0i32) {
            // Make an estimate based for the alignment pattern based on extending
            // lines from capstones A and C.
            if !(line_intersect(
                &q.capstones[a as (usize)].corners[0usize],
                &q.capstones[a as (usize)].corners[1usize],
                &q.capstones[c as (usize)].corners[0usize],
                &q.capstones[c as (usize)].corners[3usize],
                &mut (*qr).align,
            ) == 0)
            {
                // On V2+ grids, we should use the alignment pattern.
                if (*qr).grid_size > 21i32 {
                    // Try to find the actual location of the alignment pattern.
                    find_alignment_pattern(q, qr_index);

                    // Find the point of the alignment pattern closest to the
                    // top-left of the QR grid.
                    if (*qr).align_region >= 0i32 {
                        let mut psd: PolygonScoreData = std::mem::uninitialized();
                        let reg: *mut Region = &mut q.regions[(*qr).align_region as (usize)];

                        // Start from some point inside the alignment pattern
                        memcpy(
                            &mut (*qr).align as (*mut Point) as (*mut ::std::os::raw::c_void),
                            &mut (*reg).seed as (*mut Point) as (*const ::std::os::raw::c_void),
                            ::std::mem::size_of::<Point>(),
                        );

                        memcpy(
                            &mut psd.r#ref as (*mut Point) as (*mut ::std::os::raw::c_void),
                            &mut hd as (*mut Point) as (*const ::std::os::raw::c_void),
                            ::std::mem::size_of::<Point>(),
                        );
                        psd.corners = &mut (*qr).align;
                        psd.scores[0usize] = -hd.y * (*qr).align.x + hd.x * (*qr).align.y;
                        flood_fill_seed(
                            q,
                            (*reg).seed.x,
                            (*reg).seed.y,
                            (*qr).align_region,
                            PIXEL_BLACK,
                            None,
                            0i32 as (*mut ::std::os::raw::c_void),
                            0i32,
                        );
                        flood_fill_seed(
                            q,
                            (*reg).seed.x,
                            (*reg).seed.y,
                            PIXEL_BLACK,
                            (*qr).align_region,
                            Some(find_leftmost_to_line),
                            &mut psd as (*mut PolygonScoreData) as (*mut ::std::os::raw::c_void),
                            0i32,
                        );
                    }
                }
                setup_qr_perspective(q, qr_index);
                return;
            }
        }

        // We've been unable to complete setup for this grid. Undo what we've
        // recorded and pretend it never happened.
        i = 0i32;
        'loop12: loop {
            if !(i < 3i32) {
                break;
            }
            q.capstones[(*qr).caps[i as (usize)] as (usize)].qr_grid = -1i32;
            i = i + 1;
        }
        q.num_grids = q.num_grids - 1;
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct Neighbour {
    pub index: i32,
    pub distance: f64,
}

impl Clone for Neighbour {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct NeighbourList {
    pub n: [Neighbour; MAX_CAPSTONES],
    pub count: i32,
}

impl Clone for NeighbourList {
    fn clone(&self) -> Self {
        *self
    }
}

pub unsafe extern "C" fn test_neighbours(
    q: &mut Quirc,
    i: i32,
    hlist: *mut NeighbourList,
    vlist: *mut NeighbourList,
) {
    let mut j: i32;
    let mut k: i32;
    let mut best_score: f64 = 0.0f64;
    let mut best_h: i32 = -1i32;
    let mut best_v: i32 = -1i32;

    // Test each possible grouping
    j = 0i32;
    'loop1: loop {
        if !(j < (*hlist).count) {
            break;
        }
        k = 0i32;
        'loop6: loop {
            if !(k < (*vlist).count) {
                break;
            }
            let hn: *const Neighbour = &mut (*hlist).n[j as (usize)];
            let vn: *const Neighbour = &mut (*vlist).n[k as (usize)];
            let score: f64 = (1.0 - (*hn).distance / (*vn).distance).abs();
            if !(score > 2.5f64) {
                if best_h < 0i32 || score < best_score {
                    best_h = (*hn).index;
                    best_v = (*vn).index;
                    best_score = score;
                }
            }
            k = k + 1;
        }
        j = j + 1;
    }
    if best_h < 0i32 || best_v < 0i32 {
    } else {
        record_qr_grid(q, best_h, i, best_v);
    }
}

pub unsafe extern "C" fn test_grouping(q: &mut Quirc, i: i32) {
    let c1: *mut Capstone = &mut q.capstones[i as (usize)];
    let mut j: i32;
    let mut hlist: NeighbourList = std::mem::uninitialized();
    let mut vlist: NeighbourList = std::mem::uninitialized();
    if (*c1).qr_grid >= 0i32 {
    } else {
        hlist.count = 0i32;
        vlist.count = 0i32;

        // Look for potential neighbours by examining the relative gradients
        // from this capstone to others.
        j = 0i32;
        'loop2: loop {
            if !(j < q.num_capstones) {
                break;
            }
            let c2: *mut Capstone = &mut q.capstones[j as (usize)];
            if !(i == j || (*c2).qr_grid >= 0i32) {
                let (mut u, mut v) = perspective_unmap(&(*c1).c, &mut (*c2).center);

                u = (u - 3.5).abs();
                v = (v - 3.5).abs();

                if u < 0.2f64 * v {
                    let mut n: *mut Neighbour = &mut hlist.n[{
                        let _old = hlist.count;
                        hlist.count = hlist.count + 1;
                        _old
                    } as (usize)];
                    (*n).index = j;
                    (*n).distance = v;
                }
                if v < 0.2f64 * u {
                    let mut n: *mut Neighbour = &mut vlist.n[{
                        let _old = vlist.count;
                        vlist.count = vlist.count + 1;
                        _old
                    } as (usize)];
                    (*n).index = j;
                    (*n).distance = u;
                }
            }
            j = j + 1;
        }
        (if !(hlist.count != 0 && (vlist.count != 0)) {
        } else {
            test_neighbours(q, i, &mut hlist, &mut vlist);
        })
    }
}

pub unsafe extern "C" fn pixels_setup(q: &mut Quirc) {
    q.pixels.copy_from_slice(&q.image);
}

/// These functions are used to process images for QR-code recognition.
/// quirc_begin() must first be called to obtain access to a buffer into
/// which the input image should be placed. Optionally, the current
/// width and height may be returned.
///
/// After filling the buffer, quirc_end() should be called to process
/// the image for QR-code recognition. The locations and content of each
/// code may be obtained using accessor functions described below.
pub unsafe extern "C" fn quirc_begin(q: &mut Quirc, w: *mut i32, h: *mut i32) -> &mut [u8] {
    q.num_regions = PIXEL_REGION;
    q.num_capstones = 0i32;
    q.num_grids = 0i32;
    if !w.is_null() {
        *w = q.w;
    }
    if !h.is_null() {
        *h = q.h;
    }
    q.image.as_mut()
}

pub unsafe extern "C" fn quirc_end(q: &mut Quirc) {
    let mut i: i32;
    pixels_setup(q);
    threshold(q);
    i = 0i32;
    'loop1: loop {
        if !(i < q.h) {
            break;
        }
        finder_scan(q, i);
        i = i + 1;
    }
    i = 0i32;
    'loop3: loop {
        if !(i < q.num_capstones) {
            break;
        }
        test_grouping(q, i);
        i = i + 1;
    }
}

/// Extract the QR-code specified by the given index.
pub unsafe extern "C" fn quirc_extract(q: &mut Quirc, index: i32, mut code: *mut QuircCode) {
    let qr: *mut Grid = &mut q.grids[index as (usize)];
    let mut y: i32;
    let mut i: i32 = 0i32;
    if index < 0i32 || index > q.num_grids {
    } else {
        memset(
            code as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<QuircCode>(),
        );
        (*code).corners[0usize] = perspective_map(&(*qr).c, 0.0f64, 0.0f64);
        (*code).corners[1usize] = perspective_map(&(*qr).c, (*qr).grid_size as (f64), 0.0f64);
        (*code).corners[2usize] =
            perspective_map(&(*qr).c, (*qr).grid_size as (f64), (*qr).grid_size as (f64));
        (*code).corners[3usize] = perspective_map(&(*qr).c, 0.0f64, (*qr).grid_size as (f64));
        (*code).size = (*qr).grid_size;
        y = 0i32;
        'loop2: loop {
            if !(y < (*qr).grid_size) {
                break;
            }
            let mut x: i32;
            x = 0i32;
            'loop5: loop {
                if !(x < (*qr).grid_size) {
                    break;
                }
                if read_cell(q, index, x, y) > 0i32 {
                    let _rhs = 1i32 << (i & 7i32);
                    let _lhs = &mut (*code).cell_bitmap[(i >> 3i32) as (usize)];
                    *_lhs = (*_lhs as (i32) | _rhs) as (u8);
                }
                i = i + 1;
                x = x + 1;
            }
            y = y + 1;
        }
    }
}
