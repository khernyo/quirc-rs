extern {
    fn _setjmp(__env : *mut __jmp_buf_tag) -> i32;
    fn fclose(__stream : *mut _IO_FILE) -> i32;
    fn fopen(
        __filename : *const u8, __modes : *const u8
    ) -> *mut _IO_FILE;
    fn fprintf(
        __stream : *mut _IO_FILE, __format : *const u8, ...
    ) -> i32;
    fn fread(
        __ptr : *mut ::std::os::raw::c_void,
        __size : usize,
        __n : usize,
        __stream : *mut _IO_FILE
    ) -> usize;
    fn jpeg_CreateDecompress(
        cinfo : *mut jpeg_decompress_struct,
        version : i32,
        structsize : usize
    );
    fn jpeg_destroy_decompress(cinfo : *mut jpeg_decompress_struct);
    fn jpeg_finish_decompress(
        cinfo : *mut jpeg_decompress_struct
    ) -> i32;
    fn jpeg_read_header(
        cinfo : *mut jpeg_decompress_struct, require_image : i32
    ) -> i32;
    fn jpeg_read_scanlines(
        cinfo : *mut jpeg_decompress_struct,
        scanlines : *mut *mut u8,
        max_lines : u32
    ) -> u32;
    fn jpeg_start_decompress(
        cinfo : *mut jpeg_decompress_struct
    ) -> i32;
    fn jpeg_std_error(
        err : *mut jpeg_error_mgr
    ) -> *mut jpeg_error_mgr;
    fn jpeg_stdio_src(
        cinfo : *mut jpeg_decompress_struct, infile : *mut _IO_FILE
    );
    fn longjmp(__env : *mut __jmp_buf_tag, __val : i32);
    fn memset(
        __s : *mut ::std::os::raw::c_void, __c : i32, __n : usize
    ) -> *mut ::std::os::raw::c_void;
    fn perror(__s : *const u8);
    fn png_create_info_struct(
        png_ptr : *const png_struct_def
    ) -> *mut png_info_def;
    fn png_create_read_struct(
        user_png_ver : *const u8,
        error_ptr : *mut ::std::os::raw::c_void,
        error_fn : unsafe extern fn(*mut png_struct_def, *const u8),
        warn_fn : unsafe extern fn(*mut png_struct_def, *const u8)
    ) -> *mut png_struct_def;
    fn png_destroy_read_struct(
        png_ptr_ptr : *mut *mut png_struct_def,
        info_ptr_ptr : *mut *mut png_info_def,
        end_info_ptr_ptr : *mut *mut png_info_def
    );
    fn png_get_bit_depth(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> u8;
    fn png_get_color_type(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> u8;
    fn png_get_image_height(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> u32;
    fn png_get_image_width(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> u32;
    fn png_get_interlace_type(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> u8;
    fn png_get_rowbytes(
        png_ptr : *const png_struct_def, info_ptr : *const png_info_def
    ) -> usize;
    fn png_get_valid(
        png_ptr : *const png_struct_def,
        info_ptr : *const png_info_def,
        flag : u32
    ) -> u32;
    fn png_init_io(png_ptr : *mut png_struct_def, fp : *mut _IO_FILE);
    fn png_read_end(
        png_ptr : *mut png_struct_def, info_ptr : *mut png_info_def
    );
    fn png_read_info(
        png_ptr : *mut png_struct_def, info_ptr : *mut png_info_def
    );
    fn png_read_rows(
        png_ptr : *mut png_struct_def,
        row : *mut *mut u8,
        display_row : *mut *mut u8,
        num_rows : u32
    );
    fn png_read_update_info(
        png_ptr : *mut png_struct_def, info_ptr : *mut png_info_def
    );
    fn png_set_expand_gray_1_2_4_to_8(png_ptr : *mut png_struct_def);
    fn png_set_interlace_handling(
        png_ptr : *mut png_struct_def
    ) -> i32;
    fn png_set_longjmp_fn(
        png_ptr : *mut png_struct_def,
        longjmp_fn : unsafe extern fn(*mut __jmp_buf_tag, i32),
        jmp_buf_size : usize
    ) -> *mut [__jmp_buf_tag; 1];
    fn png_set_palette_to_rgb(png_ptr : *mut png_struct_def);
    fn png_set_rgb_to_gray_fixed(
        png_ptr : *mut png_struct_def,
        error_action : i32,
        red : i32,
        green : i32
    );
    fn png_set_scale_16(png_ptr : *mut png_struct_def);
    fn png_set_strip_alpha(png_ptr : *mut png_struct_def);
    fn png_set_tRNS_to_alpha(png_ptr : *mut png_struct_def);
    fn png_sig_cmp(
        sig : *const u8, start : usize, num_to_check : usize
    ) -> i32;
    fn printf(__format : *const u8, ...) -> i32;
    fn quirc_begin(
        q : *mut quirc, w : *mut i32, h : *mut i32
    ) -> *mut u8;
    fn quirc_resize(q : *mut quirc, w : i32, h : i32) -> i32;
    static mut stderr : *mut _IO_FILE;
}

unsafe extern fn data_type_str(mut dt : i32) -> *const u8 {
    if dt == 8i32 {
        (*b"KANJI\0").as_ptr()
    } else if dt == 4i32 {
        (*b"BYTE\0").as_ptr()
    } else if dt == 2i32 {
        (*b"ALPHA\0").as_ptr()
    } else if dt == 1i32 {
        (*b"NUMERIC\0").as_ptr()
    } else {
        (*b"unknown\0").as_ptr()
    }
}

#[no_mangle]
pub unsafe extern fn dump_data(mut data : *mut quirc_data) {
    printf((*b"    Version: %d\n\0").as_ptr(),(*data).version);
    printf(
        (*b"    ECC level: %c\n\0").as_ptr(),
        (*b"MLHQ\0")[(*data).ecc_level as (usize)] as (i32)
    );
    printf((*b"    Mask: %d\n\0").as_ptr(),(*data).mask);
    printf(
        (*b"    Data type: %d (%s)\n\0").as_ptr(),
        (*data).data_type,
        data_type_str((*data).data_type)
    );
    printf((*b"    Length: %d\n\0").as_ptr(),(*data).payload_len);
    printf(
        (*b"    Payload: %s\n\0").as_ptr(),
        (*data).payload.as_mut_ptr()
    );
    if (*data).eci != 0 {
        printf((*b"    ECI: %d\n\0").as_ptr(),(*data).eci);
    }
}

#[no_mangle]
pub unsafe extern fn dump_cells(mut code : *const quirc_code) {
    let mut u : i32;
    let mut v : i32;
    printf((*b"    %d cells, corners:\0").as_ptr(),(*code).size);
    u = 0i32;
    'loop1: loop {
        if !(u < 4i32) {
            break;
        }
        printf(
            (*b" (%d,%d)\0").as_ptr(),
            (*code).corners[u as (usize)].x,
            (*code).corners[u as (usize)].y
        );
        u = u + 1;
    }
    printf((*b"\n\0").as_ptr());
    v = 0i32;
    'loop3: loop {
        if !(v < (*code).size) {
            break;
        }
        printf((*b"    \0").as_ptr());
        u = 0i32;
        'loop6: loop {
            if !(u < (*code).size) {
                break;
            }
            let mut p : i32 = v * (*code).size + u;
            if (*code).cell_bitmap[
                   (p >> 3i32) as (usize)
               ] as (i32) & 1i32 << (p & 7i32) != 0 {
                printf((*b"[]\0").as_ptr());
            } else {
                printf((*b"  \0").as_ptr());
            }
            u = u + 1;
        }
        printf((*b"\n\0").as_ptr());
        v = v + 1;
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct my_jpeg_error {
    pub base : jpeg_error_mgr,
    pub env : [__jmp_buf_tag; 1],
}

impl Clone for my_jpeg_error {
    fn clone(&self) -> Self { *self }
}

unsafe extern fn my_output_message(
    mut com : *mut jpeg_common_struct
) {
    let mut err
        : *mut my_jpeg_error
        = (*com).err as (*mut my_jpeg_error);
    let mut buf : [u8; 200];
    ((*err).base.format_message)(com,buf.as_mut_ptr());
    fprintf(stderr,(*b"JPEG error: %s\n\0").as_ptr(),buf.as_mut_ptr());
}

unsafe extern fn my_error_exit(mut com : *mut jpeg_common_struct) {
    let mut err
        : *mut my_jpeg_error
        = (*com).err as (*mut my_jpeg_error);
    my_output_message(com);
    longjmp((*err).env.as_mut_ptr(),0i32);
}

unsafe extern fn my_error_mgr(
    mut err : *mut my_jpeg_error
) -> *mut jpeg_error_mgr {
    jpeg_std_error(&mut (*err).base as (*mut jpeg_error_mgr));
    (*err).base.error_exit = my_error_exit;
    (*err).base.output_message = my_output_message;
    &mut (*err).base as (*mut jpeg_error_mgr)
}

#[no_mangle]
pub unsafe extern fn load_jpeg(
    mut q : *mut quirc, mut filename : *const u8
) -> i32 {
    let mut infile
        : *mut _IO_FILE
        = fopen(filename,(*b"rb\0").as_ptr());
    let mut dinfo : jpeg_decompress_struct = std::mem::uninitialized();
    let mut err : my_jpeg_error;
    let mut image : *mut u8;
    let mut y : i32;
    if infile.is_null() {
        perror((*b"can\'t open input file\0").as_ptr());
        -1i32
    } else {
        memset(
            &mut dinfo as (*mut jpeg_decompress_struct) as (*mut ::std::os::raw::c_void),
            0i32,
            ::std::mem::size_of::<jpeg_decompress_struct>()
        );
        dinfo.err = my_error_mgr(
                        &mut err as (*mut my_jpeg_error)
                    ) as (*mut jpeg_error_mgr);
        if _setjmp(err.env.as_mut_ptr()) == 0 {
            jpeg_CreateDecompress(
                &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct),
                62i32,
                ::std::mem::size_of::<jpeg_decompress_struct>()
            );
            jpeg_stdio_src(
                &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct),
                infile
            );
            jpeg_read_header(
                &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct),
                1i32
            );
            dinfo.output_components = 1i32;
            dinfo.out_color_space = Enum1::JCS_GRAYSCALE;
            jpeg_start_decompress(
                &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct)
            );
            if dinfo.output_components != 1i32 {
                fprintf(
                    stderr,
                    (*b"Unexpected number of output components: %d\0").as_ptr(),
                    dinfo.output_components
                );
            } else if !(quirc_resize(
                            q,
                            dinfo.output_width as (i32),
                            dinfo.output_height as (i32)
                        ) < 0i32) {
                image = quirc_begin(
                            q,
                            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
                            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32)
                        );
                y = 0i32;
                'loop5: loop {
                    if !(y as (u32) < dinfo.output_height) {
                        break;
                    }
                    let mut row_pointer
                        : *mut u8
                        = image.offset(
                              (y as (u32)).wrapping_mul(dinfo.output_width) as (isize)
                          );
                    jpeg_read_scanlines(
                        &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct),
                        &mut row_pointer as (*mut *mut u8),
                        1u32
                    );
                    y = y + 1;
                }
                jpeg_finish_decompress(
                    &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct)
                );
                fclose(infile);
                jpeg_destroy_decompress(
                    &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct)
                );
                return 0i32;
            }
        }
        fclose(infile);
        jpeg_destroy_decompress(
            &mut dinfo as (*mut jpeg_decompress_struct) as (*mut jpeg_decompress_struct)
        );
        -1i32
    }
}

