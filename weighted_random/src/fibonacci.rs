pub static FIBONACCI: [u8; 14] = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];

pub fn fibonacci(nth: usize) -> u8 {
    match nth >= FIBONACCI.len() {
        true => 233,
        false => FIBONACCI[nth]
    }
}