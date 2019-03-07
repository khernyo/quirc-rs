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

extern crate quirc_rs;
extern crate sdl2;
extern crate sdl2_unifont;

use std::ffi::CStr;
use std::path::Path;
use std::os::raw::c_double;

use libc::{c_char, c_void, fprintf, memset, perror, printf, puts, snprintf, timespec, FILE};
use libc_extra::unix::stdio::stderr;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use quirc_rs::quirc::*;
use quirc_rs::decode::*;
use quirc_rs::identify::*;

extern "C" {
    fn rint(x: c_double) -> c_double;
}

include!("../tests/util/dbgutil.rs");

fn main() {
    use ::std::os::unix::ffi::OsStringExt;
    let mut argv_storage
        = ::std::env::args_os().map(
              |str| {
                        let mut vec = str.into_vec();
                        vec.push(b'\0');
                        vec
                    }
          ).collect::<Vec<_>>(
          );
    let mut argv
        = argv_storage.iter_mut().map(|vec| vec.as_mut_ptr()).chain(
              Some(::std::ptr::null_mut())
          ).collect::<Vec<_>>(
          );
    let ret
        = unsafe {
              _c_main(argv_storage.len() as (i32),argv.as_mut_ptr())
          };
    ::std::process::exit(ret);
}

unsafe extern fn dump_info(mut q : *mut quirc) {
    let mut count : i32 = quirc_count(q as (*const quirc));
    let mut i : i32;
    printf((*b"%d QR-codes found:\n\n\0").as_ptr() as *const c_char,count);
    i = 0i32;
    'loop1: loop {
        if !(i < count) {
            break;
        }
        let mut code : quirc_code = std::mem::uninitialized();
        let mut data : quirc_data = std::mem::uninitialized();
        let mut err : Enum1;
        quirc_extract(
            q as (*mut quirc),
            i,
            &mut code as (*mut quirc_code)
        );
        err = quirc_decode(
                  &mut code as (*mut quirc_code) as (*const quirc_code),
                  &mut data as (*mut quirc_data)
              );
        dump_cells(&mut code as (*mut quirc_code) as (*const quirc_code));
        printf((*b"\n\0").as_ptr() as *const c_char);
        if err != Enum1::QUIRC_SUCCESS {
            printf(
                (*b"  Decoding FAILED: %s\n\0").as_ptr() as *const c_char,
                quirc_strerror(err)
            );
        } else {
            printf((*b"  Decoding successful:\n\0").as_ptr() as *const c_char);
            dump_data(&mut data as (*mut quirc_data));
        }
        printf((*b"\n\0").as_ptr() as *const c_char);
        i = i + 1;
    }
}

unsafe fn pixelColor(mut canvas : &mut Canvas<Window>, x: i16, y: i16, color: Color) {
    canvas.set_draw_color(color);
    canvas.draw_point((x as i32, y as i32)).unwrap();
}

unsafe fn lineColor(
    mut canvas : &mut Canvas<Window>,
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    color: Color,
) {
    canvas.set_draw_color(color);
    canvas.draw_line((x1 as i32, y1 as i32), (x2 as i32, y2 as i32));
}

unsafe fn stringColor(
    mut canvas : &mut Canvas<Window>,
    x: i16,
    y: i16,
    s: *const u8,
    color: Color,
) {
    let renderer = sdl2_unifont::renderer::SurfaceRenderer::new(color, Color::RGBA(0, 0, 0, 0));
    let surface = renderer.draw(CStr::from_ptr(s as *const c_char).to_str().unwrap()).unwrap();
    let (w, h) = canvas.output_size().unwrap();
    let mut screen = sdl2::surface::Surface::new(
        w,
        h,
        sdl2::pixels::PixelFormatEnum::RGBA8888,
    ).unwrap();
    surface.blit(None, &mut screen, Rect::new(x as i32, y as i32, 0, 0));
    let texture_creator = canvas.texture_creator();
    let tex = texture_creator
        .create_texture_from_surface(screen)
        .unwrap();
    canvas.copy(&tex, None, None).unwrap();
}

const QUIRC_PIXEL_WHITE: i32 = 0;
const QUIRC_PIXEL_BLACK: i32 = 1;

