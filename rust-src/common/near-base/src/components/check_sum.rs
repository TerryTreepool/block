
#[deny(arithmetic_overflow)]
pub trait CheckSum: AsRef<[u8]> {
    fn check_sum(&self) -> u16 {
        let mut sum = 0u16;

        self.as_ref()
            .iter()
            .for_each(| c | {
                sum += *c as u16;
                sum = sum.checked_shr(16).unwrap_or(0) + (sum & 0xffff);
                sum = sum + sum.checked_shr(16).unwrap_or(0);
            });

        return !sum;
    }
}
