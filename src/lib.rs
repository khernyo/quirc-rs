#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod decode;
pub mod identify;
pub mod quirc;
mod version_db;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
