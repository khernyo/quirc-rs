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

use std::cmp::max;
use std::os::raw::c_double;

extern "C" {
    fn abs(__x: i32) -> i32;
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
        0
    } else {
        r.x = (d * e - b * f) / det;
        r.y = (-c * e + a * f) / det;
        1
    }
}

fn perspective_setup(rect: &[Point; 4], w: f64, h: f64) -> [f64; 8] {
    let x0: f64 = rect[0].x as f64;
    let y0: f64 = rect[0].y as f64;
    let x1: f64 = rect[1].x as f64;
    let y1: f64 = rect[1].y as f64;
    let x2: f64 = rect[2].x as f64;
    let y2: f64 = rect[2].y as f64;
    let x3: f64 = rect[3].x as f64;
    let y3: f64 = rect[3].y as f64;
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

type SpanFunc<T> = unsafe fn(user_data: *mut T, y: i32, left: i32, right: i32);

/// Span-based floodfill routine
unsafe fn flood_fill_seed<T>(
    q: &mut Quirc,
    x: i32,
    y: i32,
    from: i32,
    to: i32,
    func: Option<SpanFunc<T>>,
    user_data: *mut T,
    depth: i32,
) {
    let mut left: i32 = x;
    let mut right: i32 = x;
    let mut row: usize = (y * q.w) as usize;
    if depth >= FLOOD_FILL_MAX_DEPTH {
        return;
    }

    while left > 0 && (q.pixels[row + (left - 1) as usize] as i32 == from) {
        left = left - 1;
    }

    while right < q.w - 1 && (q.pixels[row + (right + 1) as usize] as i32 == from) {
        right = right + 1;
    }

    // Fill the extent
    // TODO Use a simple for statement (currently, it causes a stack overflow during tests)
    let mut i = left;
    while i <= right {
        q.pixels[row + i as usize] = to as u8;
        i += 1;
    }

    if let Some(f) = func {
        f(user_data, y, left, right);
    }

    // Seed new flood-fills
    if y > 0 {
        row = ((y - 1) * q.w) as usize;

        let mut i = left;
        while i <= right {
            if q.pixels[row + i as usize] as i32 == from {
                flood_fill_seed(q, i, y - 1, from, to, func, user_data, depth + 1);
            }
            i += 1;
        }
    }

    if y < q.h - 1 {
        row = ((y + 1) * q.w) as usize;

        let mut i = left;
        while i <= right {
            if q.pixels[row + i as usize] as i32 == from {
                flood_fill_seed(q, i, y + 1, from, to, func, user_data, depth + 1);
            }
            i += 1;
        }
    }
}

const THRESHOLD_S_MIN: i32 = 1;
const THRESHOLD_S_DEN: i32 = 8;
const THRESHOLD_T: i32 = 5;

/// Adaptive thresholding
fn threshold(q: &mut Quirc) {
    let mut avg_w: i32 = 0;
    let mut avg_u: i32 = 0;
    let mut threshold_s: i32 = q.w / THRESHOLD_S_DEN;
    let mut row: usize = 0;

    // Ensure a sane, non-zero value for threshold_s.
    //
    // threshold_s can be zero if the image width is small. We need to avoid
    // SIGFPE as it will be used as divisor.
    threshold_s = max(threshold_s, THRESHOLD_S_MIN);

    for y in 0..q.h {
        q.row_average.iter_mut().for_each(|x| *x = 0);

        for x in 0..q.w {
            let w: usize;
            let u: usize;

            if y & 1 != 0 {
                w = x as usize;
                u = (q.w - 1 - x) as usize;
            } else {
                w = (q.w - 1 - x) as usize;
                u = x as usize;
            }

            avg_w = avg_w * (threshold_s - 1) / threshold_s + q.pixels[row + w] as i32;
            avg_u = avg_u * (threshold_s - 1) / threshold_s + q.pixels[row + u] as i32;

            q.row_average[w as usize] += avg_w;
            q.row_average[u as usize] += avg_u;
        }

        for x in 0..q.w {
            if (q.pixels[row + x as usize] as i32)
                < q.row_average[x as usize] * (100 - THRESHOLD_T) / (200 * threshold_s)
            {
                q.pixels[row + x as usize] = PIXEL_BLACK as u8;
            } else {
                q.pixels[row + x as usize] = PIXEL_WHITE as u8;
            }
        }
        row += q.w as usize;
    }
}

unsafe fn area_count(region: *mut Region, _y: i32, left: i32, right: i32) {
    (*region).count += right - left + 1;
}

unsafe fn region_code(q: &mut Quirc, x: i32, y: i32) -> i32 {
    if x < 0 || y < 0 || x >= q.w || y >= q.h {
        return -1;
    }

    let pixel = q.pixels[(y * q.w + x) as usize] as i32;

    if pixel >= PIXEL_REGION {
        return pixel;
    }

    if pixel == PIXEL_WHITE {
        return -1;
    }

    if q.regions.len() >= MAX_REGIONS {
        return -1;
    }

    let region: i32 = q.regions.len() as i32;
    q.regions.push(Region {
        seed: Point { x, y },
        capstone: -1,
        ..Default::default()
    });
    let r#box: *mut Region = q.regions.last_mut().unwrap();

    flood_fill_seed(q, x, y, pixel, region, Some(area_count), r#box, 0);

    region
}

#[derive(Copy)]
#[repr(C)]
struct PolygonScoreData {
    r#ref: Point,
    scores: [i32; 4],
    corners: *mut Point,
}

impl Clone for PolygonScoreData {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe fn find_one_corner(psd: *mut PolygonScoreData, y: i32, left: i32, right: i32) {
    let xs: [i32; 2] = [left, right];
    let dy: i32 = y - (*psd).r#ref.y;

    for i in 0..2 {
        let dx: i32 = xs[i as usize] - (*psd).r#ref.x;
        let d: i32 = dx * dx + dy * dy;

        if d > (*psd).scores[0] {
            (*psd).scores[0] = d;
            (*(*psd).corners.offset(0)).x = xs[i as usize];
            (*(*psd).corners.offset(0)).y = y;
        }
    }
}

unsafe fn find_other_corners(psd: *mut PolygonScoreData, y: i32, left: i32, right: i32) {
    let xs: [i32; 2] = [left, right];

    for i in 0..2 {
        let up: i32 = xs[i as usize] * (*psd).r#ref.x + y * (*psd).r#ref.y;
        let right: i32 = xs[i as usize] * -(*psd).r#ref.y + y * (*psd).r#ref.x;
        let scores: [i32; 4] = [up, right, -up, -right];

        for j in 0..4 {
            if scores[j as usize] > (*psd).scores[j as usize] {
                (*psd).scores[j as usize] = scores[j as usize];
                (*(*psd).corners.offset(j as isize)).x = xs[i as usize];
                (*(*psd).corners.offset(j as isize)).y = y;
            }
        }
    }
}

unsafe fn find_region_corners(q: &mut Quirc, rcode: i32, r#ref: *const Point, corners: *mut Point) {
    let region: *mut Region = &mut q.regions[rcode as usize];
    let mut psd: PolygonScoreData = PolygonScoreData {
        r#ref: *r#ref,
        scores: [-1, 0, 0, 0],
        corners,
    };

    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        rcode,
        PIXEL_BLACK,
        Some(find_one_corner),
        &mut psd,
        0,
    );

    psd.r#ref.x = (*psd.corners.offset(0)).x - psd.r#ref.x;
    psd.r#ref.y = (*psd.corners.offset(0)).y - psd.r#ref.y;

    for i in 0..4 {
        *psd.corners.offset(i as isize) = (*region).seed;
    }

    let i = (*region).seed.x * psd.r#ref.x + (*region).seed.y * psd.r#ref.y;
    psd.scores[0] = i;
    psd.scores[2] = -i;
    let i = (*region).seed.x * -psd.r#ref.y + (*region).seed.y * psd.r#ref.x;
    psd.scores[1] = i;
    psd.scores[3] = -i;

    flood_fill_seed(
        q,
        (*region).seed.x,
        (*region).seed.y,
        PIXEL_BLACK,
        rcode,
        Some(find_other_corners),
        &mut psd,
        0,
    );
}

unsafe fn record_capstone(q: &mut Quirc, ring: i32, stone: i32) {
    let mut stone_reg: *mut Region = &mut q.regions[stone as usize];
    let mut ring_reg: *mut Region = &mut q.regions[ring as usize];

    if q.capstones.len() >= MAX_CAPSTONES {
        return;
    }

    let cs_index = q.capstones.len();
    q.capstones.push(Capstone {
        qr_grid: -1,
        ring,
        stone,
        ..Default::default()
    });
    let capstone: *mut Capstone = q.capstones.last_mut().unwrap();
    (*stone_reg).capstone = cs_index as i32;
    (*ring_reg).capstone = cs_index as i32;

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

unsafe fn test_capstone(q: &mut Quirc, x: i32, y: i32, pb: *mut i32) {
    let ring_right: i32 = region_code(q, x - *pb.offset(4), y);
    let stone: i32 = region_code(q, x - *pb.offset(4) - *pb.offset(3) - *pb.offset(2), y);
    let ring_left: i32 = region_code(
        q,
        x - *pb.offset(4) - *pb.offset(3) - *pb.offset(2) - *pb.offset(1) - *pb.offset(0),
        y,
    );

    if ring_left < 0 || ring_right < 0 || stone < 0 {
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

    let stone_reg = &mut q.regions[stone as usize] as *mut Region;
    let ring_reg = &mut q.regions[ring_left as usize] as *mut Region;

    // Already detected
    if (*stone_reg).capstone >= 0 || (*ring_reg).capstone >= 0 {
        return;
    }

    // Ratio should ideally be 37.5
    let ratio = (*stone_reg).count * 100 / (*ring_reg).count;
    if ratio < 10 || ratio > 70 {
        return;
    }

    record_capstone(q, ring_left, stone);
}

unsafe fn finder_scan(q: &mut Quirc, y: i32) {
    let row: usize = (y * q.w) as usize;
    let mut last_color: i32 = 0;
    let mut run_length: i32 = 0;
    let mut run_count: i32 = 0;
    let mut pb: [i32; 5] = [0; 5];

    for x in 0..q.w {
        let color: i32 = if q.pixels[row + x as usize] != 0 {
            1
        } else {
            0
        };

        if x != 0 && (color != last_color) {
            memmove(
                pb.as_mut_ptr() as *mut ::std::os::raw::c_void,
                pb.as_mut_ptr().offset(1) as *const ::std::os::raw::c_void,
                ::std::mem::size_of::<i32>().wrapping_mul(4),
            );
            pb[4] = run_length;
            run_length = 0;
            run_count += 1;

            if color == 0 && (run_count >= 5) {
                const CHECK: [i32; 5] = [1, 1, 3, 1, 1];
                let mut ok: i32 = 1;

                let avg = (pb[0] + pb[1] + pb[3] + pb[4]) / 4;
                let err = avg * 3 / 4;

                for i in 0..5 {
                    if pb[i as usize] < CHECK[i as usize] * avg - err
                        || pb[i as usize] > CHECK[i as usize] * avg + err
                    {
                        ok = 0;
                    }
                }

                if ok != 0 {
                    test_capstone(q, x, y, pb.as_mut_ptr());
                }
            }
        }
        run_length += 1;
        last_color = color;
    }
}

unsafe fn find_alignment_pattern(q: &mut Quirc, index: i32) {
    let mut qr: *mut Grid = &mut q.grids[index as usize];
    let c0: *mut Capstone = &mut q.capstones[(*qr).caps[0] as usize];
    let c2: *mut Capstone = &mut q.capstones[(*qr).caps[2] as usize];
    let mut step_size: i32 = 1;
    let mut dir: i32 = 0;

    // Grab our previous estimate of the alignment pattern corner
    let mut b = (*qr).align;

    // Guess another two corners of the alignment pattern so that we
    // can estimate its size.
    let (u, v) = perspective_unmap(&(*c0).c, &mut b);
    let a = perspective_map(&(*c0).c, u, v + 1.0f64);
    let (u, v) = perspective_unmap(&(*c2).c, &mut b);
    let c = perspective_map(&(*c2).c, u + 1.0f64, v);

    let size_estimate = abs((a.x - b.x) * -(c.y - b.y) + (a.y - b.y) * (c.x - b.x));

    // Spiral outwards from the estimate point until we find something
    // roughly the right size. Don't look too far from the estimate
    // point.
    while step_size * step_size < size_estimate * 100 {
        const DX_MAP: [i32; 4] = [1, 0, -1, 0];
        const DY_MAP: [i32; 4] = [0, -1, 0, 1];

        for _ in 0..step_size {
            let code: i32 = region_code(q, b.x, b.y);
            if code >= 0 {
                let reg: *mut Region = &mut q.regions[code as usize];
                if (*reg).count >= size_estimate / 2 && ((*reg).count <= size_estimate * 2) {
                    (*qr).align_region = code;
                    return;
                }
            }
            b.x = b.x + DX_MAP[dir as usize];
            b.y = b.y + DY_MAP[dir as usize];
        }
        dir = (dir + 1) % 4;
        if dir & 1 == 0 {
            step_size += 1;
        }
    }
}

unsafe fn find_leftmost_to_line(psd: *mut PolygonScoreData, y: i32, left: i32, right: i32) {
    let xs: [i32; 2] = [left, right];

    for i in 0..2 {
        let d: i32 = -(*psd).r#ref.y * xs[i as usize] + (*psd).r#ref.x * y;

        if d < (*psd).scores[0] {
            (*psd).scores[0] = d;
            (*(*psd).corners.offset(0)).x = xs[i as usize];
            (*(*psd).corners.offset(0)).y = y;
        }
    }
}

/// Do a Bresenham scan from one point to another and count the number
/// of black/white transitions.
unsafe fn timing_scan(q: &Quirc, p0: *const Point, p1: *const Point) -> i32 {
    if (*p0).x < 0 || (*p0).y < 0 || (*p0).x >= q.w || (*p0).y >= q.h {
        return -1;
    }
    if (*p1).x < 0 || (*p1).y < 0 || (*p1).x >= q.w || (*p1).y >= q.h {
        return -1;
    }

    let mut n: i32 = (*p1).x - (*p0).x;
    let mut d: i32 = (*p1).y - (*p0).y;
    let mut x: i32 = (*p0).x;
    let mut y: i32 = (*p0).y;
    let dom: *mut i32;
    let nondom: *mut i32;

    if abs(n) > abs(d) {
        let swap: i32 = n;

        n = d;
        d = swap;

        dom = &mut x as *mut i32;
        nondom = &mut y as *mut i32;
    } else {
        dom = &mut y as *mut i32;
        nondom = &mut x as *mut i32;
    }

    let nondom_step: i32;
    if n < 0 {
        n = -n;
        nondom_step = -1;
    } else {
        nondom_step = 1;
    }

    let dom_step: i32;
    if d < 0 {
        d = -d;
        dom_step = -1;
    } else {
        dom_step = 1;
    }

    let mut a: i32 = 0;
    let mut run_length: i32 = 0;
    let mut count: i32 = 0;

    x = (*p0).x;
    y = (*p0).y;
    for _ in 0..=d {
        let pixel: i32;

        if y < 0 || y >= q.h || x < 0 || x >= q.w {
            break;
        }

        pixel = q.pixels[(y * q.w + x) as usize] as i32;

        if pixel != 0 {
            if run_length >= 2 {
                count += 1;
            }
            run_length = 0;
        } else {
            run_length += 1;
        }

        a += n;
        *dom = *dom + dom_step;
        if a >= d {
            *nondom = *nondom + nondom_step;
            a -= d;
        }
    }
    count
}

/// Try the measure the timing pattern for a given QR code. This does
/// not require the global perspective to have been set up, but it
/// does require that the capstone corners have been set to their
/// canonical rotation.
///
/// For each capstone, we find a point in the middle of the ring band
/// which is nearest the centre of the code. Using these points, we do
/// a horizontal and a vertical timing scan.
unsafe fn measure_timing_pattern(q: &mut Quirc, index: i32) -> i32 {
    let mut qr: *mut Grid = &mut q.grids[index as usize];

    for i in 0..3 {
        const US: [f64; 3] = [6.5, 6.5, 0.5];
        const VS: [f64; 3] = [0.5, 6.5, 6.5];
        let cap: *mut Capstone = &mut q.capstones[(*qr).caps[i as usize] as usize];
        (*qr).tpep[i as usize] = perspective_map(&(*cap).c, US[i as usize], VS[i as usize]);
    }

    (*qr).hscan = timing_scan(q, &mut (*qr).tpep[1], &mut (*qr).tpep[2]);
    (*qr).vscan = timing_scan(q, &mut (*qr).tpep[1], &mut (*qr).tpep[0]);

    let scan = max((*qr).hscan, (*qr).vscan);

    // If neither scan worked, we can't go any further.
    if scan < 0 {
        return -1;
    }

    // Choose the nearest allowable grid size
    let size = scan * 2 + 13;
    let ver = (size - 15) / 4;
    (*qr).grid_size = ver * 4 + 17;

    return 0;
}

/// Read a cell from a grid using the currently set perspective
/// transform. Returns +/- 1 for black/white, 0 for cells which are
/// out of image bounds.
unsafe fn read_cell(q: &mut Quirc, index: i32, x: i32, y: i32) -> i32 {
    let qr: *mut Grid = &mut q.grids[index as usize];

    let p = perspective_map(&(*qr).c, x as f64 + 0.5f64, y as f64 + 0.5f64);
    if p.y < 0 || p.y >= q.h || p.x < 0 || p.x >= q.w {
        0
    } else if q.pixels[(p.y * q.w + p.x) as usize] != 0 {
        1
    } else {
        -1
    }
}

unsafe fn fitness_cell(q: &mut Quirc, index: i32, x: i32, y: i32) -> i32 {
    let qr: *mut Grid = &mut q.grids[index as usize];
    let mut score: i32 = 0;

    for v in 0..3 {
        for u in 0..3 {
            const OFFSETS: [f64; 3] = [0.3, 0.5, 0.7];

            let p = perspective_map(
                &(*qr).c,
                x as f64 + OFFSETS[u as usize],
                y as f64 + OFFSETS[v as usize],
            );

            if !(p.y < 0 || p.y >= q.h || p.x < 0 || p.x >= q.w) {
                if q.pixels[(p.y * q.w + p.x) as usize] != 0 {
                    score += 1;
                } else {
                    score -= 1;
                }
            }
        }
    }
    score
}

unsafe fn fitness_ring(q: &mut Quirc, index: i32, cx: i32, cy: i32, radius: i32) -> i32 {
    let mut score: i32 = 0;

    for i in 0..radius * 2 {
        score += fitness_cell(q, index, cx - radius + i, cy - radius);
        score += fitness_cell(q, index, cx - radius, cy + radius - i);
        score += fitness_cell(q, index, cx + radius, cy - radius + i);
        score += fitness_cell(q, index, cx + radius - i, cy + radius);
    }
    score
}

unsafe fn fitness_apat(q: &mut Quirc, index: i32, cx: i32, cy: i32) -> i32 {
    fitness_cell(q, index, cx, cy) - fitness_ring(q, index, cx, cy, 1)
        + fitness_ring(q, index, cx, cy, 2)
}

unsafe fn fitness_capstone(q: &mut Quirc, index: i32, mut x: i32, mut y: i32) -> i32 {
    x += 3;
    y += 3;

    fitness_cell(q, index, x, y) + fitness_ring(q, index, x, y, 1) - fitness_ring(q, index, x, y, 2)
        + fitness_ring(q, index, x, y, 3)
}

/// Compute a fitness score for the currently configured perspective
/// transform, using the features we expect to find by scanning the
/// grid.
unsafe fn fitness_all(q: &mut Quirc, index: i32) -> i32 {
    let qr: *const Grid = &mut q.grids[index as usize];
    let version: i32 = ((*qr).grid_size - 17) / 4;
    let mut score: i32 = 0;

    // Check the timing pattern
    for i in 0..(*qr).grid_size - 14 {
        let expect: i32 = if i & 1 != 0 { 1 } else { -1 };
        score += fitness_cell(q, index, i + 7, 6) * expect;
        score += fitness_cell(q, index, 6, i + 7) * expect;
    }

    // Check capstones
    score += fitness_capstone(q, index, 0, 0);
    score += fitness_capstone(q, index, (*qr).grid_size - 7, 0);
    score += fitness_capstone(q, index, 0, (*qr).grid_size - 7);

    if version < 0 || version > QUIRC_MAX_VERSION as i32 {
        score
    } else {
        let info: *const VersionInfo = &VERSION_DB[version as usize];

        // Check alignment patterns
        let mut ap_count: i32 = 0;
        while ap_count < QUIRC_MAX_ALIGNMENT as i32 && ((*info).apat[ap_count as usize] != 0) {
            ap_count += 1;
        }

        for i in 1..ap_count - 1 {
            score += fitness_apat(q, index, 6, (*info).apat[i as usize]);
            score += fitness_apat(q, index, (*info).apat[i as usize], 6);
        }

        for i in 1..ap_count {
            for j in 1..ap_count {
                score += fitness_apat(q, index, (*info).apat[i as usize], (*info).apat[j as usize]);
            }
        }
        score
    }
}

unsafe fn jiggle_perspective(q: &mut Quirc, index: i32) {
    let mut qr: *mut Grid = &mut q.grids[index as usize];
    let mut best: i32 = fitness_all(q, index);
    let mut adjustments: [f64; 8] = [0f64; 8];

    for i in 0..8 {
        adjustments[i as usize] = (*qr).c[i as usize] * 0.02;
    }

    for _pass in 0..5 {
        for i in 0..16 {
            let j: i32 = i >> 1;
            let old: f64 = (*qr).c[j as usize];
            let step: f64 = adjustments[j as usize];

            let new = if i & 1 != 0 { old + step } else { old - step };

            (*qr).c[j as usize] = new;
            let test = fitness_all(q, index);

            if test > best {
                best = test;
            } else {
                (*qr).c[j as usize] = old;
            }
        }

        for i in 0..8 {
            adjustments[i as usize] *= 0.5;
        }
    }
}

/// Once the capstones are in place and an alignment point has been
/// chosen, we call this function to set up a grid-reading perspective
/// transform.
unsafe fn setup_qr_perspective(q: &mut Quirc, index: i32) {
    let qr: *mut Grid = &mut q.grids[index as usize];

    // Set up the perspective map for reading the grid
    let rect: [Point; 4] = [
        q.capstones[(*qr).caps[1] as usize].corners[0],
        q.capstones[(*qr).caps[2] as usize].corners[0],
        (*qr).align,
        q.capstones[(*qr).caps[0] as usize].corners[0],
    ];
    (*qr).c = perspective_setup(
        &rect,
        ((*qr).grid_size - 7) as f64,
        ((*qr).grid_size - 7) as f64,
    );

    jiggle_perspective(q, index);
}

/// Rotate the capstone with so that corner 0 is the leftmost with respect
/// to the given reference line.
unsafe fn rotate_capstone(cap: *mut Capstone, h0: *const Point, hd: *const Point) {
    let mut copy: [Point; 4] = std::mem::uninitialized();
    let mut best: i32 = std::mem::uninitialized();
    let mut best_score: i32 = std::mem::uninitialized();

    for j in 0..4 {
        let p: *mut Point = &mut (*cap).corners[j as usize];
        let score: i32 = ((*p).x - (*h0).x) * -(*hd).y + ((*p).y - (*h0).y) * (*hd).x;

        if j == 0 || score < best_score {
            best = j;
            best_score = score;
        }
    }

    // Rotate the capstone
    for j in 0..4 {
        copy[j as usize] = (*cap).corners[((j + best) % 4) as usize];
    }
    (*cap).corners = copy;
    (*cap).c = perspective_setup(&(*cap).corners, 7.0, 7.0);
}

unsafe fn record_qr_grid(mut q: &mut Quirc, mut a: i32, b: i32, mut c: i32) {
    let qr_index: i32;
    let mut qr: *mut Grid;

    if q.num_grids >= MAX_GRIDS as i32 {
        return;
    }

    // Construct the hypotenuse line from A to C. B should be to
    // the left of this line.
    let mut h0 = q.capstones[a as usize].center;
    let mut hd = Point {
        x: q.capstones[c as usize].center.x - q.capstones[a as usize].center.x,
        y: q.capstones[c as usize].center.y - q.capstones[a as usize].center.y,
    };

    // Make sure A-B-C is clockwise
    if (q.capstones[b as usize].center.x - h0.x) * -hd.y
        + (q.capstones[b as usize].center.y - h0.y) * hd.x
        > 0
    {
        let swap: i32 = a;
        a = c;
        c = swap;
        hd.x = -hd.x;
        hd.y = -hd.y;
    }

    // Record the grid and its components
    qr_index = q.num_grids;
    qr = &mut q.grids[q.num_grids as usize];
    q.num_grids += 1;

    memset(
        qr as (*mut ::std::os::raw::c_void),
        0,
        ::std::mem::size_of::<Grid>(),
    );
    (*qr).caps[0] = a;
    (*qr).caps[1] = b;
    (*qr).caps[2] = c;
    (*qr).align_region = -1;

    // Rotate each capstone so that corner 0 is top-left with respect
    // to the grid.
    for i in 0..3 {
        let mut cap: *mut Capstone = &mut q.capstones[(*qr).caps[i as usize] as usize];
        rotate_capstone(cap, &mut h0, &mut hd);
        (*cap).qr_grid = qr_index;
    }

    // Check the timing pattern. This doesn't require a perspective
    // transform.
    if !(measure_timing_pattern(q, qr_index) < 0) {
        // Make an estimate based for the alignment pattern based on extending
        // lines from capstones A and C.
        if !(line_intersect(
            &q.capstones[a as usize].corners[0],
            &q.capstones[a as usize].corners[1],
            &q.capstones[c as usize].corners[0],
            &q.capstones[c as usize].corners[3],
            &mut (*qr).align,
        ) == 0)
        {
            // On V2+ grids, we should use the alignment pattern.
            if (*qr).grid_size > 21 {
                // Try to find the actual location of the alignment pattern.
                find_alignment_pattern(q, qr_index);

                // Find the point of the alignment pattern closest to the
                // top-left of the QR grid.
                if (*qr).align_region >= 0 {
                    let reg: *mut Region = &mut q.regions[(*qr).align_region as usize];

                    // Start from some point inside the alignment pattern
                    (*qr).align = (*reg).seed;

                    let mut psd = PolygonScoreData {
                        r#ref: hd,
                        scores: [-hd.y * (*qr).align.x + hd.x * (*qr).align.y, 0, 0, 0],
                        corners: &mut (*qr).align,
                    };

                    flood_fill_seed(
                        q,
                        (*reg).seed.x,
                        (*reg).seed.y,
                        (*qr).align_region,
                        PIXEL_BLACK,
                        None,
                        0 as (*mut ::std::os::raw::c_void),
                        0,
                    );
                    flood_fill_seed(
                        q,
                        (*reg).seed.x,
                        (*reg).seed.y,
                        PIXEL_BLACK,
                        (*qr).align_region,
                        Some(find_leftmost_to_line),
                        &mut psd,
                        0,
                    );
                }
            }

            setup_qr_perspective(q, qr_index);
            return;
        }
    }

    // We've been unable to complete setup for this grid. Undo what we've
    // recorded and pretend it never happened.
    for i in 0..3 {
        q.capstones[(*qr).caps[i as usize] as usize].qr_grid = -1;
    }
    q.num_grids -= 1;
}

#[derive(Copy)]
#[repr(C)]
struct Neighbour {
    index: i32,
    distance: f64,
}

impl Clone for Neighbour {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
struct NeighbourList {
    n: [Neighbour; MAX_CAPSTONES],
    count: i32,
}

impl Clone for NeighbourList {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe fn test_neighbours(
    q: &mut Quirc,
    i: i32,
    hlist: *mut NeighbourList,
    vlist: *mut NeighbourList,
) {
    let mut best_score: f64 = 0.0;
    let mut best_h: i32 = -1;
    let mut best_v: i32 = -1;

    // Test each possible grouping
    for j in 0..(*hlist).count {
        for k in 0..(*vlist).count {
            let hn: &Neighbour = &(*hlist).n[j as usize];
            let vn: &Neighbour = &(*vlist).n[k as usize];
            let score: f64 = (1.0 - hn.distance / vn.distance).abs();

            if !(score > 2.5) {
                if best_h < 0 || score < best_score {
                    best_h = hn.index;
                    best_v = vn.index;
                    best_score = score;
                }
            }
        }
    }

    if best_h < 0 || best_v < 0 {
        return;
    }

    record_qr_grid(q, best_h, i, best_v);
}

unsafe fn test_grouping(q: &mut Quirc, i: usize) {
    let c1: *mut Capstone = &mut q.capstones[i as usize];

    if (*c1).qr_grid >= 0 {
        return;
    }

    let mut hlist: NeighbourList = std::mem::uninitialized();
    let mut vlist: NeighbourList = std::mem::uninitialized();
    hlist.count = 0;
    vlist.count = 0;

    // Look for potential neighbours by examining the relative gradients
    // from this capstone to others.
    for j in 0..q.capstones.len() {
        let c2: *mut Capstone = &mut q.capstones[j as usize];

        if i == j || (*c2).qr_grid >= 0 {
            continue;
        }

        let (mut u, mut v) = perspective_unmap(&(*c1).c, &mut (*c2).center);

        u = (u - 3.5).abs();
        v = (v - 3.5).abs();

        if u < 0.2 * v {
            let n: &mut Neighbour = &mut hlist.n[hlist.count as usize];
            hlist.count += 1;

            n.index = j as i32;
            n.distance = v;
        }

        if v < 0.2 * u {
            let n: &mut Neighbour = &mut vlist.n[vlist.count as usize];
            vlist.count += 1;

            n.index = j as i32;
            n.distance = u;
        }
    }

    if !(hlist.count != 0 && (vlist.count != 0)) {
        return;
    }

    test_neighbours(q, i as i32, &mut hlist, &mut vlist);
}

fn pixels_setup(q: &mut Quirc) {
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
pub unsafe fn quirc_begin(q: &mut Quirc, w: *mut i32, h: *mut i32) -> &mut [u8] {
    q.regions.resize(2, Default::default());
    q.num_grids = 0;

    if !w.is_null() {
        *w = q.w;
    }
    if !h.is_null() {
        *h = q.h;
    }

    q.image.as_mut()
}

pub unsafe fn quirc_end(q: &mut Quirc) {
    pixels_setup(q);
    threshold(q);

    for i in 0..q.h {
        finder_scan(q, i);
    }

    for i in 0..q.capstones.len() {
        test_grouping(q, i);
    }
}

/// Extract the QR-code specified by the given index.
pub unsafe fn quirc_extract(q: &mut Quirc, index: i32, mut code: *mut QuircCode) {
    let qr: *mut Grid = &mut q.grids[index as usize];

    if index < 0 || index > q.num_grids {
        return;
    }

    memset(
        code as (*mut ::std::os::raw::c_void),
        0,
        ::std::mem::size_of::<QuircCode>(),
    );

    (*code).corners[0] = perspective_map(&(*qr).c, 0.0, 0.0);
    (*code).corners[1] = perspective_map(&(*qr).c, (*qr).grid_size as f64, 0.0);
    (*code).corners[2] = perspective_map(&(*qr).c, (*qr).grid_size as f64, (*qr).grid_size as f64);
    (*code).corners[3] = perspective_map(&(*qr).c, 0.0, (*qr).grid_size as f64);

    (*code).size = (*qr).grid_size;

    let mut i: i32 = 0;
    for y in 0..(*qr).grid_size {
        for x in 0..(*qr).grid_size {
            if read_cell(q, index, x, y) > 0 {
                (*code).cell_bitmap[(i >> 3) as usize] |= 1 << (i & 7);
            }
            i = i + 1;
        }
    }
}
