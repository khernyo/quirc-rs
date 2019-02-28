extern crate clap;
extern crate quirc_rs;

use std::fs;
use std::path::Path;

use clap::{Arg, App};
use libc::{c_char, memset, perror, printf, puts, timespec};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

include!("util/dbgutil.rs");

static mut want_cell_dump: bool = false;
static mut want_verbose: bool = false;

#[derive(Copy)]
#[repr(C)]
pub struct result_info {
    pub file_count : i32,
    pub id_count : i32,
    pub decode_count : i32,
    pub load_time : u32,
    pub identify_time : u32,
    pub total_time : u32,
}

impl Clone for result_info {
    fn clone(&self) -> Self { *self }
}

#[no_mangle]
pub unsafe extern fn print_result(
    mut name : &str, mut info : *mut result_info
) {
    puts(
        (*b"-------------------------------------------------------------------------------\0").as_ptr(
        ) as *const c_char
    );
    printf(
        (*b"%s: %d files, %d codes, %d decoded (%d failures)\0").as_ptr() as *const c_char,
        name,
        (*info).file_count,
        (*info).id_count,
        (*info).decode_count,
        (*info).id_count - (*info).decode_count
    );
    if (*info).id_count != 0 {
        printf(
            (*b", %d%% success rate\0").as_ptr() as *const c_char,
            ((*info).decode_count * 100i32 + (*info).id_count / 2i32) / (*info).id_count
        );
    }
    printf((*b"\n\0").as_ptr() as *const c_char);
    printf(
        (*b"Total time [load: %u, identify: %u, total: %u]\n\0").as_ptr() as *const c_char,
        (*info).load_time,
        (*info).identify_time,
        (*info).total_time
    );
    if (*info).file_count != 0 {
        printf(
            (*b"Average time [load: %u, identify: %u, total: %u]\n\0").as_ptr(
            ) as *const c_char,
            (*info).load_time.wrapping_div((*info).file_count as (u32)),
            (*info).identify_time.wrapping_div((*info).file_count as (u32)),
            (*info).total_time.wrapping_div((*info).file_count as (u32))
        );
    }
}

#[no_mangle]
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

#[no_mangle]
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
        printf(
            (*b"  %-30s: %5u %5u %5u %5d %5d\n\0").as_ptr() as *const c_char,
            filename,
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
                        printf((*b"  ERROR: %s\n\n\0").as_ptr() as *const c_char,quirc_strerror(err));
                    } else {
                        printf((*b"  Decode successful:\n\0").as_ptr() as *const c_char);
                        dump_data(&mut data as (*mut quirc_data));
                        println!();
                    }
                }
            }
        }
        (*info).file_count = 1i32;
        1i32
    }
}

#[no_mangle]
pub unsafe extern fn scan_dir(
    mut decoder: *mut quirc,
    mut path : &Path,
    mut info : *mut result_info
) -> i32 {
    let entries = path.read_dir().unwrap();

    println!("{}", path.display());

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

#[no_mangle]
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

#[no_mangle]
pub unsafe extern fn run_tests(paths: Vec<&str>) -> i32 {
    let mut sum : result_info = std::mem::uninitialized();
    let mut count : i32 = 0i32;
    let mut i : i32;
    let mut decoder : *mut quirc = quirc_new();

    if decoder.is_null() {
        perror((*b"quirc_new\0").as_ptr() as *const c_char);
        -1i32
    } else {
        printf(
            (*b"  %-30s  %17s %11s\n\0").as_ptr() as *const c_char,
            (*b"\0").as_ptr(),
            (*b"Time (ms)\0").as_ptr(),
            (*b"Count\0").as_ptr()
        );
        printf(
            (*b"  %-30s  %5s %5s %5s %5s %5s\n\0").as_ptr() as *const c_char,
            (*b"Filename\0").as_ptr(),
            (*b"Load\0").as_ptr(),
            (*b"ID\0").as_ptr(),
            (*b"Total\0").as_ptr(),
            (*b"ID\0").as_ptr(),
            (*b"Dec\0").as_ptr()
        );
        puts(
            (*b"-------------------------------------------------------------------------------\0").as_ptr(
            ) as *const c_char
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

#[no_mangle]
pub unsafe extern fn _c_main(
    mut argc : i32, mut argv : *mut *mut u8
) -> i32 {
    let mut opt : i32;

    let cell_dump_arg_name = "cell-dump";
    let cell_dump_arg = Arg::with_name(cell_dump_arg_name).short("d");
    let verbose_arg_name = "verbose";
    let verbose_arg = Arg::with_name(verbose_arg_name).short("v");
    let paths_arg_name = "paths";
    let paths_arg = Arg::with_name(paths_arg_name).multiple(true).required(true);

    let args = App::new("qrtest")
        .about("quirc test program")
        .version(quirc_version())
        .author("Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>")
        .args(&[cell_dump_arg, verbose_arg, paths_arg]);

    let matches = args.get_matches();
    want_cell_dump = matches.is_present(cell_dump_arg_name);
    want_verbose = matches.is_present(verbose_arg_name);
    let paths = matches.values_of(paths_arg_name).unwrap();

    run_tests(paths.collect())
}
