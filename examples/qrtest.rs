extern {
    fn __errno_location() -> *mut i32;
    fn clock_gettime(__clock_id : i32, __tp : *mut timespec) -> i32;
    fn closedir(__dirp : *mut __dirstream) -> i32;
    fn dump_cells(code : *const quirc_code);
    fn dump_data(data : *const quirc_data);
    fn fprintf(
        __stream : *mut _IO_FILE, __format : *const u8, ...
    ) -> i32;
    fn getopt(
        ___argc : i32, ___argv : *const *mut u8, __shortopts : *const u8
    ) -> i32;
    fn load_jpeg(q : *mut quirc, filename : *const u8) -> i32;
    fn load_png(q : *mut quirc, filename : *const u8) -> i32;
    fn lstat(__file : *const u8, __buf : *mut stat) -> i32;
    fn memset(
        __s : *mut ::std::os::raw::c_void, __c : i32, __n : usize
    ) -> *mut ::std::os::raw::c_void;
    fn opendir(__name : *const u8) -> *mut __dirstream;
    static mut optind : i32;
    fn perror(__s : *const u8);
    fn printf(__format : *const u8, ...) -> i32;
    fn puts(__s : *const u8) -> i32;
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
    fn readdir(__dirp : *mut __dirstream) -> *mut dirent;
    fn snprintf(
        __s : *mut u8, __maxlen : usize, __format : *const u8, ...
    ) -> i32;
    static mut stderr : *mut _IO_FILE;
    fn strcasecmp(__s1 : *const u8, __s2 : *const u8) -> i32;
    fn strerror(__errnum : i32) -> *mut u8;
    fn strlen(__s : *const u8) -> usize;
}

static mut want_verbose : i32 = 0i32;

static mut want_cell_dump : i32 = 0i32;

