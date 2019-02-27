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

#[no_mangle]
pub unsafe extern "C" fn quirc_begin(
    mut q: *mut quirc,
    mut w: *mut i32,
    mut h: *mut i32,
) -> *mut u8 {
    (*q).num_regions = 2i32;
    (*q).num_capstones = 0i32;
    (*q).num_grids = 0i32;
    if !w.is_null() {
        *w = (*q).w;
    }
    if !h.is_null() {
        *h = (*q).h;
    }
    (*q).image
}
//
//void quirc_end(struct quirc *q)
//{
//	int i;
//
//	pixels_setup(q);
//	threshold(q);
//
//	for (i = 0; i < q->h; i++)
//		finder_scan(q, i);
//
//	for (i = 0; i < q->num_capstones; i++)
//		test_grouping(q, i);
//}
//
//void quirc_extract(const struct quirc *q, int index,
//		   struct quirc_code *code)
//{
//	const struct quirc_grid *qr = &q->grids[index];
//	int y;
//	int i = 0;
//
//	if (index < 0 || index > q->num_grids)
//		return;
//
//	memset(code, 0, sizeof(*code));
//
//	perspective_map(qr->c, 0.0, 0.0, &code->corners[0]);
//	perspective_map(qr->c, qr->grid_size, 0.0, &code->corners[1]);
//	perspective_map(qr->c, qr->grid_size, qr->grid_size,
//			&code->corners[2]);
//	perspective_map(qr->c, 0.0, qr->grid_size, &code->corners[3]);
//
//	code->size = qr->grid_size;
//
//	for (y = 0; y < qr->grid_size; y++) {
//		int x;
//
//		for (x = 0; x < qr->grid_size; x++) {
//			if (read_cell(q, index, x, y) > 0)
//				code->cell_bitmap[i >> 3] |= (1 << (i & 7));
//
//			i++;
//		}
//	}
//}
