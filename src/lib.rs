#![allow(unused)]

pub mod catchable;
pub mod spec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t() {
        let s = std::fs::read_to_string("data/compare.json").unwrap();
        let v: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();
        panic!("{v:#?}");
    }
}