static mut decoder : *mut quirc = 0 as (*mut quirc);

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
    mut name : *const u8, mut info : *mut result_info
) {
    puts(
        (*b"-------------------------------------------------------------------------------\0").as_ptr(
        )
    );
    printf(
        (*b"%s: %d files, %d codes, %d decoded (%d failures)\0").as_ptr(),
        name,
        (*info).file_count,
        (*info).id_count,
        (*info).decode_count,
        (*info).id_count - (*info).decode_count
    );
    if (*info).id_count != 0 {
        printf(
            (*b", %d%% success rate\0").as_ptr(),
            ((*info).decode_count * 100i32 + (*info).id_count / 2i32) / (*info).id_count
        );
    }
    printf((*b"\n\0").as_ptr());
    printf(
        (*b"Total time [load: %u, identify: %u, total: %u]\n\0").as_ptr(),
        (*info).load_time,
        (*info).identify_time,
        (*info).total_time
    );
    if (*info).file_count != 0 {
        printf(
            (*b"Average time [load: %u, identify: %u, total: %u]\n\0").as_ptr(
            ),
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
    mut path : *const u8,
    mut filename : *const u8,
    mut info : *mut result_info
) -> i32 {
    let mut loader : unsafe extern fn(*mut quirc, *const u8) -> i32;
    let mut len : i32 = strlen(filename) as (i32);
    let mut ext : *const u8;
    let mut tp : timespec;
    let mut start : u32;
    let mut total_start : u32;
    let mut ret : i32;
    let mut i : i32;
    'loop1: loop {
        if !(len >= 0i32 && (*filename.offset(
                                  len as (isize)
                              ) as (i32) != b'.' as (i32))) {
            break;
        }
        len = len - 1;
    }
    ext = filename.offset(len as (isize)).offset(1isize);
    if strcasecmp(ext,(*b"jpg\0").as_ptr()) == 0i32 || strcasecmp(
                                                           ext,
                                                           (*b"jpeg\0").as_ptr()
                                                       ) == 0i32 {
        loader = load_jpeg;
    } else if strcasecmp(ext,(*b"png\0").as_ptr()) == 0i32 {
        loader = load_png;
    } else {
        return 0i32;
    }
    clock_gettime(2i32,&mut tp as (*mut timespec));
    total_start = {
                      start = (tp.tv_sec * 1000isize + tp.tv_nsec / 1000000isize) as (u32);
                      start
                  };
    ret = loader(decoder,path);
    clock_gettime(2i32,&mut tp as (*mut timespec));
    (*info).load_time = ((tp.tv_sec * 1000isize + tp.tv_nsec / 1000000isize) as (u32)).wrapping_sub(
                            start
                        );
    if ret < 0i32 {
        fprintf(stderr,(*b"%s: load failed\n\0").as_ptr(),filename);
        -1i32
    } else {
        clock_gettime(2i32,&mut tp as (*mut timespec));
        start = (tp.tv_sec * 1000isize + tp.tv_nsec / 1000000isize) as (u32);
        quirc_end(decoder);
        clock_gettime(2i32,&mut tp as (*mut timespec));
        (*info).identify_time = ((tp.tv_sec * 1000isize + tp.tv_nsec / 1000000isize) as (u32)).wrapping_sub(
                                    start
                                );
        (*info).id_count = quirc_count(decoder as (*const quirc));
        i = 0i32;
        'loop9: loop {
            if !(i < (*info).id_count) {
                break;
            }
            let mut code : quirc_code;
            let mut data : quirc_data;
            quirc_extract(
                decoder as (*const quirc),
                i,
                &mut code as (*mut quirc_code)
            );
            if quirc_decode(
                   &mut code as (*mut quirc_code) as (*const quirc_code),
                   &mut data as (*mut quirc_data)
               ) == 0 {
                (*info).decode_count = (*info).decode_count + 1;
            }
            i = i + 1;
        }
        clock_gettime(2i32,&mut tp as (*mut timespec));
        (*info).total_time = (*info).total_time.wrapping_add(
                                 ((tp.tv_sec * 1000isize + tp.tv_nsec / 1000000isize) as (u32)).wrapping_sub(
                                     total_start
                                 )
                             );
        printf(
            (*b"  %-30s: %5u %5u %5u %5d %5d\n\0").as_ptr(),
            filename,
            (*info).load_time,
            (*info).identify_time,
            (*info).total_time,
            (*info).id_count,
            (*info).decode_count
        );
        if want_cell_dump != 0 || want_verbose != 0 {
            i = 0i32;
            'loop12: loop {
                if !(i < (*info).id_count) {
                    break;
                }
                let mut code : quirc_code;
                quirc_extract(
                    decoder as (*const quirc),
                    i,
                    &mut code as (*mut quirc_code)
                );
                if want_cell_dump != 0 {
                    dump_cells(&mut code as (*mut quirc_code) as (*const quirc_code));
                    printf((*b"\n\0").as_ptr());
                }
                if want_verbose != 0 {
                    let mut data : quirc_data;
                    let mut err
                        : Enum1
                        = quirc_decode(
                              &mut code as (*mut quirc_code) as (*const quirc_code),
                              &mut data as (*mut quirc_data)
                          );
                    if err != 0 {
                        printf((*b"  ERROR: %s\n\n\0").as_ptr(),quirc_strerror(err));
                    } else {
                        printf((*b"  Decode successful:\n\0").as_ptr());
                        dump_data(&mut data as (*mut quirc_data) as (*const quirc_data));
                        printf((*b"\n\0").as_ptr());
                    }
                }
                i = i + 1;
            }
        }
        (*info).file_count = 1i32;
        1i32
    }
}

#[no_mangle]
pub unsafe extern fn scan_dir(
    mut path : *const u8,
    mut filename : *const u8,
    mut info : *mut result_info
) -> i32 {
    let mut d : *mut __dirstream = opendir(path);
    let mut ent : *mut dirent;
    let mut count : i32 = 0i32;
    if d.is_null() {
        fprintf(
            stderr,
            (*b"%s: opendir: %s\n\0").as_ptr(),
            path,
            strerror(*__errno_location())
        );
        -1i32
    } else {
        printf((*b"%s:\n\0").as_ptr(),path);
        'loop2: loop {
            if {
                   ent = readdir(d);
                   ent
               }.is_null(
               ) {
                break;
            }
            if !((*ent).d_name[0usize] as (i32) != b'.' as (i32)) {
                continue;
            }
            let mut fullpath : [u8; 1024];
            let mut sub : result_info;
            snprintf(
                fullpath.as_mut_ptr(),
                ::std::mem::size_of::<[u8; 1024]>(),
                (*b"%s/%s\0").as_ptr(),
                path,
                (*ent).d_name.as_mut_ptr()
            );
            if !(test_scan(
                     fullpath.as_mut_ptr() as (*const u8),
                     &mut sub as (*mut result_info)
                 ) > 0i32) {
                continue;
            }
            add_result(info,&mut sub as (*mut result_info));
            count = count + 1;
        }
        closedir(d);
        if count > 1i32 {
            print_result(filename,info);
            puts((*b"\0").as_ptr());
        }
        (count > 0i32) as (i32)
    }
}

