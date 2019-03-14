use std::path::Path;

use quirc_rs::decode::*;
use quirc_rs::identify::*;
use quirc_rs::quirc::*;

use test_utils::dbgutil::*;

#[derive(Debug, Eq, PartialEq)]
struct Data {
    version: i32,
    data_type: i32,
    ecc_level: i32,
    eci: u32,
    mask: i32,
    payload: String,
}

impl Data {
    fn new(
        version: i32,
        data_type: i32,
        ecc_level: i32,
        eci: u32,
        mask: i32,
        payload: String,
    ) -> Data {
        Data {
            version,
            data_type,
            ecc_level,
            eci,
            mask,
            payload,
        }
    }
}

unsafe fn validate_against_original(path: &Path, expected_contents: &[Option<Data>]) {
    let mut decoder = Quirc::new();
    let ret = load_image(&mut decoder, path);
    assert_eq!(ret, 0);

    let image_bytes = decoder.image.clone();

    quirc_end(&mut decoder);

    let result: Vec<_> = (0..quirc_count(&decoder))
        .map(|i| {
            let mut data: QuircData = std::mem::uninitialized();
            let mut code = quirc_extract(&mut decoder, i).unwrap();
            if quirc_decode(&mut code, &mut data).is_ok() {
                Some(Data::new(
                    data.version,
                    data.data_type,
                    data.ecc_level,
                    data.eci,
                    data.mask,
                    String::from_utf8(data.payload[0..data.payload_len as usize].to_vec()).unwrap(),
                ))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(result, expected_contents);

    validate(&mut decoder, &image_bytes);
}

macro_rules! check {
    ($test_name:ident, $filename:expr, $expected_content:expr) => {
        #[test]
        fn $test_name() {
            let path = Path::new("tests/images").join($filename);
            unsafe {
                validate_against_original(&path, $expected_content);
            }
        }
    };
}

// TODO Add images containing multiple QR codes
check!(
    test_image_1,
    "20140626_QR-code_door_Ninette_Koning_Gelderse_Hout_Lelystad.jpg",
    &[]
);
check!(test_image_2, "20150618_Prospekt_Mira_39-41_02.jpg", &[None]);
check!(test_image_3, "2_150_150DPI_ty_oerny_08_2011.jpg", &[None]);
check!(test_image_4, "Cong_Cem_DC_old_Matlovich_QR.JPG", &[]);
check!(test_image_5, "Cong_Cem_DC_Old_QR.JPG", &[]);
check!(
    test_image_6,
    "Moe_Epsilon_QR_code.png",
    &[Some(Data {
        version: 4,
        data_type: 4,
        ecc_level: 0,
        eci: 0,
        mask: 1,
        payload: "http://en.wikipedia.org/wiki/User:Moe_Epsilon".to_owned()
    })]
);
check!(test_image_7, "QR_CC_Chief_Taza.JPG", &[]);
check!(
    test_image_8,
    "QRCode-1-Intro.png",
    &[Some(Data {
        version: 3,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 7,
        payload: "Mr. Watson, come here - I want to see you.".to_owned()
    })]
);
check!(test_image_9, "QRCode-2-Structure.png", &[]);
check!(
    test_image_10,
    "QR_code_Congressional_Cemetery.jpg",
    &[Some(Data {
        version: 3,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 3,
        payload: "http://en.qrwp.org/Congressional_Cemetery".to_owned()
    })]
);
check!(
    test_image_11,
    "QR_Code_Damaged.jpg",
    &[Some(Data {
        version: 3,
        data_type: 4,
        ecc_level: 3,
        eci: 0,
        mask: 1,
        payload: "http://en.m.wikipedia.org".to_owned()
    })]
);
check!(test_image_12, "Qr_code_details.png", &[None]);
check!(
    test_image_13,
    "QRcode_-_De_Verdieping_van_Nederland_(3).jpg",
    &[Some(Data {
        version: 4,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 3,
        payload: "http://nl.wikipedia.org/wiki/Geschiedenis_van_de_Nederlandse_slavernij"
            .to_owned()
    })]
);
check!(
    test_image_14,
    "QRcode_-_De_Verdieping_van_Nederland_(4).JPG",
    &[Some(Data {
        version: 4,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 3,
        payload: "http://nl.wikipedia.org/wiki/Geschiedenis_van_de_Nederlandse_slavernij"
            .to_owned()
    })]
);
check!(
    test_image_15,
    "QR_code_for_QRpedia.png",
    &[Some(Data {
        version: 2,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 4,
        payload: "http://en.qrwp.org/QRpedia".to_owned()
    })]
);
check!(
    test_image_16,
    "QR_Code,_Museum_für_Hamburgische_Geschichte_IMG_1607_original.jpg",
    &[None]
);
check!(
    test_image_17,
    "QR-code-Open-research.png",
    &[Some(Data {
        version: 3,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 2,
        payload: "http://en.qrwp.org/Open_research".to_owned()
    })]
);
check!(test_image_18, "QR-Code_so_nicht.jpg", &[]);
check!(test_image_19, "Qr-code-ver-10.png", &[Some(Data { version: 10, data_type: 4, ecc_level: 2, eci: 0, mask: 4, payload: "VERSION 10 QR CODE, UP TO 174 CHAR AT H LEVEL, WITH 57X57 MODULES AND PLENTY OF ERROR CORRECTION TO GO AROUND.  NOTE THAT THERE ARE ADDITIONAL TRACKING BOXES".to_owned() })]);
check!(
    test_image_20,
    "Qrcode_wikipedia_fr_v2clean.png",
    &[Some(Data {
        version: 2,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 1,
        payload: "http://fr.wikipedia.org/".to_owned()
    })]
);
check!(
    test_image_21,
    "QRpedia_code_for_Ohrenqualle_at_Phyletisches_Museum_-_IMAG6096.jpg",
    &[]
);
check!(
    test_image_22,
    "QRpedia_Infotafel,_Bauhof_am_Deichtor,_MHG,_Hamburg,_Deutschland_IMG_5461_edit.jpg",
    &[]
);
check!(
    test_image_23,
    "Sk.wikipedia.org_QR_Code.png",
    &[Some(Data {
        version: 2,
        data_type: 4,
        ecc_level: 1,
        eci: 0,
        mask: 2,
        payload: "http://sk.wikipedia.org/".to_owned()
    })]
);
