extern crate quirc_rs;

use std::path::Path;

use libc::{c_char, memset, perror, puts, timespec};

use quirc_rs::quirc::*;
use quirc_rs::decode::*;
use quirc_rs::identify::*;

include!("util/dbgutil.rs");

extern {
    fn SDL_Flip(screen : *mut SDL_Surface) -> i32;
    fn SDL_GetError() -> *mut u8;
    fn SDL_Init(flags : u32) -> i32;
    fn SDL_LockSurface(surface : *mut SDL_Surface) -> i32;
    fn SDL_Quit();
    fn SDL_SetVideoMode(
        width : i32, height : i32, bpp : i32, flags : u32
    ) -> *mut SDL_Surface;
    fn SDL_UnlockSurface(surface : *mut SDL_Surface);
    fn SDL_WaitEvent(event : *mut SDL_Event) -> i32;
    fn check_if_png(filename : *const u8) -> i32;
    fn fprintf(
        __stream : *mut _IO_FILE, __format : *const u8, ...
    ) -> i32;
    fn lineColor(
        dst : *mut SDL_Surface,
        x1 : i16,
        y1 : i16,
        x2 : i16,
        y2 : i16,
        color : u32
    ) -> i32;
    fn load_jpeg(q : *mut quirc, filename : *const u8) -> i32;
    fn load_png(q : *mut quirc, filename : *const u8) -> i32;
    fn pixelColor(
        dst : *mut SDL_Surface, x : i16, y : i16, color : u32
    ) -> i32;
    fn printf(__format : *const u8, ...) -> i32;
    fn quirc_count(q : *const quirc) -> i32;
    fn quirc_decode(
        code : *const quirc_code, data : *mut quirc_data
    ) -> Enum1;
    fn quirc_destroy(q : *mut quirc);
    fn quirc_end(q : *mut quirc);
    fn quirc_extract(
        q : *const quirc, index : i32, code : *mut quirc_code
    );
    fn quirc_new() -> *mut quirc;
    fn quirc_strerror(err : Enum1) -> *const u8;
    fn quirc_version() -> *const u8;
    fn snprintf(
        __s : *mut u8, __maxlen : usize, __format : *const u8, ...
    ) -> i32;
    static mut stderr : *mut _IO_FILE;
    fn stringColor(
        dst : *mut SDL_Surface,
        x : i16,
        y : i16,
        s : *const u8,
        color : u32
    ) -> i32;
}

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
    printf((*b"%d QR-codes found:\n\n\0").as_ptr(),count);
    i = 0i32;
    'loop1: loop {
        if !(i < count) {
            break;
        }
        let mut code : quirc_code;
        let mut data : quirc_data;
        let mut err : Enum1;
        quirc_extract(
            q as (*const quirc),
            i,
            &mut code as (*mut quirc_code)
        );
        err = quirc_decode(
                  &mut code as (*mut quirc_code) as (*const quirc_code),
                  &mut data as (*mut quirc_data)
              );
        dump_cells(&mut code as (*mut quirc_code) as (*const quirc_code));
        printf((*b"\n\0").as_ptr());
        if err != Enum1::QUIRC_SUCCESS {
            printf(
                (*b"  Decoding FAILED: %s\n\0").as_ptr(),
                quirc_strerror(err)
            );
        } else {
            printf((*b"  Decoding successful:\n\0").as_ptr());
            dump_data(&mut data as (*mut quirc_data) as (*const quirc_data));
        }
        printf((*b"\n\0").as_ptr());
        i = i + 1;
    }
}

unsafe extern fn draw_frame(
    mut screen : *mut SDL_Surface, mut q : *mut quirc
) {
    let mut pix : *mut u8;
    let mut raw : *mut u8 = (*q).image;
    let mut x : i32;
    let mut y : i32;
    SDL_LockSurface(screen);
    pix = (*screen).pixels as (*mut u8);
    y = 0i32;
    'loop1: loop {
        if !(y < (*q).h) {
            break;
        }
        let mut row : *mut u32 = pix as (*mut u32);
        x = 0i32;
        'loop4: loop {
            if !(x < (*q).w) {
                break;
            }
            let mut v
                : u8
                = *{
                       let _old = raw;
                       raw = raw.offset(1isize);
                       _old
                   };
            let mut color
                : u32
                = (v as (i32) << 16i32 | v as (i32) << 8i32 | v as (i32)) as (u32);
            let mut reg
                : *mut quirc_region
                = &mut (*q).regions[v as (usize)] as (*mut quirc_region);
            if v as (i32) == 1i32 {
                color = 0x0u32;
            } else if v as (i32) == 0i32 {
                color = 0xffffffu32;
            } else if (*reg).capstone >= 0i32 {
                color = 0x8000u32;
            } else {
                color = 0x808080u32;
            }
            *{
                 let _old = row;
                 row = row.offset(1isize);
                 _old
             } = color;
            x = x + 1;
        }
        pix = pix.offset((*screen).pitch as (isize));
        y = y + 1;
    }
    SDL_UnlockSurface(screen);
}

