use std::ffi::CStr;
use std::path::Path;

use libc::{c_char, c_void, malloc, memcmp, memcpy};

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

include!("util/dbgutil.rs");

unsafe fn validate_against_original(path: &Path) {
    let mut decoder : *mut quirc = quirc_new();
    let ret = load_image(decoder, path);
    assert_eq!(ret, 0);

    let image_bytes = {
        let dst = malloc(((*decoder).w * (*decoder).h) as usize);
        memcpy(dst, (*decoder).image as *const c_void, ((*decoder).w * (*decoder).h) as usize);
        dst
    };

    quirc_end(decoder);

    let mut info : result_info = std::mem::uninitialized();
    info.id_count = quirc_count(decoder as (*const quirc));
    for i in 0..info.id_count {
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
            info.decode_count = info.decode_count + 1;
        }
    }

    validate(decoder, path, &mut info, image_bytes);
}

macro_rules! check {
    ($test_name:ident, $filename:expr) => {
        #[test]
        fn $test_name() {
            let path = Path::new("tests/images").join($filename);
            unsafe {
                validate_against_original(&path);
            }
        }
    };
}

check!(test_image_1, "20140626_QR-code_door_Ninette_Koning_Gelderse_Hout_Lelystad.jpg");
check!(test_image_2, "20150618_Prospekt_Mira_39-41_02.jpg");
check!(test_image_3, "2_150_150DPI_ty_oerny_08_2011.jpg");
check!(test_image_4, "Cong_Cem_DC_old_Matlovich_QR.JPG");
check!(test_image_5, "Cong_Cem_DC_Old_QR.JPG");
check!(test_image_6, "Moe_Epsilon_QR_code.png");
check!(test_image_7, "QR_CC_Chief_Taza.JPG");
check!(test_image_8, "QRCode-1-Intro.png");
check!(test_image_9, "QRCode-2-Structure.png");
check!(test_image_10, "QR_code_Congressional_Cemetery.jpg");
check!(test_image_11, "QR_Code_Damaged.jpg");
check!(test_image_12, "Qr_code_details.png");
check!(test_image_13, "QRcode_-_De_Verdieping_van_Nederland_(3).jpg");
check!(test_image_14, "QRcode_-_De_Verdieping_van_Nederland_(4).JPG");
check!(test_image_15, "QR_code_for_QRpedia.png");
check!(test_image_16, "QR_Code,_Museum_für_Hamburgische_Geschichte_IMG_1607_original.jpg");
check!(test_image_17, "QR-code-Open-research.png");
check!(test_image_18, "QR-Code_so_nicht.jpg");
check!(test_image_19, "Qr-code-ver-10.png");
check!(test_image_20, "Qrcode_wikipedia_fr_v2clean.png");
check!(test_image_21, "QRpedia_code_for_Ohrenqualle_at_Phyletisches_Museum_-_IMAG6096.jpg");
check!(test_image_22, "QRpedia_Infotafel,_Bauhof_am_Deichtor,_MHG,_Hamburg,_Deutschland_IMG_5461_edit.jpg");
check!(test_image_23, "Sk.wikipedia.org_QR_Code.png");
