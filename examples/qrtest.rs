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
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

extern crate clap;
extern crate quirc_rs;

use std::ffi::CStr;
use std::fs;
use std::path::Path;

use clap::{Arg, App};
use libc::{c_char, c_void, malloc, memcmp, memcpy, memset, perror, puts, timespec};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

include!("../tests/util/dbgutil.rs");

static mut want_cell_dump: bool = false;
static mut want_validate: bool = false;
static mut want_verbose: bool = false;

pub unsafe extern fn print_result(
    mut name : &str, mut info : *mut result_info
) {
    println!("-------------------------------------------------------------------------------");
    print!(
        "{}: {} files, {} codes, {} decoded ({} failures)",
        name,
        (*info).file_count,
        (*info).id_count,
        (*info).decode_count,
        (*info).id_count - (*info).decode_count
    );
    if (*info).id_count != 0 {
        print!(
            ", {}% success rate",
            ((*info).decode_count * 100i32 + (*info).id_count / 2i32) / (*info).id_count
        );
    }
    println!();
    println!(
        "Total time [load: {}, identify: {}, total: {}]",
        (*info).load_time,
        (*info).identify_time,
        (*info).total_time
    );
    if (*info).file_count != 0 {
        println!(
            "Average time [load: {}, identify: {}, total: {}]",
            (*info).load_time.wrapping_div((*info).file_count as (u32)),
            (*info).identify_time.wrapping_div((*info).file_count as (u32)),
            (*info).total_time.wrapping_div((*info).file_count as (u32))
        );
    }
}

pub unsafe extern fn add_result(
    mut sum : *mut result_info, mut inf : *mut result_info
) {
    (*sum).file_count = (*sum).file_count + (*inf).file_count;
    (*sum).id_count = (*sum).id_count + (*inf).id_count;
    (*sum).decode_count = (*sum).decode_count + (*inf).decode_count;
    (*sum).load_time = (*sum).load_time.wrapping_add((*inf).load_time);
    (*sum).identify_time = (*sum).identify_time.wrapping_add(
                               (*inf).identify_time
                           );
    (*sum).total_time = (*sum).total_time.wrapping_add(
                            (*inf).total_time
                        );
}

pub unsafe extern fn scan_file(
    mut decoder: *mut quirc,
    mut path : &Path,
    mut info : *mut result_info
) -> i32 {
    let filename = path.file_name().unwrap();
    let mut len = filename.len();
    let mut tp : libc::timespec = std::mem::uninitialized();
    let mut start : u32;
    let mut total_start : u32;
    let mut ret : i32;
    let mut i : i32;

    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    total_start = {
                      start = (tp.tv_sec * 1000i64 + tp.tv_nsec / 1000000i64) as (u32);
                      start
                  };
    ret = load_image(decoder, path);
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID,&mut tp as (*mut timespec));
    (*info).load_time = ((tp.tv_sec * 1000i64 + tp.tv_nsec / 1000000i64) as (u32)).wrapping_sub(
                            start
                        );
    let image_bytes = if want_validate {
        let dst = malloc(((*decoder).w * (*decoder).h) as usize);
        memcpy(dst, (*decoder).image as *const c_void, ((*decoder).w * (*decoder).h) as usize);
        dst
    } else {
        0 as *const c_void
    };
    if ret < 0i32 {
        eprintln!("{}: load failed", filename.to_str().unwrap());
        -1i32
    } else {
        libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
        start = (tp.tv_sec * 1000i64 + tp.tv_nsec / 1000000i64) as (u32);
        quirc_end(decoder);
        libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
        (*info).identify_time = ((tp.tv_sec * 1000i64 + tp.tv_nsec / 1000000i64) as (u32)).wrapping_sub(
                                    start
                                );
        (*info).id_count = quirc_count(decoder as (*const quirc));
        for i in 0..(*info).id_count {
            let mut code : quirc_code = std::mem::uninitialized();
            let mut data : quirc_data = std::mem::uninitialized();
            quirc_extract(
                decoder as (*mut quirc),
                i,
                &mut code as (*mut quirc_code)
            );
            if quirc_decode(
                   &mut code as (*mut quirc_code) as (*const quirc_code),
                   &mut data as (*mut quirc_data)
               ) == Enum1::QUIRC_SUCCESS {
                (*info).decode_count = (*info).decode_count + 1;
            }
        }
        libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID,&mut tp as (*mut timespec));
        (*info).total_time = (*info).total_time.wrapping_add(
                                 ((tp.tv_sec * 1000i64 + tp.tv_nsec / 1000000i64) as (u32)).wrapping_sub(
                                     total_start
                                 )
                             );
          println!(
            "  {:-30}: {:5} {:5} {:5} {:5} {:5}",
            filename.to_str().unwrap(),
            (*info).load_time,
            (*info).identify_time,
            (*info).total_time,
            (*info).id_count,
            (*info).decode_count
        );
        if want_cell_dump || want_verbose {
            for i in 0..(*info).id_count {
                let mut code : quirc_code = std::mem::uninitialized();
                quirc_extract(
                    decoder as (*mut quirc),
                    i,
                    &mut code as (*mut quirc_code)
                );
                if want_cell_dump {
                    dump_cells(&mut code as (*mut quirc_code) as (*const quirc_code));
                    println!();
                }
                if want_verbose {
                    let mut data : quirc_data = std::mem::uninitialized();
                    let mut err
                        : Enum1
                        = quirc_decode(
                              &mut code as (*mut quirc_code) as (*const quirc_code),
                              &mut data as (*mut quirc_data)
                          );
                    if err != Enum1::QUIRC_SUCCESS {
                        println!("  ERROR: {}", quirc_strerror(err));
                        println!();
                    } else {
                        println!("  Decode successful:");
                        dump_data(&mut data as (*mut quirc_data));
                        println!();
                    }
                }
            }
        }
        if want_validate {
            validate(decoder, path, image_bytes);
        }
        (*info).file_count = 1i32;
        1i32
    }
}

