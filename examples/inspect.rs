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

use std::os::raw::c_double;
use std::path::Path;

use clap::{App, Arg};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::consts::*;
use quirc_rs::quirc::*;

use test_utils::dbgutil::*;

extern "C" {
    fn rint(x: c_double) -> c_double;
}

unsafe fn dump_info(q: &mut Quirc) {
    let count: i32 = quirc_count(q);
    println!("{} QR-codes found:\n", count);
    for i in 0..count {
        let code = quirc_extract(q, i).unwrap();
        let result = quirc_decode(&code);
        dump_cells(&code);
        println!();
        match result {
            Ok(data) => {
                println!("  Decoding successful:");
                dump_data(&data);
            }
            Err(e) => println!("  Decoding FAILED: {}", quirc_strerror(e)),
        }
        println!();
    }
}

fn pixel_color(canvas: &mut Canvas<Window>, x: i16, y: i16, color: Color) {
    canvas.set_draw_color(color);
    canvas.draw_point((i32::from(x), i32::from(y))).unwrap();
}

fn line_color(canvas: &mut Canvas<Window>, x1: i16, y1: i16, x2: i16, y2: i16, color: Color) {
    canvas.set_draw_color(color);
    canvas
        .draw_line(
            (i32::from(x1), i32::from(y1)),
            (i32::from(x2), i32::from(y2)),
        )
        .unwrap();
}

fn string_color(canvas: &mut Canvas<Window>, x: i16, y: i16, s: &str, color: Color) {
    let renderer = sdl2_unifont::renderer::SurfaceRenderer::new(color, Color::RGBA(0, 0, 0, 0));
    let surface = renderer.draw(s).unwrap();
    let (width, height) = canvas.output_size().unwrap();
    let mut screen =
        sdl2::surface::Surface::new(width, height, sdl2::pixels::PixelFormatEnum::RGBA8888)
            .unwrap();
    surface
        .blit(
            None,
            &mut screen,
            Rect::new(i32::from(x), i32::from(y), 0, 0),
        )
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let tex = texture_creator.create_texture_from_surface(screen).unwrap();
    canvas.copy(&tex, None, None).unwrap();
}

unsafe fn draw_frame(canvas: &mut Canvas<Window>, q: &mut Quirc) {
    let mut raw = q.image.pixels().as_ptr();

    for y in 0..q.image.height() {
        for x in 0..q.image.width() {
            let v: u8 = *{
                let _old = raw;
                raw = raw.offset(1isize);
                _old
            };
            let reg: &mut Region = &mut q.regions[v as (usize)];
            let color = match i32::from(v) {
                PIXEL_BLACK => Color::RGB(0, 0, 0),
                PIXEL_WHITE => Color::RGB(0xff, 0xff, 0xff),
                _ => {
                    if reg.capstone >= 0i32 {
                        Color::RGB(0, 0x80, 0)
                    } else {
                        Color::RGB(0x80, 0x80, 0x80)
                    }
                }
            };
            pixel_color(canvas, x as i16, y as i16, color);
        }
    }
}

fn draw_blob(canvas: &mut Canvas<Window>, x: i32, y: i32) {
    for i in -2..=2 {
        for j in -2..=2 {
            pixel_color(
                canvas,
                (x + i) as (i16),
                (y + j) as (i16),
                Color::RGBA(0, 0, 0xff, 0xff),
            );
        }
    }
}

fn draw_mark(canvas: &mut Canvas<Window>, x: i32, y: i32) {
    let red = Color::RGBA(0xff, 0, 0, 0xff);
    pixel_color(canvas, x as (i16), y as (i16), red);
    pixel_color(canvas, (x + 1i32) as (i16), y as (i16), red);
    pixel_color(canvas, (x - 1i32) as (i16), y as (i16), red);
    pixel_color(canvas, x as (i16), (y + 1i32) as (i16), red);
    pixel_color(canvas, x as (i16), (y - 1i32) as (i16), red);
}

