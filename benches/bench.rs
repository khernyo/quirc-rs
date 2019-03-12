#![feature(test)]

extern crate test;

use std::path::Path;

use libc::{c_void, memcpy};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

use quirc_wrapper as qw;
use test_utils::dbgutil::*;

unsafe fn run(width: u32, height: u32, image_bytes: &[u8]) {
    let mut decoder = Quirc::new();
    quirc_resize(&mut decoder, width as i32, height as i32);
    let quirc_image_bytes = quirc_begin(
        &mut decoder,
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
    );
    quirc_image_bytes.copy_from_slice(image_bytes);
    quirc_end(&mut decoder);

    let id_count = quirc_count(&decoder);
    for i in 0..id_count {
        let mut code: QuircCode = std::mem::uninitialized();
        let mut data: QuircData = std::mem::uninitialized();
        quirc_extract(&mut decoder, i, &mut code);
        quirc_decode(&mut code, &mut data);
    }
}

unsafe fn run_original(width: u32, height: u32, image_bytes: &[u8]) {
    let decoder: *mut qw::quirc = qw::quirc_new();
    qw::quirc_resize(decoder, width as i32, height as i32);
    let quirc_image_bytes = qw::quirc_begin(
        decoder,
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
        0i32 as (*mut ::std::os::raw::c_void) as (*mut i32),
    );
    memcpy(
        quirc_image_bytes as *mut c_void,
        image_bytes.as_ptr() as *mut c_void,
        (width * height) as usize,
    );
    qw::quirc_end(decoder);

    let id_count = qw::quirc_count(decoder as (*const qw::quirc));
    for i in 0..id_count {
        let mut code: qw::quirc_code = std::mem::uninitialized();
        let mut data: qw::quirc_data = std::mem::uninitialized();
        qw::quirc_extract(
            decoder as (*mut qw::quirc),
            i,
            &mut code as (*mut qw::quirc_code),
        );
        qw::quirc_decode(
            &mut code as (*mut qw::quirc_code) as (*const qw::quirc_code),
            &mut data as (*mut qw::quirc_data),
        );
    }
    qw::quirc_destroy(decoder);
}

unsafe fn bench(
    b: &mut test::Bencher,
    path: &Path,
    f: unsafe fn(width: u32, height: u32, image_bytes: &[u8]),
) {
    let (width, height, q) = {
        let mut decoder = Quirc::new();
        // TODO move quirc setup out of load_image()
        let ret = load_image(&mut decoder, path);
        assert_eq!(ret, 0);

        (decoder.w as u32, decoder.h as u32, decoder)
    };

    b.iter(|| f(width, height, &q.image));
}

macro_rules! bench {
    ($bench_name:ident, $bench_name_original:ident, $filename:expr) => {
        #[bench]
        fn $bench_name(b: &mut test::Bencher) {
            let path = Path::new("tests/images").join($filename);
            unsafe {
                bench(b, &path, run);
            }
        }

        #[bench]
        fn $bench_name_original(b: &mut test::Bencher) {
            let path = Path::new("tests/images").join($filename);
            unsafe {
                bench(b, &path, run_original);
            }
        }
    };
}

bench!(
    bench_image_01,
    bench_image_01_original,
    "20140626_QR-code_door_Ninette_Koning_Gelderse_Hout_Lelystad.jpg"
);
bench!(
    bench_image_02,
    bench_image_02_original,
    "20150618_Prospekt_Mira_39-41_02.jpg"
);
bench!(
    bench_image_03,
    bench_image_03_original,
    "2_150_150DPI_ty_oerny_08_2011.jpg"
);
bench!(
    bench_image_04,
    bench_image_04_original,
    "Cong_Cem_DC_old_Matlovich_QR.JPG"
);
bench!(
    bench_image_05,
    bench_image_05_original,
    "Cong_Cem_DC_Old_QR.JPG"
);
bench!(
    bench_image_06,
    bench_image_06_original,
    "Moe_Epsilon_QR_code.png"
);
bench!(
    bench_image_07,
    bench_image_07_original,
    "QR_CC_Chief_Taza.JPG"
);
bench!(
    bench_image_08,
    bench_image_08_original,
    "QRCode-1-Intro.png"
);
bench!(
    bench_image_09,
    bench_image_09_original,
    "QRCode-2-Structure.png"
);
bench!(
    bench_image_10,
    bench_image_10_original,
    "QR_code_Congressional_Cemetery.jpg"
);
bench!(
    bench_image_11,
    bench_image_11_original,
    "QR_Code_Damaged.jpg"
);
bench!(
    bench_image_12,
    bench_image_12_original,
    "Qr_code_details.png"
);
bench!(
    bench_image_13,
    bench_image_13_original,
    "QRcode_-_De_Verdieping_van_Nederland_(3).jpg"
);
bench!(
    bench_image_14,
    bench_image_14_original,
    "QRcode_-_De_Verdieping_van_Nederland_(4).JPG"
);
bench!(
    bench_image_15,
    bench_image_15_original,
    "QR_code_for_QRpedia.png"
);
bench!(
    bench_image_16,
    bench_image_16_original,
    "QR_Code,_Museum_für_Hamburgische_Geschichte_IMG_1607_original.jpg"
);
bench!(
    bench_image_17,
    bench_image_17_original,
    "QR-code-Open-research.png"
);
bench!(
    bench_image_18,
    bench_image_18_original,
    "QR-Code_so_nicht.jpg"
);
bench!(
    bench_image_19,
    bench_image_19_original,
    "Qr-code-ver-10.png"
);
bench!(
    bench_image_20,
    bench_image_20_original,
    "Qrcode_wikipedia_fr_v2clean.png"
);
bench!(
    bench_image_21,
    bench_image_21_original,
    "QRpedia_code_for_Ohrenqualle_at_Phyletisches_Museum_-_IMAG6096.jpg"
);
bench!(
    bench_image_22,
    bench_image_22_original,
    "QRpedia_Infotafel,_Bauhof_am_Deichtor,_MHG,_Hamburg,_Deutschland_IMG_5461_edit.jpg"
);
bench!(
    bench_image_23,
    bench_image_23_original,
    "Sk.wikipedia.org_QR_Code.png"
);