pub unsafe extern fn scan_dir(
    mut decoder: *mut quirc,
    mut path : &Path,
    mut info : *mut result_info
) -> i32 {
    let entries = path.read_dir().unwrap();

    println!("{}:", path.display());

    let mut count : i32 = 0i32;
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().chars().next().unwrap() != '.' {
            let mut sub : result_info = std::mem::uninitialized();
            let p = entry.path();
            let fullpath = p.as_path();

            if test_scan(decoder, fullpath, &mut sub as (*mut result_info)) > 0i32 {
                add_result(info,&mut sub as (*mut result_info));
                count = count + 1;
            }
        }
    }

    if count > 1i32 {
        print_result(path.file_name().unwrap().to_str().unwrap(), info);
        puts((*b"\0").as_ptr() as *const c_char);
    }
    (count > 0i32) as (i32)
}

pub unsafe extern fn test_scan(
    mut decoder: *mut quirc,
    mut path : &Path,
    mut info : *mut result_info
) -> i32 {
    memset(
        info as (*mut ::std::os::raw::c_void),
        0i32,
        ::std::mem::size_of::<result_info>()
    );

    if path.is_file() {
        scan_file(decoder, path, info)
    } else if path.is_dir() {
        scan_dir(decoder, path, info)
    } else {
        0
    }
}

pub unsafe extern fn run_tests(paths: Vec<&str>) -> i32 {
    let mut sum : result_info = std::mem::uninitialized();
    let mut count : i32 = 0i32;
    let mut i : i32;
    let mut decoder : *mut quirc = quirc_new();

    if decoder.is_null() {
        perror((*b"quirc_new\0").as_ptr() as *const c_char);
        -1i32
    } else {
        println!(
            "  {:30}  {:>17} {:>11}",
            "",
            "Time (ms)",
            "Count"
        );
        println!(
            "  {:<30}  {:>5} {:>5} {:>5} {:>5} {:>5}",
            "Filename",
            "Load",
            "ID",
            "Total",
            "ID",
            "Dec"
        );
        println!(
            "-------------------------------------------------------------------------------"
        );
        memset(
            &mut sum as (*mut result_info) as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<result_info>()
        );
        for path in paths {
            let mut info : result_info = std::mem::uninitialized();
            if test_scan(
                   decoder,
                   Path::new(path),
                   &mut info as (*mut result_info)
               ) > 0i32 {
                add_result(
                    &mut sum as (*mut result_info),
                    &mut info as (*mut result_info)
                );
                count = count + 1;
            }
        }
        if count > 1i32 {
            print_result(
                "TOTAL",
                &mut sum as (*mut result_info)
            );
        }
        quirc_destroy(decoder);
        0i32
    }
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

pub unsafe extern fn _c_main(
    mut argc : i32, mut argv : *mut *mut u8
) -> i32 {
    let mut opt : i32;

    let cell_dump_arg_name = "cell-dump";
    let cell_dump_arg = Arg::with_name(cell_dump_arg_name).short("d").help("Dumps cell data");
    let no_validation_arg_name = "no-validate";
    let no_validation_arg = Arg::with_name(no_validation_arg_name).long(no_validation_arg_name).help("Disables validating the results against quirc");
    let verbose_arg_name = "verbose";
    let verbose_arg = Arg::with_name(verbose_arg_name).short("v").help("Enables verbose output");
    let paths_arg_name = "paths";
    let paths_arg = Arg::with_name(paths_arg_name).multiple(true).required(true);

    let args = App::new("qrtest")
        .about("quirc test program")
        .version(quirc_version())
        .author("Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>")
        .args(&[cell_dump_arg, no_validation_arg, verbose_arg, paths_arg]);

    let matches = args.get_matches();
    want_cell_dump = matches.is_present(cell_dump_arg_name);
    want_validate = !matches.is_present(no_validation_arg_name);
    want_verbose = matches.is_present(verbose_arg_name);
    let paths = matches.values_of(paths_arg_name).unwrap();

    run_tests(paths.collect())
}
