pub mod cpu;
pub mod mem;
pub mod net;
pub mod machine;
pub mod os;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
