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

///! QR-code version information database

pub const QUIRC_MAX_VERSION: usize = 40;
pub const QUIRC_MAX_ALIGNMENT: usize = 7;

#[derive(Copy)]
#[repr(C)]
pub struct RsParams {
    /// Small block size
    pub bs: i32,

    /// Small data words
    pub dw: i32,

    /// Number of small blocks
    pub ns: i32,
}

impl Clone for RsParams {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct VersionInfo {
    pub data_bytes: i32,
    pub apat: [i32; QUIRC_MAX_ALIGNMENT],
    pub ecc: [RsParams; 4],
}

impl Clone for VersionInfo {
    fn clone(&self) -> Self {
        *self
    }
}

pub static mut VERSION_DB: [VersionInfo; QUIRC_MAX_VERSION + 1] = [
    VersionInfo {
        data_bytes: 0i32,
        apat: [0i32; 7],
        ecc: [RsParams {
            bs: 0,
            dw: 0,
            ns: 0,
        }; 4],
    },
    // Version 1
    VersionInfo {
        data_bytes: 26i32,
        apat: [0i32, 0i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 26i32,
                dw: 16i32,
                ns: 1i32,
            },
            RsParams {
                bs: 26i32,
                dw: 19i32,
                ns: 1i32,
            },
            RsParams {
                bs: 26i32,
                dw: 9i32,
                ns: 1i32,
            },
            RsParams {
                bs: 26i32,
                dw: 13i32,
                ns: 1i32,
            },
        ],
    },
    // Version 2
    VersionInfo {
        data_bytes: 44i32,
        apat: [6i32, 18i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 44i32,
                dw: 28i32,
                ns: 1i32,
            },
            RsParams {
                bs: 44i32,
                dw: 34i32,
                ns: 1i32,
            },
            RsParams {
                bs: 44i32,
                dw: 16i32,
                ns: 1i32,
            },
            RsParams {
                bs: 44i32,
                dw: 22i32,
                ns: 1i32,
            },
        ],
    },
    // Version 3
    VersionInfo {
        data_bytes: 70i32,
        apat: [6i32, 22i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 70i32,
                dw: 44i32,
                ns: 1i32,
            },
            RsParams {
                bs: 70i32,
                dw: 55i32,
                ns: 1i32,
            },
            RsParams {
                bs: 35i32,
                dw: 13i32,
                ns: 2i32,
            },
            RsParams {
                bs: 35i32,
                dw: 17i32,
                ns: 2i32,
            },
        ],
    },
    // Version 4
    VersionInfo {
        data_bytes: 100i32,
        apat: [6i32, 26i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 50i32,
                dw: 32i32,
                ns: 2i32,
            },
            RsParams {
                bs: 100i32,
                dw: 80i32,
                ns: 1i32,
            },
            RsParams {
                bs: 25i32,
                dw: 9i32,
                ns: 4i32,
            },
            RsParams {
                bs: 50i32,
                dw: 24i32,
                ns: 2i32,
            },
        ],
    },
    // Version 5
    VersionInfo {
        data_bytes: 134i32,
        apat: [6i32, 30i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 67i32,
                dw: 43i32,
                ns: 2i32,
            },
            RsParams {
                bs: 134i32,
                dw: 108i32,
                ns: 1i32,
            },
            RsParams {
                bs: 33i32,
                dw: 11i32,
                ns: 2i32,
            },
            RsParams {
                bs: 33i32,
                dw: 15i32,
                ns: 2i32,
            },
        ],
    },
    // Version 6
    VersionInfo {
        data_bytes: 172i32,
        apat: [6i32, 34i32, 0i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 43i32,
                dw: 27i32,
                ns: 4i32,
            },
            RsParams {
                bs: 86i32,
                dw: 68i32,
                ns: 2i32,
            },
            RsParams {
                bs: 43i32,
                dw: 15i32,
                ns: 4i32,
            },
            RsParams {
                bs: 43i32,
                dw: 19i32,
                ns: 4i32,
            },
        ],
    },
    // Version 7
    VersionInfo {
        data_bytes: 196i32,
        apat: [6i32, 22i32, 38i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 49i32,
                dw: 31i32,
                ns: 4i32,
            },
            RsParams {
                bs: 98i32,
                dw: 78i32,
                ns: 2i32,
            },
            RsParams {
                bs: 39i32,
                dw: 13i32,
                ns: 4i32,
            },
            RsParams {
                bs: 32i32,
                dw: 14i32,
                ns: 2i32,
            },
        ],
    },
    // Version 8
    VersionInfo {
        data_bytes: 242i32,
        apat: [6i32, 24i32, 42i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 60i32,
                dw: 38i32,
                ns: 2i32,
            },
            RsParams {
                bs: 121i32,
                dw: 97i32,
                ns: 2i32,
            },
            RsParams {
                bs: 40i32,
                dw: 14i32,
                ns: 4i32,
            },
            RsParams {
                bs: 40i32,
                dw: 18i32,
                ns: 4i32,
            },
        ],
    },
    // Version 9
    VersionInfo {
        data_bytes: 292i32,
        apat: [6i32, 26i32, 46i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 58i32,
                dw: 36i32,
                ns: 3i32,
            },
            RsParams {
                bs: 146i32,
                dw: 116i32,
                ns: 2i32,
            },
            RsParams {
                bs: 36i32,
                dw: 12i32,
                ns: 4i32,
            },
            RsParams {
                bs: 36i32,
                dw: 16i32,
                ns: 4i32,
            },
        ],
    },
    // Version 10
    VersionInfo {
        data_bytes: 346i32,
        apat: [6i32, 28i32, 50i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 69i32,
                dw: 43i32,
                ns: 4i32,
            },
            RsParams {
                bs: 86i32,
                dw: 68i32,
                ns: 2i32,
            },
            RsParams {
                bs: 43i32,
                dw: 15i32,
                ns: 6i32,
            },
            RsParams {
                bs: 43i32,
                dw: 19i32,
                ns: 6i32,
            },
        ],
    },
    // Version 11
    VersionInfo {
        data_bytes: 404i32,
        apat: [6i32, 30i32, 54i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 80i32,
                dw: 50i32,
                ns: 1i32,
            },
            RsParams {
                bs: 101i32,
                dw: 81i32,
                ns: 4i32,
            },
            RsParams {
                bs: 36i32,
                dw: 12i32,
                ns: 3i32,
            },
            RsParams {
                bs: 50i32,
                dw: 22i32,
                ns: 4i32,
            },
        ],
    },
    // Version 12
    VersionInfo {
        data_bytes: 466i32,
        apat: [6i32, 32i32, 58i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 58i32,
                dw: 36i32,
                ns: 6i32,
            },
            RsParams {
                bs: 116i32,
                dw: 92i32,
                ns: 2i32,
            },
            RsParams {
                bs: 42i32,
                dw: 14i32,
                ns: 7i32,
            },
            RsParams {
                bs: 46i32,
                dw: 20i32,
                ns: 4i32,
            },
        ],
    },
    // Version 13
    VersionInfo {
        data_bytes: 532i32,
        apat: [6i32, 34i32, 62i32, 0i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 59i32,
                dw: 37i32,
                ns: 8i32,
            },
            RsParams {
                bs: 133i32,
                dw: 107i32,
                ns: 4i32,
            },
            RsParams {
                bs: 33i32,
                dw: 11i32,
                ns: 12i32,
            },
            RsParams {
                bs: 44i32,
                dw: 20i32,
                ns: 8i32,
            },
        ],
    },
    // Version 14
    VersionInfo {
        data_bytes: 581i32,
        apat: [6i32, 26i32, 46i32, 66i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 64i32,
                dw: 40i32,
                ns: 4i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 3i32,
            },
            RsParams {
                bs: 36i32,
                dw: 12i32,
                ns: 11i32,
            },
            RsParams {
                bs: 36i32,
                dw: 16i32,
                ns: 11i32,
            },
        ],
    },
    // Version 15
    VersionInfo {
        data_bytes: 655i32,
        apat: [6i32, 26i32, 48i32, 70i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 65i32,
                dw: 41i32,
                ns: 5i32,
            },
            RsParams {
                bs: 109i32,
                dw: 87i32,
                ns: 5i32,
            },
            RsParams {
                bs: 36i32,
                dw: 12i32,
                ns: 11i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 5i32,
            },
        ],
    },
    // Version 16
    VersionInfo {
        data_bytes: 733i32,
        apat: [6i32, 26i32, 50i32, 74i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 73i32,
                dw: 45i32,
                ns: 7i32,
            },
            RsParams {
                bs: 122i32,
                dw: 98i32,
                ns: 5i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 3i32,
            },
            RsParams {
                bs: 43i32,
                dw: 19i32,
                ns: 15i32,
            },
        ],
    },
    // Version 17
    VersionInfo {
        data_bytes: 815i32,
        apat: [6i32, 30i32, 54i32, 78i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 10i32,
            },
            RsParams {
                bs: 135i32,
                dw: 107i32,
                ns: 1i32,
            },
            RsParams {
                bs: 42i32,
                dw: 14i32,
                ns: 2i32,
            },
            RsParams {
                bs: 50i32,
                dw: 22i32,
                ns: 1i32,
            },
        ],
    },
    // Version 18
    VersionInfo {
        data_bytes: 901i32,
        apat: [6i32, 30i32, 56i32, 82i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 69i32,
                dw: 43i32,
                ns: 9i32,
            },
            RsParams {
                bs: 150i32,
                dw: 120i32,
                ns: 5i32,
            },
            RsParams {
                bs: 42i32,
                dw: 14i32,
                ns: 2i32,
            },
            RsParams {
                bs: 50i32,
                dw: 22i32,
                ns: 17i32,
            },
        ],
    },
    // Version 19
    VersionInfo {
        data_bytes: 991i32,
        apat: [6i32, 30i32, 58i32, 86i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 70i32,
                dw: 44i32,
                ns: 3i32,
            },
            RsParams {
                bs: 141i32,
                dw: 113i32,
                ns: 3i32,
            },
            RsParams {
                bs: 39i32,
                dw: 13i32,
                ns: 9i32,
            },
            RsParams {
                bs: 47i32,
                dw: 21i32,
                ns: 17i32,
            },
        ],
    },
    // Version 20
    VersionInfo {
        data_bytes: 1085i32,
        apat: [6i32, 34i32, 62i32, 90i32, 0i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 67i32,
                dw: 41i32,
                ns: 3i32,
            },
            RsParams {
                bs: 135i32,
                dw: 107i32,
                ns: 3i32,
            },
            RsParams {
                bs: 43i32,
                dw: 15i32,
                ns: 15i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 15i32,
            },
        ],
    },
    // Version 21
    VersionInfo {
        data_bytes: 1156i32,
        apat: [6i32, 28i32, 50i32, 72i32, 92i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 68i32,
                dw: 42i32,
                ns: 17i32,
            },
            RsParams {
                bs: 144i32,
                dw: 116i32,
                ns: 4i32,
            },
            RsParams {
                bs: 46i32,
                dw: 16i32,
                ns: 19i32,
            },
            RsParams {
                bs: 50i32,
                dw: 22i32,
                ns: 17i32,
            },
        ],
    },
    // Version 22
    VersionInfo {
        data_bytes: 1258i32,
        apat: [6i32, 26i32, 50i32, 74i32, 98i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 17i32,
            },
            RsParams {
                bs: 139i32,
                dw: 111i32,
                ns: 2i32,
            },
            RsParams {
                bs: 37i32,
                dw: 13i32,
                ns: 34i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 7i32,
            },
        ],
    },
    // Version 23
    VersionInfo {
        data_bytes: 1364i32,
        apat: [6i32, 30i32, 54i32, 78i32, 102i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 4i32,
            },
            RsParams {
                bs: 151i32,
                dw: 121i32,
                ns: 4i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 16i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 11i32,
            },
        ],
    },
    // Version 24
    VersionInfo {
        data_bytes: 1474i32,
        apat: [6i32, 28i32, 54i32, 80i32, 106i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 73i32,
                dw: 45i32,
                ns: 6i32,
            },
            RsParams {
                bs: 147i32,
                dw: 117i32,
                ns: 6i32,
            },
            RsParams {
                bs: 46i32,
                dw: 16i32,
                ns: 30i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 11i32,
            },
        ],
    },
    // Version 25
    VersionInfo {
        data_bytes: 1588i32,
        apat: [6i32, 32i32, 58i32, 84i32, 110i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 8i32,
            },
            RsParams {
                bs: 132i32,
                dw: 106i32,
                ns: 8i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 22i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 7i32,
            },
        ],
    },
    // Version 26
    VersionInfo {
        data_bytes: 1706i32,
        apat: [6i32, 30i32, 58i32, 86i32, 114i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 19i32,
            },
            RsParams {
                bs: 142i32,
                dw: 114i32,
                ns: 10i32,
            },
            RsParams {
                bs: 46i32,
                dw: 16i32,
                ns: 33i32,
            },
            RsParams {
                bs: 50i32,
                dw: 22i32,
                ns: 28i32,
            },
        ],
    },
    // Version 27
    VersionInfo {
        data_bytes: 1828i32,
        apat: [6i32, 34i32, 62i32, 90i32, 118i32, 0i32, 0i32],
        ecc: [
            RsParams {
                bs: 73i32,
                dw: 45i32,
                ns: 22i32,
            },
            RsParams {
                bs: 152i32,
                dw: 122i32,
                ns: 8i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 12i32,
            },
            RsParams {
                bs: 53i32,
                dw: 23i32,
                ns: 8i32,
            },
        ],
    },
    // Version 28
    VersionInfo {
        data_bytes: 1921i32,
        apat: [6i32, 26i32, 50i32, 74i32, 98i32, 122i32, 0i32],
        ecc: [
            RsParams {
                bs: 73i32,
                dw: 45i32,
                ns: 3i32,
            },
            RsParams {
                bs: 147i32,
                dw: 117i32,
                ns: 3i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 11i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 4i32,
            },
        ],
    },
    // Version 29
    VersionInfo {
        data_bytes: 2051i32,
        apat: [6i32, 30i32, 54i32, 78i32, 102i32, 126i32, 0i32],
        ecc: [
            RsParams {
                bs: 73i32,
                dw: 45i32,
                ns: 21i32,
            },
            RsParams {
                bs: 146i32,
                dw: 116i32,
                ns: 7i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 19i32,
            },
            RsParams {
                bs: 53i32,
                dw: 23i32,
                ns: 1i32,
            },
        ],
    },
    // Version 30
    VersionInfo {
        data_bytes: 2185i32,
        apat: [6i32, 26i32, 52i32, 78i32, 104i32, 130i32, 0i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 19i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 5i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 23i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 15i32,
            },
        ],
    },
    // Version 31
    VersionInfo {
        data_bytes: 2323i32,
        apat: [6i32, 30i32, 56i32, 82i32, 108i32, 134i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 2i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 13i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 23i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 42i32,
            },
        ],
    },
    // Version 32
    VersionInfo {
        data_bytes: 2465i32,
        apat: [6i32, 34i32, 60i32, 86i32, 112i32, 138i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 10i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 17i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 19i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 10i32,
            },
        ],
    },
    // Version 33
    VersionInfo {
        data_bytes: 2611i32,
        apat: [6i32, 30i32, 58i32, 86i32, 114i32, 142i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 14i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 17i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 11i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 29i32,
            },
        ],
    },
    // Version 34
    VersionInfo {
        data_bytes: 2761i32,
        apat: [6i32, 34i32, 62i32, 90i32, 118i32, 146i32, 0i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 14i32,
            },
            RsParams {
                bs: 145i32,
                dw: 115i32,
                ns: 13i32,
            },
            RsParams {
                bs: 46i32,
                dw: 16i32,
                ns: 59i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 44i32,
            },
        ],
    },
    // Version 35
    VersionInfo {
        data_bytes: 2876i32,
        apat: [6i32, 30i32, 54i32, 78i32, 102i32, 126i32, 150i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 12i32,
            },
            RsParams {
                bs: 151i32,
                dw: 121i32,
                ns: 12i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 22i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 39i32,
            },
        ],
    },
    // Version 36
    VersionInfo {
        data_bytes: 3034i32,
        apat: [6i32, 24i32, 50i32, 76i32, 102i32, 128i32, 154i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 6i32,
            },
            RsParams {
                bs: 151i32,
                dw: 121i32,
                ns: 6i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 2i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 46i32,
            },
        ],
    },
    // Version 37
    VersionInfo {
        data_bytes: 3196i32,
        apat: [6i32, 28i32, 54i32, 80i32, 106i32, 132i32, 158i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 29i32,
            },
            RsParams {
                bs: 152i32,
                dw: 122i32,
                ns: 17i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 24i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 49i32,
            },
        ],
    },
    // Version 38
    VersionInfo {
        data_bytes: 3362i32,
        apat: [6i32, 32i32, 58i32, 84i32, 110i32, 136i32, 162i32],
        ecc: [
            RsParams {
                bs: 74i32,
                dw: 46i32,
                ns: 13i32,
            },
            RsParams {
                bs: 152i32,
                dw: 122i32,
                ns: 4i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 42i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 48i32,
            },
        ],
    },
    // Version 39
    VersionInfo {
        data_bytes: 3532i32,
        apat: [6i32, 26i32, 54i32, 82i32, 110i32, 138i32, 166i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 40i32,
            },
            RsParams {
                bs: 147i32,
                dw: 117i32,
                ns: 20i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 10i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 43i32,
            },
        ],
    },
    // Version 40
    VersionInfo {
        data_bytes: 3706i32,
        apat: [6i32, 30i32, 58i32, 86i32, 114i32, 142i32, 170i32],
        ecc: [
            RsParams {
                bs: 75i32,
                dw: 47i32,
                ns: 18i32,
            },
            RsParams {
                bs: 148i32,
                dw: 118i32,
                ns: 19i32,
            },
            RsParams {
                bs: 45i32,
                dw: 15i32,
                ns: 20i32,
            },
            RsParams {
                bs: 54i32,
                dw: 24i32,
                ns: 34i32,
            },
        ],
    },
];
