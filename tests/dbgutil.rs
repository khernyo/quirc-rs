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

enum Union7 {
}

enum _IO_FILE {
}

enum jpeg_color_deconverter {
}

enum jpeg_color_quantizer {
}

enum jpeg_d_coef_controller {
}

enum jpeg_d_main_controller {
}

enum jpeg_d_post_controller {
}

enum jpeg_decomp_master {
}

enum jpeg_entropy_decoder {
}

enum jpeg_input_controller {
}

enum jpeg_inverse_dct {
}

enum jpeg_marker_reader {
}

enum jpeg_marker_struct {
}

enum jpeg_memory_mgr {
}

enum jpeg_progress_mgr {
}

enum jpeg_source_mgr {
}

enum jpeg_upsampler {
}

enum png_info_def {
}

enum png_struct_def {
}

enum quirc {
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_data {
    pub version : i32,
    pub ecc_level : i32,
    pub mask : i32,
    pub data_type : i32,
    pub payload : [u8; 8896],
    pub payload_len : i32,
    pub eci : u32,
}

impl Clone for quirc_data {
    fn clone(&self) -> Self { *self }
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
pub unsafe extern fn dump_data(mut data : *const quirc_data) {
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

#[derive(Copy)]
#[repr(C)]
pub struct quirc_point {
    pub x : i32,
    pub y : i32,
}

impl Clone for quirc_point {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct quirc_code {
    pub corners : [quirc_point; 4],
    pub size : i32,
    pub cell_bitmap : [u8; 3917],
}

impl Clone for quirc_code {
    fn clone(&self) -> Self { *self }
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

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum Enum1 {
    JCS_UNKNOWN,
    JCS_GRAYSCALE,
    JCS_RGB,
    JCS_YCbCr,
    JCS_CMYK,
    JCS_YCCK,
    JCS_EXT_RGB,
    JCS_EXT_RGBX,
    JCS_EXT_BGR,
    JCS_EXT_BGRX,
    JCS_EXT_XBGR,
    JCS_EXT_XRGB,
    JCS_EXT_RGBA,
    JCS_EXT_BGRA,
    JCS_EXT_ABGR,
    JCS_EXT_ARGB,
    JCS_RGB565,
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum Enum2 {
    JDCT_ISLOW,
    JDCT_IFAST,
    JDCT_FLOAT,
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum Enum3 {
    JDITHER_NONE,
    JDITHER_ORDERED,
    JDITHER_FS,
}

#[derive(Copy)]
#[repr(C)]
pub struct Struct4 {
    pub quantval : [u16; 64],
    pub sent_table : i32,
}

impl Clone for Struct4 {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct Struct5 {
    pub bits : [u8; 17],
    pub huffval : [u8; 256],
    pub sent_table : i32,
}

impl Clone for Struct5 {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct Struct6 {
    pub component_id : i32,
    pub component_index : i32,
    pub h_samp_factor : i32,
    pub v_samp_factor : i32,
    pub quant_tbl_no : i32,
    pub dc_tbl_no : i32,
    pub ac_tbl_no : i32,
    pub width_in_blocks : u32,
    pub height_in_blocks : u32,
    pub DCT_scaled_size : i32,
    pub downsampled_width : u32,
    pub downsampled_height : u32,
    pub component_needed : i32,
    pub MCU_width : i32,
    pub MCU_height : i32,
    pub MCU_blocks : i32,
    pub MCU_sample_width : i32,
    pub last_col_width : i32,
    pub last_row_height : i32,
    pub quant_table : *mut Struct4,
    pub dct_table : *mut ::std::os::raw::c_void,
}

impl Clone for Struct6 {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct jpeg_decompress_struct {
    pub err : *mut jpeg_error_mgr,
    pub mem : *mut jpeg_memory_mgr,
    pub progress : *mut jpeg_progress_mgr,
    pub client_data : *mut ::std::os::raw::c_void,
    pub is_decompressor : i32,
    pub global_state : i32,
    pub src : *mut jpeg_source_mgr,
    pub image_width : u32,
    pub image_height : u32,
    pub num_components : i32,
    pub jpeg_color_space : Enum1,
    pub out_color_space : Enum1,
    pub scale_num : u32,
    pub scale_denom : u32,
    pub output_gamma : f64,
    pub buffered_image : i32,
    pub raw_data_out : i32,
    pub dct_method : Enum2,
    pub do_fancy_upsampling : i32,
    pub do_block_smoothing : i32,
    pub quantize_colors : i32,
    pub dither_mode : Enum3,
    pub two_pass_quantize : i32,
    pub desired_number_of_colors : i32,
    pub enable_1pass_quant : i32,
    pub enable_external_quant : i32,
    pub enable_2pass_quant : i32,
    pub output_width : u32,
    pub output_height : u32,
    pub out_color_components : i32,
    pub output_components : i32,
    pub rec_outbuf_height : i32,
    pub actual_number_of_colors : i32,
    pub colormap : *mut *mut u8,
    pub output_scanline : u32,
    pub input_scan_number : i32,
    pub input_iMCU_row : u32,
    pub output_scan_number : i32,
    pub output_iMCU_row : u32,
    pub coef_bits : *mut [i32; 64],
    pub quant_tbl_ptrs : [*mut Struct4; 4],
    pub dc_huff_tbl_ptrs : [*mut Struct5; 4],
    pub ac_huff_tbl_ptrs : [*mut Struct5; 4],
    pub data_precision : i32,
    pub comp_info : *mut Struct6,
    pub progressive_mode : i32,
    pub arith_code : i32,
    pub arith_dc_L : [u8; 16],
    pub arith_dc_U : [u8; 16],
    pub arith_ac_K : [u8; 16],
    pub restart_interval : u32,
    pub saw_JFIF_marker : i32,
    pub JFIF_major_version : u8,
    pub JFIF_minor_version : u8,
    pub density_unit : u8,
    pub X_density : u16,
    pub Y_density : u16,
    pub saw_Adobe_marker : i32,
    pub Adobe_transform : u8,
    pub CCIR601_sampling : i32,
    pub marker_list : *mut jpeg_marker_struct,
    pub max_h_samp_factor : i32,
    pub max_v_samp_factor : i32,
    pub min_DCT_scaled_size : i32,
    pub total_iMCU_rows : u32,
    pub sample_range_limit : *mut u8,
    pub comps_in_scan : i32,
    pub cur_comp_info : [*mut Struct6; 4],
    pub MCUs_per_row : u32,
    pub MCU_rows_in_scan : u32,
    pub blocks_in_MCU : i32,
    pub MCU_membership : [i32; 10],
    pub Ss : i32,
    pub Se : i32,
    pub Ah : i32,
    pub Al : i32,
    pub unread_marker : i32,
    pub master : *mut jpeg_decomp_master,
    pub _c_main : *mut jpeg_d_main_controller,
    pub coef : *mut jpeg_d_coef_controller,
    pub post : *mut jpeg_d_post_controller,
    pub inputctl : *mut jpeg_input_controller,
    pub marker : *mut jpeg_marker_reader,
    pub entropy : *mut jpeg_entropy_decoder,
    pub idct : *mut jpeg_inverse_dct,
    pub upsample : *mut jpeg_upsampler,
    pub cconvert : *mut jpeg_color_deconverter,
    pub cquantize : *mut jpeg_color_quantizer,
}

impl Clone for jpeg_decompress_struct {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct jpeg_common_struct {
    pub err : *mut jpeg_error_mgr,
    pub mem : *mut jpeg_memory_mgr,
    pub progress : *mut jpeg_progress_mgr,
    pub client_data : *mut ::std::os::raw::c_void,
    pub is_decompressor : i32,
    pub global_state : i32,
}

impl Clone for jpeg_common_struct {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct jpeg_error_mgr {
    pub error_exit : unsafe extern fn(*mut jpeg_common_struct),
    pub emit_message : unsafe extern fn(*mut jpeg_common_struct, i32),
    pub output_message : unsafe extern fn(*mut jpeg_common_struct),
    pub format_message : unsafe extern fn(*mut jpeg_common_struct, *mut u8),
    pub reset_error_mgr : unsafe extern fn(*mut jpeg_common_struct),
    pub msg_code : i32,
    pub msg_parm : Union7,
    pub trace_level : i32,
    pub num_warnings : isize,
    pub jpeg_message_table : *const *const u8,
    pub last_jpeg_message : i32,
    pub addon_message_table : *const *const u8,
    pub first_addon_message : i32,
    pub last_addon_message : i32,
}

impl Clone for jpeg_error_mgr {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct Struct8 {
    pub __val : [usize; 16],
}

impl Clone for Struct8 {
    fn clone(&self) -> Self { *self }
}

#[derive(Copy)]
#[repr(C)]
pub struct __jmp_buf_tag {
    pub __jmpbuf : [isize; 8],
    pub __mask_was_saved : i32,
    pub __saved_mask : Struct8,
}

impl Clone for __jmp_buf_tag {
    fn clone(&self) -> Self { *self }
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
    let mut dinfo : jpeg_decompress_struct;
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
