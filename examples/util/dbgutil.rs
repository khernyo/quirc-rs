use image;

use libc::{c_void, fprintf, STDERR_FILENO};

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
    printf((*b"    Version: %d\n\0").as_ptr() as *const c_char,(*data).version);
    printf(
        (*b"    ECC level: %c\n\0").as_ptr() as *const c_char,
        (*b"MLHQ\0")[(*data).ecc_level as (usize)] as (i32)
    );
    printf((*b"    Mask: %d\n\0").as_ptr() as *const c_char,(*data).mask);
    printf(
        (*b"    Data type: %d (%s)\n\0").as_ptr() as *const c_char,
        (*data).data_type,
        data_type_str((*data).data_type)
    );
    printf((*b"    Length: %d\n\0").as_ptr() as *const c_char,(*data).payload_len);
    printf(
        (*b"    Payload: %s\n\0").as_ptr() as *const c_char,
        (*data).payload.as_mut_ptr()
    );
    if (*data).eci != 0 {
        printf((*b"    ECI: %d\n\0").as_ptr() as *const c_char,(*data).eci);
    }
}

#[no_mangle]
pub unsafe extern fn dump_cells(mut code : *const quirc_code) {
    let mut u : i32;
    let mut v : i32;
    printf((*b"    %d cells, corners:\0").as_ptr() as *const c_char,(*code).size);
    u = 0i32;
    'loop1: loop {
        if !(u < 4i32) {
            break;
        }
        printf(
            (*b" (%d,%d)\0").as_ptr() as *const c_char,
            (*code).corners[u as (usize)].x,
            (*code).corners[u as (usize)].y
        );
        u = u + 1;
    }
    printf((*b"\n\0").as_ptr() as *const c_char);
    v = 0i32;
    'loop3: loop {
        if !(v < (*code).size) {
            break;
        }
        printf((*b"    \0").as_ptr() as *const c_char);
        u = 0i32;
        'loop6: loop {
            if !(u < (*code).size) {
                break;
            }
            let mut p : i32 = v * (*code).size + u;
            if (*code).cell_bitmap[
                   (p >> 3i32) as (usize)
               ] as (i32) & 1i32 << (p & 7i32) != 0 {
                printf((*b"[]\0").as_ptr() as *const c_char);
            } else {
                printf((*b"  \0").as_ptr() as *const c_char);
            }
            u = u + 1;
        }
        printf((*b"\n\0").as_ptr() as *const c_char);
        v = v + 1;
    }
}

pub unsafe fn load_image(q: *mut quirc, path: &Path) -> i32 {
    use image::{self, ColorType};
    let img = image::open(path).unwrap().grayscale().to_luma();
    let (width, height) = img.dimensions();

    if !(quirc_resize(q, width as i32, height as i32) < 0i32) {
        let image_bytes = quirc_begin(
            q,
            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
            0i32 as (*mut ::std::os::raw::c_void) as (*mut i32)
        );

        let img_bytes = img.into_raw();
        assert_eq!(img_bytes.len(), width as usize * height as usize);
        libc::memcpy(image_bytes as *mut c_void, img_bytes.as_ptr() as *mut c_void, img_bytes.len());

        return 0i32;
    }
    -1i32
}