unsafe extern fn draw_frame(
    mut canvas : &mut Canvas<Window>, mut q : *mut quirc
) {
    let mut raw : *mut u8 = (*q).image;
    let mut x : i32;
    let mut y : i32;

    for y in 0..(*q).h {
        for x in 0..(*q).w {
            let mut v
                : u8
                = *{
                       let _old = raw;
                       raw = raw.offset(1isize);
                       _old
                   };
            let mut reg
                : *mut quirc_region
                = &mut (*q).regions[v as (usize)] as (*mut quirc_region);
            let color = match v as (i32) {
                QUIRC_PIXEL_BLACK => Color::RGB(0, 0, 0),
                QUIRC_PIXEL_WHITE => Color::RGB(0xff, 0xff, 0xff),
                _ => {
                    if (*reg).capstone >= 0i32 {
                        Color::RGB(0, 0x80, 0)
                    } else {
                        Color::RGB(0x80, 0x80, 0x80)
                    }
                }
            };
            pixelColor(canvas, x as i16, y as i16, color);
        }
    }
}

unsafe extern fn draw_blob(
    mut canvas : &mut Canvas<Window>, mut x : i32, mut y : i32
) {
    for i in -2..=2 {
        for j in -2..=2 {
            pixelColor(canvas, (x + i) as (i16), (y + j) as (i16), Color::RGBA(0, 0, 0xff, 0xff));
        }
    }
}

unsafe extern fn draw_capstone(
    mut canvas : &mut Canvas<Window>, mut q : *mut quirc, mut index : i32
) {
    let mut cap
        : *mut quirc_capstone
        = &mut (*q).capstones[index as (usize)] as (*mut quirc_capstone);
    let mut j : i32;
    let mut buf : [u8; 8] = std::mem::uninitialized();
    for j in 0..4 {
        let mut p0
            : *mut quirc_point
            = &mut (*cap).corners[j as (usize)] as (*mut quirc_point);
        let mut p1
            : *mut quirc_point
            = &mut (*cap).corners[
                       ((j + 1i32) % 4i32) as (usize)
                   ] as (*mut quirc_point);
        lineColor(
            canvas,
            (*p0).x as (i16),
            (*p0).y as (i16),
            (*p1).x as (i16),
            (*p1).y as (i16),
            Color::RGBA(0x80, 0, 0x80, 0xff)
        );
    }
    draw_blob(
        canvas,
        (*cap).corners[0usize].x,
        (*cap).corners[0usize].y
    );
    if (*cap).qr_grid < 0i32 {
        snprintf(
            buf.as_mut_ptr() as *mut c_char,
            ::std::mem::size_of::<[u8; 8]>(),
            (*b"?%d\0").as_ptr() as *const c_char,
            index
        );
        stringColor(
            canvas,
            (*cap).center.x as (i16),
            (*cap).center.y as (i16),
            buf.as_mut_ptr() as (*const u8),
            Color::RGB(0, 0, 0)
        );
    }
}

unsafe extern fn perspective_map(
    mut c : *const f64,
    mut u : f64,
    mut v : f64,
    mut ret : *mut quirc_point
) {
    let mut den
        : f64
        = *c.offset(6isize) * u + *c.offset(7isize) * v + 1.0f64;
    let mut x
        : f64
        = (*c.offset(0isize) * u + *c.offset(1isize) * v + *c.offset(
                                                                2isize
                                                            )) / den;
    let mut y
        : f64
        = (*c.offset(3isize) * u + *c.offset(4isize) * v + *c.offset(
                                                                5isize
                                                            )) / den;

    (*ret).x = rint(x) as i32;
    (*ret).y = rint(y) as i32;
}

unsafe extern fn draw_mark(
    mut canvas : &mut Canvas<Window>, mut x : i32, mut y : i32
) {
    let red = Color::RGBA(0xff, 0, 0, 0xff);
    pixelColor(canvas,x as (i16),y as (i16), red);
    pixelColor(canvas,(x + 1i32) as (i16),y as (i16), red);
    pixelColor(canvas,(x - 1i32) as (i16),y as (i16), red);
    pixelColor(canvas,x as (i16),(y + 1i32) as (i16), red);
    pixelColor(canvas,x as (i16),(y - 1i32) as (i16), red);
}

