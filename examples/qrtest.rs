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
extern crate clap;
extern crate quirc_rs;

use std::path::Path;

use clap::{App, Arg};
use libc::{memset, timespec};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

use test_utils::dbgutil::*;

static mut WANT_CELL_DUMP: bool = false;
static mut WANT_VALIDATE: bool = false;
static mut WANT_VERBOSE: bool = false;

fn ms(ts: libc::timespec) -> u32 {
    ((ts.tv_sec * 1000) + (ts.tv_nsec / 1000000)) as u32
}

#[derive(Copy)]
#[repr(C)]
pub struct ResultInfo {
    pub file_count: i32,
    pub id_count: i32,
    pub decode_count: i32,
    pub load_time: u32,
    pub identify_time: u32,
    pub total_time: u32,
}

impl Clone for ResultInfo {
    fn clone(&self) -> Self {
        *self
    }
}

pub unsafe extern "C" fn print_result(name: &str, info: *mut ResultInfo) {
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
            (*info)
                .identify_time
                .wrapping_div((*info).file_count as (u32)),
            (*info).total_time.wrapping_div((*info).file_count as (u32))
        );
    }
}

pub unsafe extern "C" fn add_result(mut sum: *mut ResultInfo, inf: *mut ResultInfo) {
    (*sum).file_count = (*sum).file_count + (*inf).file_count;
    (*sum).id_count = (*sum).id_count + (*inf).id_count;
    (*sum).decode_count = (*sum).decode_count + (*inf).decode_count;
    (*sum).load_time = (*sum).load_time.wrapping_add((*inf).load_time);
    (*sum).identify_time = (*sum).identify_time.wrapping_add((*inf).identify_time);
    (*sum).total_time = (*sum).total_time.wrapping_add((*inf).total_time);
}

pub unsafe extern "C" fn scan_file(decoder: &mut Quirc, path: &Path, mut info: *mut ResultInfo) {
    let filename = path.file_name().unwrap();
    let mut tp: libc::timespec = std::mem::uninitialized();
    let mut start: u32;
    let total_start: u32;

    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    total_start = {
        start = ms(tp);
        start
    };
    let image_bytes = load_image(decoder, path);
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    (*info).load_time = ms(tp).wrapping_sub(start);

    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    start = ms(tp);
    quirc_identify(decoder, &image_bytes);
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    (*info).identify_time = ms(tp).wrapping_sub(start);
    (*info).id_count = quirc_count(decoder);
    for i in 0..(*info).id_count {
        let code = quirc_extract(decoder, i).unwrap();
        if let Ok(_) = quirc_decode(&code) {
            (*info).decode_count = (*info).decode_count + 1;
        }
    }
    libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut tp as (*mut timespec));
    (*info).total_time = (*info)
        .total_time
        .wrapping_add(ms(tp).wrapping_sub(total_start));
    println!(
        "  {:-30}: {:5} {:5} {:5} {:5} {:5}",
        filename.to_str().unwrap(),
        (*info).load_time,
        (*info).identify_time,
        (*info).total_time,
        (*info).id_count,
        (*info).decode_count
    );
    if WANT_CELL_DUMP || WANT_VERBOSE {
        for i in 0..(*info).id_count {
            let code = quirc_extract(decoder, i).unwrap();
            if WANT_CELL_DUMP {
                dump_cells(&code);
                println!();
            }
            if WANT_VERBOSE {
                match quirc_decode(&code) {
                    Ok(data) => {
                        println!("  Decode successful:");
                        dump_data(&data);
                        println!();
                    }
                    Err(e) => {
                        println!("  ERROR: {}", quirc_strerror(e));
                        println!();
                    }
                }
            }
        }
    }
    if WANT_VALIDATE {
        validate(decoder, &image_bytes);
    }
    (*info).file_count = 1i32;
}

pub unsafe extern "C" fn scan_dir(decoder: &mut Quirc, path: &Path, info: *mut ResultInfo) -> i32 {
    let entries = path.read_dir().unwrap();

    println!("{}:", path.display());

    let mut count: i32 = 0i32;
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().chars().next().unwrap() != '.' {
            let mut sub: ResultInfo = std::mem::uninitialized();
            let p = entry.path();
            let fullpath = p.as_path();

            if test_scan(decoder, fullpath, &mut sub) > 0i32 {
                add_result(info, &mut sub);
                count = count + 1;
            }
        }
    }

    if count > 1i32 {
        print_result(path.file_name().unwrap().to_str().unwrap(), info);
        println!();
    }
    (count > 0i32) as (i32)
}

pub unsafe extern "C" fn test_scan(decoder: &mut Quirc, path: &Path, info: *mut ResultInfo) -> i32 {
    memset(
        info as (*mut ::std::os::raw::c_void),
        0i32,
        ::std::mem::size_of::<ResultInfo>(),
    );

    if path.is_file() {
        scan_file(decoder, path, info);
        1
    } else if path.is_dir() {
        scan_dir(decoder, path, info);
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn run_tests(paths: Vec<&str>) {
    let mut sum: ResultInfo = std::mem::uninitialized();
    let mut count: i32 = 0i32;
    let mut decoder = Quirc::new();

    println!("  {:30}  {:>17} {:>11}", "", "Time (ms)", "Count");
    println!(
        "  {:<30}  {:>5} {:>5} {:>5} {:>5} {:>5}",
        "Filename", "Load", "ID", "Total", "ID", "Dec"
    );
    println!("-------------------------------------------------------------------------------");
    memset(
        &mut sum as (*mut ResultInfo) as (*mut ::std::os::raw::c_void),
        0i32,
        ::std::mem::size_of::<ResultInfo>(),
    );
    for path in paths {
        let mut info: ResultInfo = std::mem::uninitialized();
        if test_scan(&mut decoder, Path::new(path), &mut info) > 0i32 {
            add_result(&mut sum, &mut info);
            count = count + 1;
        }
    }
    if count > 1i32 {
        print_result("TOTAL", &mut sum);
    }
}

fn main() {
    unsafe { _c_main() }
}

pub unsafe extern "C" fn _c_main() {
    let cell_dump_arg_name = "cell-dump";
    let cell_dump_arg = Arg::with_name(cell_dump_arg_name)
        .short("d")
        .help("Dumps cell data");
    let no_validation_arg_name = "no-validate";
    let no_validation_arg = Arg::with_name(no_validation_arg_name)
        .long(no_validation_arg_name)
        .help("Disables validating the results against quirc");
    let verbose_arg_name = "verbose";
    let verbose_arg = Arg::with_name(verbose_arg_name)
        .short("v")
        .help("Enables verbose output");
    let paths_arg_name = "paths";
    let paths_arg = Arg::with_name(paths_arg_name).multiple(true).required(true);

    let args = App::new("qrtest")
        .about("quirc test program")
        .version(quirc_version())
        .author("Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>")
        .args(&[cell_dump_arg, no_validation_arg, verbose_arg, paths_arg]);

    let matches = args.get_matches();
    WANT_CELL_DUMP = matches.is_present(cell_dump_arg_name);
    WANT_VALIDATE = !matches.is_present(no_validation_arg_name);
    WANT_VERBOSE = matches.is_present(verbose_arg_name);
    let paths = matches.values_of(paths_arg_name).unwrap();

    run_tests(paths.collect())
}