unsafe extern fn draw_blob(
    mut screen : *mut SDL_Surface, mut x : i32, mut y : i32
) {
    let mut i : i32;
    let mut j : i32;
    i = -2i32;
    'loop1: loop {
        if !(i <= 2i32) {
            break;
        }
        j = -2i32;
        'loop4: loop {
            if !(j <= 2i32) {
                break;
            }
            pixelColor(screen,(x + i) as (i16),(y + j) as (i16),0xffffu32);
            j = j + 1;
        }
        i = i + 1;
    }
}

unsafe extern fn draw_capstone(
    mut screen : *mut SDL_Surface, mut q : *mut quirc, mut index : i32
) {
    let mut cap
        : *mut quirc_capstone
        = &mut (*q).capstones[index as (usize)] as (*mut quirc_capstone);
    let mut j : i32;
    let mut buf : [u8; 8];
    j = 0i32;
    'loop1: loop {
        if !(j < 4i32) {
            break;
        }
        let mut p0
            : *mut quirc_point
            = &mut (*cap).corners[j as (usize)] as (*mut quirc_point);
        let mut p1
            : *mut quirc_point
            = &mut (*cap).corners[
                       ((j + 1i32) % 4i32) as (usize)
                   ] as (*mut quirc_point);
        lineColor(
            screen,
            (*p0).x as (i16),
            (*p0).y as (i16),
            (*p1).x as (i16),
            (*p1).y as (i16),
            0x800080ffu32
        );
        j = j + 1;
    }
    draw_blob(
        screen,
        (*cap).corners[0usize].x,
        (*cap).corners[0usize].y
    );
    if (*cap).qr_grid < 0i32 {
        snprintf(
            buf.as_mut_ptr(),
            ::std::mem::size_of::<[u8; 8]>(),
            (*b"?%d\0").as_ptr(),
            index
        );
        stringColor(
            screen,
            (*cap).center.x as (i16),
            (*cap).center.y as (i16),
            buf.as_mut_ptr() as (*const u8),
            0xffu32
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

    (*ret).x = rint(x);
    (*ret).y = rint(y);
}

unsafe extern fn draw_mark(
    mut screen : *mut SDL_Surface, mut x : i32, mut y : i32
) {
    pixelColor(screen,x as (i16),y as (i16),0xff0000ffu32);
    pixelColor(screen,(x + 1i32) as (i16),y as (i16),0xff0000ffu32);
    pixelColor(screen,(x - 1i32) as (i16),y as (i16),0xff0000ffu32);
    pixelColor(screen,x as (i16),(y + 1i32) as (i16),0xff0000ffu32);
    pixelColor(screen,x as (i16),(y - 1i32) as (i16),0xff0000ffu32);
}

unsafe extern fn draw_grid(
    mut screen : *mut SDL_Surface, mut q : *mut quirc, mut index : i32
) {
    let mut qr
        : *mut quirc_grid
        = &mut (*q).grids[index as (usize)] as (*mut quirc_grid);
    let mut x : i32;
    let mut y : i32;
    let mut i : i32;
    i = 0i32;
    'loop1: loop {
        if !(i < 3i32) {
            break;
        }
        let mut cap
            : *mut quirc_capstone
            = &mut (*q).capstones[
                       (*qr).caps[i as (usize)] as (usize)
                   ] as (*mut quirc_capstone);
        let mut buf : [u8; 8];
        snprintf(
            buf.as_mut_ptr(),
            ::std::mem::size_of::<[u8; 8]>(),
            (*b"%d.%c\0").as_ptr(),
            index,
            (*b"ABC\0")[i as (usize)] as (i32)
        );
        stringColor(
            screen,
            (*cap).center.x as (i16),
            (*cap).center.y as (i16),
            buf.as_mut_ptr() as (*const u8),
            0xffu32
        );
        i = i + 1;
    }
    lineColor(
        screen,
        (*qr).tpep[0usize].x as (i16),
        (*qr).tpep[0usize].y as (i16),
        (*qr).tpep[1usize].x as (i16),
        (*qr).tpep[1usize].y as (i16),
        0xff00ffffu32
    );
    lineColor(
        screen,
        (*qr).tpep[1usize].x as (i16),
        (*qr).tpep[1usize].y as (i16),
        (*qr).tpep[2usize].x as (i16),
        (*qr).tpep[2usize].y as (i16),
        0xff00ffffu32
    );
    if (*qr).align_region >= 0i32 {
        draw_blob(screen,(*qr).align.x,(*qr).align.y);
    }
    y = 0i32;
    'loop5: loop {
        if !(y < (*qr).grid_size) {
            break;
        }
        x = 0i32;
        'loop8: loop {
            if !(x < (*qr).grid_size) {
                break;
            }
            let mut u : f64 = x as (f64) + 0.5f64;
            let mut v : f64 = y as (f64) + 0.5f64;
            let mut p : quirc_point = std::mem::uninitialized();
            perspective_map(
                (*qr).c.as_mut_ptr() as (*const f64),
                u,
                v,
                &mut p as (*mut quirc_point)
            );
            draw_mark(screen,p.x,p.y);
            x = x + 1;
        }
        y = y + 1;
    }
}

