pub mod cpu;
pub mod mem;
pub mod net;
pub mod proc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