unsafe extern fn draw_grid(
    mut canvas : &mut Canvas<Window>, mut q : *mut quirc, mut index : i32
) {
    let mut qr
        : *mut quirc_grid
        = &mut (*q).grids[index as (usize)] as (*mut quirc_grid);
    let mut x : i32;
    let mut y : i32;
    let mut i : i32;
    for i in 0..3 {
        let mut cap
            : *mut quirc_capstone
            = &mut (*q).capstones[
                       (*qr).caps[i as (usize)] as (usize)
                   ] as (*mut quirc_capstone);
        let mut buf : [u8; 8] = std::mem::uninitialized();
        snprintf(
            buf.as_mut_ptr() as *mut c_char,
            ::std::mem::size_of::<[u8; 8]>(),
            (*b"%d.%c\0").as_ptr() as *const c_char,
            index,
            (*b"ABC\0")[i as (usize)] as (i32)
        );
        stringColor(
            canvas,
            (*cap).center.x as (i16),
            (*cap).center.y as (i16),
            buf.as_mut_ptr() as (*const u8),
            Color::RGB(0, 0, 0)
        );
    }
    lineColor(
        canvas,
        (*qr).tpep[0usize].x as (i16),
        (*qr).tpep[0usize].y as (i16),
        (*qr).tpep[1usize].x as (i16),
        (*qr).tpep[1usize].y as (i16),
        Color::RGBA(0xff, 0, 0xff, 0xff)
    );
    lineColor(
        canvas,
        (*qr).tpep[1usize].x as (i16),
        (*qr).tpep[1usize].y as (i16),
        (*qr).tpep[2usize].x as (i16),
        (*qr).tpep[2usize].y as (i16),
        Color::RGBA(0xff, 0, 0xff, 0xff)
    );
    if (*qr).align_region >= 0i32 {
        draw_blob(canvas,(*qr).align.x,(*qr).align.y);
    }
    for y in 0..(*qr).grid_size {
        for x in 0..(*qr).grid_size {
            let mut u : f64 = x as (f64) + 0.5f64;
            let mut v : f64 = y as (f64) + 0.5f64;
            let mut p : quirc_point = std::mem::uninitialized();
            perspective_map(
                (*qr).c.as_mut_ptr() as (*const f64),
                u,
                v,
                &mut p as (*mut quirc_point)
            );
            draw_mark(canvas,p.x,p.y);
        }
    }
}

unsafe extern fn sdl_examine(mut q : *mut quirc) -> i32 {
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", (*q).w as u32, (*q).h as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    'mainloop: loop {
        let mut event_pump = sdl_context.event_pump().unwrap();
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => break 'mainloop,
                _ => (),
            }

            draw_frame(&mut canvas, q);
            for i in 0..(*q).num_capstones {
                draw_capstone(&mut canvas, q, i);
            }
            for i in 0..(*q).num_grids {
                draw_grid(&mut canvas, q, i);
            }

            canvas.present();
        }
    }
    0
}

pub unsafe extern fn _c_main(
    mut argc : i32, mut argv : *mut *mut u8
) -> i32 {
    let mut q : *mut quirc;
    printf((*b"quirc inspection program\n\0").as_ptr() as *const c_char);
    printf(
        (*b"Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>\n\0").as_ptr(
        ) as *const c_char
    );
    printf((*b"Library version: %s\n\0").as_ptr() as *const c_char,quirc_version());
    printf((*b"\n\0").as_ptr() as *const c_char);
    if argc < 2i32 {
        fprintf(
            stderr as *mut FILE,
            (*b"Usage: %s <testfile.jpg|testfile.png>\n\0").as_ptr() as *const c_char,
            *argv.offset(0isize)
        );
        -1i32
    } else {
        q = quirc_new() as (*mut quirc);
        (if q.is_null() {
             perror((*b"can\'t create quirc object\0").as_ptr() as *const c_char);
             -1i32
         } else {
             let mut status : i32 = -1i32;
             status = load_image(q, &Path::new(CStr::from_ptr(*argv.offset(1isize) as (*const c_char)).to_str().unwrap()));
             (if status < 0i32 {
                  quirc_destroy(q as (*mut quirc));
                  -1i32
              } else {
                  quirc_end(q as (*mut quirc));
                  dump_info(q);
                  (if sdl_examine(q) < 0i32 {
                       quirc_destroy(q as (*mut quirc));
                       -1i32
                   } else {
                       quirc_destroy(q as (*mut quirc));
                       0i32
                   })
              })
         })
    }
}