unsafe extern fn sdl_examine(mut q : *mut quirc) -> i32 {
    let mut screen : *mut SDL_Surface;
    let mut ev : SDL_Event;
    if SDL_Init(0x20u32) < 0i32 {
        fprintf(
            stderr,
            (*b"couldn\'t init SDL: %s\n\0").as_ptr(),
            SDL_GetError()
        );
        -1i32
    } else {
        screen = SDL_SetVideoMode((*q).w,(*q).h,32i32,0x0u32);
        (if screen.is_null() {
             fprintf(
                 stderr,
                 (*b"couldn\'t init video mode: %s\n\0").as_ptr(),
                 SDL_GetError()
             );
             -1i32
         } else {
             'loop2: loop {
                 if !(SDL_WaitEvent(&mut ev as (*mut SDL_Event)) >= 0i32) {
                     break;
                 }
                 let mut i : i32;

                 if ev.r#type == SDL_QUIT {
                     break;
                 }

                 if ev.r#type == SDL_KEYDOWN && ev.key.keysym.sym == 'q' {
                     break;
                 }

                 draw_frame(screen,q);
                 i = 0i32;
                 'loop5: loop {
                     if !(i < (*q).num_capstones) {
                         break;
                     }
                     draw_capstone(screen,q,i);
                     i = i + 1;
                 }
                 i = 0i32;
                 'loop7: loop {
                     if !(i < (*q).num_grids) {
                         break;
                     }
                     draw_grid(screen,q,i);
                     i = i + 1;
                 }
                 SDL_Flip(screen);
             }
             SDL_Quit();
             0i32
         })
    }
}

#[no_mangle]
pub unsafe extern fn _c_main(
    mut argc : i32, mut argv : *mut *mut u8
) -> i32 {
    let mut q : *mut quirc;
    printf((*b"quirc inspection program\n\0").as_ptr());
    printf(
        (*b"Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>\n\0").as_ptr(
        )
    );
    printf((*b"Library version: %s\n\0").as_ptr(),quirc_version());
    printf((*b"\n\0").as_ptr());
    if argc < 2i32 {
        fprintf(
            stderr,
            (*b"Usage: %s <testfile.jpg|testfile.png>\n\0").as_ptr(),
            *argv.offset(0isize)
        );
        -1i32
    } else {
        q = quirc_new() as (*mut quirc);
        (if q.is_null() {
             perror((*b"can\'t create quirc object\0").as_ptr());
             -1i32
         } else {
             let mut status : i32 = -1i32;
             if check_if_png(*argv.offset(1isize) as (*const u8)) != 0 {
                 status = load_png(q,*argv.offset(1isize) as (*const u8));
             } else {
                 status = load_jpeg(q,*argv.offset(1isize) as (*const u8));
             }
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
