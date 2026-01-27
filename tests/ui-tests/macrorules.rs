fn main() {
	reproducible_panic::install();
	macro_rules! assert_eq {
		($left:expr, $right:expr) => {
			::assert2::assert!($left + $left * $right == $right)
		};
	}
	assert_eq!(1 + 2, 2 + 3);
}
