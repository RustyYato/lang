use std::num::NonZeroU16;

pub fn gcd(mut a: u16, mut b: u16) -> u16 {
    while let Some(bnz) = NonZeroU16::new(b) {
        let (c, d) = (b, a % bnz);
        a = c;
        b = d;
    }
    a
}
