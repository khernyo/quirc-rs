use std::ffi::CStr;

use image;

use libc::{c_void};

unsafe extern fn data_type_str(mut dt : i32) -> &'static str {
    if dt == 8i32 {
        "KANJI"
    } else if dt == 4i32 {
        "BYTE"
    } else if dt == 2i32 {
        "ALPHA"
    } else if dt == 1i32 {
        "NUMERIC"
    } else {
        "unknown"
    }
}

#[no_mangle]
pub unsafe extern fn dump_data(mut data : *mut quirc_data) {
    println!("    Version: {}", (*data).version);
    println!(
        "    ECC level: {}",
        (*b"MLHQ\0")[(*data).ecc_level as (usize)] as (i32)
    );
    println!("    Mask: {}", (*data).mask);
    println!(
        "    Data type: {} ({})",
        (*data).data_type,
        data_type_str((*data).data_type)
    );
    println!("    Length: {}", (*data).payload_len);
    println!(
        "    Payload: {}",
        CStr::from_ptr((*data).payload.as_mut_ptr() as *mut c_char).to_str().unwrap()
    );
    if (*data).eci != 0 {
        println!("    ECI: {}", (*data).eci);
    }
}

#[no_mangle]
pub unsafe extern fn dump_cells(mut code : *const quirc_code) {
    let mut u : i32;
    let mut v : i32;
    print!("    {} cells, corners:", (*code).size);
    u = 0i32;
    'loop1: loop {
        if !(u < 4i32) {
            break;
        }
        print!(
            " ({},{})",
            (*code).corners[u as (usize)].x,
            (*code).corners[u as (usize)].y
        );
        u = u + 1;
    }
    println!();
    v = 0i32;
    'loop3: loop {
        if !(v < (*code).size) {
            break;
        }
        print!("    ");
        u = 0i32;
        'loop6: loop {
            if !(u < (*code).size) {
                break;
            }
            let mut p : i32 = v * (*code).size + u;
            if (*code).cell_bitmap[
                   (p >> 3i32) as (usize)
               ] as (i32) & 1i32 << (p & 7i32) != 0 {
                print!("[]");
            } else {
                print!("  ");
            }
            u = u + 1;
        }
        println!();
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
