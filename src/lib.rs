#![allow(non_snake_case)]

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