fn draw_capstone(canvas: &mut Canvas<Window>, q: &mut Quirc, index: usize) {
    let cap: &mut Capstone = &mut q.capstones[index];
    for j in 0..4 {
        let p0: &mut Point = &mut cap.corners[j];
        let p0x = p0.x as (i16);
        let p0y = p0.y as (i16);
        let p1: &mut Point = &mut cap.corners[(j + 1) % 4];
        let p1x = p1.x as (i16);
        let p1y = p1.y as (i16);
        line_color(canvas, p0x, p0y, p1x, p1y, Color::RGBA(0x80, 0, 0x80, 0xff));
    }
    draw_blob(canvas, cap.corners[0usize].x, cap.corners[0usize].y);
    if cap.qr_grid < 0i32 {
        let s = format!("?{}", index);
        string_color(
            canvas,
            cap.center.x as (i16),
            cap.center.y as (i16),
            &s,
            Color::RGB(0, 0, 0),
        );
    }
}

#[allow(clippy::many_single_char_names)]
unsafe fn perspective_map(c: *const f64, u: f64, v: f64, mut ret: *mut Point) {
    let den: f64 = *c.offset(6isize) * u + *c.offset(7isize) * v + 1.0f64;
    let x: f64 = (*c.offset(0isize) * u + *c.offset(1isize) * v + *c.offset(2isize)) / den;
    let y: f64 = (*c.offset(3isize) * u + *c.offset(4isize) * v + *c.offset(5isize)) / den;

    (*ret).x = rint(x) as i32;
    (*ret).y = rint(y) as i32;
}

unsafe fn draw_grid(canvas: &mut Canvas<Window>, q: &mut Quirc, index: usize) {
    let qr: &mut Grid = &mut q.grids[index];
    for i in 0..3 {
        let cap: &mut Capstone = &mut q.capstones[qr.caps[i] as (usize)];
        let s = format!("{}.{}", index, "ABC".chars().nth(i).unwrap());
        string_color(
            canvas,
            cap.center.x as (i16),
            cap.center.y as (i16),
            &s,
            Color::RGB(0, 0, 0),
        );
    }
    line_color(
        canvas,
        qr.tpep[0usize].x as (i16),
        qr.tpep[0usize].y as (i16),
        qr.tpep[1usize].x as (i16),
        qr.tpep[1usize].y as (i16),
        Color::RGBA(0xff, 0, 0xff, 0xff),
    );
    line_color(
        canvas,
        qr.tpep[1usize].x as (i16),
        qr.tpep[1usize].y as (i16),
        qr.tpep[2usize].x as (i16),
        qr.tpep[2usize].y as (i16),
        Color::RGBA(0xff, 0, 0xff, 0xff),
    );
    if qr.align_region >= 0i32 {
        draw_blob(canvas, qr.align.x, qr.align.y);
    }
    for y in 0..qr.grid_size {
        for x in 0..qr.grid_size {
            let u: f64 = f64::from(x) + 0.5f64;
            let v: f64 = f64::from(y) + 0.5f64;
            let mut p: Point = std::mem::uninitialized();
            perspective_map(qr.c.as_mut_ptr() as (*const f64), u, v, &mut p);
            draw_mark(canvas, p.x, p.y);
        }
    }
}

unsafe fn sdl_examine(q: &mut Quirc) -> i32 {
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", q.image.width() as u32, q.image.height() as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    'mainloop: loop {
        let mut event_pump = sdl_context.event_pump().unwrap();
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'mainloop,
                _ => (),
            }

            draw_frame(&mut canvas, q);
            for i in 0..q.capstones.len() {
                draw_capstone(&mut canvas, q, i);
            }
            for i in 0..q.grids.len() {
                draw_grid(&mut canvas, q, i);
            }

            canvas.present();
        }
    }
    0
}

fn main() {
    unsafe { _c_main() }
}

pub unsafe fn _c_main() {
    let paths_arg_name = "paths";
    let paths_arg = Arg::with_name(paths_arg_name).required(true);

    let args = App::new("inspect")
        .about("quirc inspection program")
        .version(quirc_version())
        .author("Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>")
        .args(&[paths_arg]);

    let matches = args.get_matches();
    let path = matches.value_of(paths_arg_name).unwrap();

    let (width, height, mut image_bytes) = load_image(&Path::new(path));

    let mut decoder = Quirc::new(Image::new(width, height, &mut image_bytes));
    quirc_identify(&mut decoder);
    dump_info(&mut decoder);
    if sdl_examine(&mut decoder) < 0i32 {
        panic!();
    }
}