#[no_mangle]
pub unsafe extern fn check_if_png(mut filename : *const u8) -> i32 {
    let mut ret : i32 = 0i32;
    let mut infile
        : *mut _IO_FILE
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut _IO_FILE);
    let mut buf : [u8; 4];
    if !({
             infile = fopen(filename,(*b"rb\0").as_ptr());
             infile
         } == 0i32 as (*mut ::std::os::raw::c_void) as (*mut _IO_FILE)) {
        if !(fread(
                 buf.as_mut_ptr() as (*mut ::std::os::raw::c_void),
                 1usize,
                 4usize,
                 infile
             ) != 4usize) {
            if png_sig_cmp(
                   buf.as_mut_ptr() as (*const u8),
                   0usize,
                   4usize
               ) == 0i32 {
                ret = 1i32;
            }
        }
    }
    if !infile.is_null() {
        fclose(infile);
    }
    ret
}

#[no_mangle]
pub unsafe extern fn load_png(
    mut q : *mut quirc, mut filename : *const u8
) -> i32 {
    let mut width : i32;
    let mut height : i32;
    let mut rowbytes : i32;
    let mut interlace_type : i32;
    let mut number_passes : i32 = 1i32;
    let mut trns : u32;
    let mut color_type : u8;
    let mut bit_depth : u8;
    let mut png_ptr
        : *mut png_struct_def
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut png_struct_def);
    let mut info_ptr
        : *mut png_info_def
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut png_info_def);
    let mut infile
        : *mut _IO_FILE
        = 0i32 as (*mut ::std::os::raw::c_void) as (*mut _IO_FILE);
    let mut image : *mut u8;
    let mut ret : i32 = -1i32;
    let mut pass : i32;
    if !({
             infile = fopen(filename,(*b"rb\0").as_ptr());
             infile
         } == 0i32 as (*mut ::std::os::raw::c_void) as (*mut _IO_FILE)) {
        png_ptr = png_create_read_struct(
                      (*b"1.6.36\0").as_ptr(),
                      0i32 as (*mut ::std::os::raw::c_void),
                      0i32 as (*mut ::std::os::raw::c_void) as (unsafe extern fn(*mut png_struct_def, *const u8)),
                      0i32 as (*mut ::std::os::raw::c_void) as (unsafe extern fn(*mut png_struct_def, *const u8))
                  );
        if !png_ptr.is_null() {
            info_ptr = png_create_info_struct(
                           png_ptr as (*const png_struct_def)
                       );
            if !info_ptr.is_null() {
                if _setjmp(
                       (*png_set_longjmp_fn(
                             png_ptr,
                             longjmp,
                             ::std::mem::size_of::<[__jmp_buf_tag; 1]>()
                         )).as_mut_ptr(
                       )
                   ) == 0 {
                    png_init_io(png_ptr,infile);
                    png_read_info(png_ptr,info_ptr);
                    color_type = png_get_color_type(
                                     png_ptr as (*const png_struct_def),
                                     info_ptr as (*const png_info_def)
                                 );
                    bit_depth = png_get_bit_depth(
                                    png_ptr as (*const png_struct_def),
                                    info_ptr as (*const png_info_def)
                                );
                    interlace_type = png_get_interlace_type(
                                         png_ptr as (*const png_struct_def),
                                         info_ptr as (*const png_info_def)
                                     ) as (i32);
                    if color_type as (i32) == 0i32 && (bit_depth as (i32) < 8i32) {
                        png_set_expand_gray_1_2_4_to_8(png_ptr);
                    }
                    if {
                           trns = png_get_valid(
                                      png_ptr as (*const png_struct_def),
                                      info_ptr as (*const png_info_def),
                                      0x10u32
                                  );
                           trns
                       } != 0 {
                        png_set_tRNS_to_alpha(png_ptr);
                    }
                    if bit_depth as (i32) == 16i32 {
                        png_set_scale_16(png_ptr);
                    }
                    if trns != 0 || color_type as (i32) & 4i32 != 0 {
                        png_set_strip_alpha(png_ptr);
                    }
                    if color_type as (i32) == 2i32 | 1i32 {
                        png_set_palette_to_rgb(png_ptr);
                    }
                    if color_type as (i32) == 2i32 | 1i32 || color_type as (i32) == 2i32 || color_type as (i32) == 2i32 | 4i32 {
                        png_set_rgb_to_gray_fixed(png_ptr,1i32,-1i32,-1i32);
                    }
                    if interlace_type != 0i32 {
                        number_passes = png_set_interlace_handling(png_ptr);
                    }
                    png_read_update_info(png_ptr,info_ptr);
                    width = png_get_image_width(
                                png_ptr as (*const png_struct_def),
                                info_ptr as (*const png_info_def)
                            ) as (i32);
                    height = png_get_image_height(
                                 png_ptr as (*const png_struct_def),
                                 info_ptr as (*const png_info_def)
                             ) as (i32);
                    rowbytes = png_get_rowbytes(
                                   png_ptr as (*const png_struct_def),
                                   info_ptr as (*const png_info_def)
                               ) as (i32);
                    if rowbytes != width {
                        fprintf(
                            stderr,
                            (*b"load_png: expected rowbytes to be %u but got %u\n\0").as_ptr(),
                            width,
                            rowbytes
                        );
                    } else if !(quirc_resize(q,width,height) < 0i32) {
                        image = quirc_begin(
                                    q,
                                    0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
                                    0i32 as (*mut ::std::os::raw::c_void) as (*mut i32)
                                );
                        pass = 0i32;
                        'loop21: loop {
                            if !(pass < number_passes) {
                                break;
                            }
                            let mut y : i32;
                            y = 0i32;
                            'loop24: loop {
                                if !(y < height) {
                                    break;
                                }
                                let mut row_pointer
                                    : *mut u8
                                    = image.offset((y * width) as (isize));
                                png_read_rows(
                                    png_ptr,
                                    &mut row_pointer as (*mut *mut u8),
                                    0i32 as (*mut ::std::os::raw::c_void) as (*mut *mut u8),
                                    1u32
                                );
                                y = y + 1;
                            }
                            pass = pass + 1;
                        }
                        png_read_end(png_ptr,info_ptr);
                        ret = 0i32;
                    }
                }
            }
        }
    }
    if !png_ptr.is_null() {
        if !info_ptr.is_null() {
            png_destroy_read_struct(
                &mut png_ptr as (*mut *mut png_struct_def),
                &mut info_ptr as (*mut *mut png_info_def),
                0i32 as (*mut ::std::os::raw::c_void) as (*mut *mut png_info_def)
            );
        } else {
            png_destroy_read_struct(
                &mut png_ptr as (*mut *mut png_struct_def),
                0i32 as (*mut ::std::os::raw::c_void) as (*mut *mut png_info_def),
                0i32 as (*mut ::std::os::raw::c_void) as (*mut *mut png_info_def)
            );
        }
    }
    if !infile.is_null() {
        fclose(infile);
    }
    ret
}
