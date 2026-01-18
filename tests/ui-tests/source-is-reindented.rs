use assert2::assert;

pub fn main() {
    reproducible_panic::install();
    assert!(
        1 == 2 - 1
            && 3 == 4 - 1
        && 5 == 6
    );
}