#[no_mangle]
pub unsafe extern fn test_scan(
    mut path : *const u8, mut info : *mut result_info
) -> i32 {
    let mut len : i32 = strlen(path) as (i32);
    let mut st : stat;
    let mut filename : *const u8;
    memset(
        info as (*mut ::std::os::raw::c_void),
        0i32,
        ::std::mem::size_of::<result_info>()
    );
    'loop1: loop {
        if !(len >= 0i32 && (*path.offset(
                                  len as (isize)
                              ) as (i32) != b'/' as (i32))) {
            break;
        }
        len = len - 1;
    }
    filename = path.offset(len as (isize)).offset(1isize);
    if lstat(path,&mut st as (*mut stat)) < 0i32 {
        fprintf(
            stderr,
            (*b"%s: lstat: %s\n\0").as_ptr(),
            path,
            strerror(*__errno_location())
        );
        -1i32
    } else if st.st_mode & 0o170000u32 == 0o100000u32 {
        scan_file(path,filename,info)
    } else if st.st_mode & 0o170000u32 == 0o40000u32 {
        scan_dir(path,filename,info)
    } else {
        0i32
    }
}

#[no_mangle]
pub unsafe extern fn run_tests(
    mut argc : i32, mut argv : *mut *mut u8
) -> i32 {
    let mut sum : result_info;
    let mut count : i32 = 0i32;
    let mut i : i32;
    decoder = quirc_new();
    if decoder.is_null() {
        perror((*b"quirc_new\0").as_ptr());
        -1i32
    } else {
        printf(
            (*b"  %-30s  %17s %11s\n\0").as_ptr(),
            (*b"\0").as_ptr(),
            (*b"Time (ms)\0").as_ptr(),
            (*b"Count\0").as_ptr()
        );
        printf(
            (*b"  %-30s  %5s %5s %5s %5s %5s\n\0").as_ptr(),
            (*b"Filename\0").as_ptr(),
            (*b"Load\0").as_ptr(),
            (*b"ID\0").as_ptr(),
            (*b"Total\0").as_ptr(),
            (*b"ID\0").as_ptr(),
            (*b"Dec\0").as_ptr()
        );
        puts(
            (*b"-------------------------------------------------------------------------------\0").as_ptr(
            )
        );
        memset(
            &mut sum as (*mut result_info) as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<result_info>()
        );
        i = 0i32;
        'loop2: loop {
            if !(i < argc) {
                break;
            }
            let mut info : result_info;
            if test_scan(
                   *argv.offset(i as (isize)) as (*const u8),
                   &mut info as (*mut result_info)
               ) > 0i32 {
                add_result(
                    &mut sum as (*mut result_info),
                    &mut info as (*mut result_info)
                );
                count = count + 1;
            }
            i = i + 1;
        }
        if count > 1i32 {
            print_result(
                (*b"TOTAL\0").as_ptr(),
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
    let mut _currentBlock;
    let mut opt : i32;
    printf((*b"quirc test program\n\0").as_ptr());
    printf(
        (*b"Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>\n\0").as_ptr(
        )
    );
    printf((*b"Library version: %s\n\0").as_ptr(),quirc_version());
    printf((*b"\n\0").as_ptr());
    'loop1: loop {
        if !({
                 opt = getopt(argc,argv as (*const *mut u8),(*b"vd\0").as_ptr());
                 opt
             } >= 0i32) {
            _currentBlock = 2;
            break;
        }
        if opt == b'?' as (i32) {
            _currentBlock = 8;
            break;
        }
        if opt == b'd' as (i32) {
            want_cell_dump = 1i32;
        } else {
            if !(opt == b'v' as (i32)) {
                continue;
            }
            want_verbose = 1i32;
        }
    }
    if _currentBlock == 2 {
        argv = argv.offset(optind as (isize));
        argc = argc - optind;
        run_tests(argc,argv)
    } else {
        -1i32
    }
}
