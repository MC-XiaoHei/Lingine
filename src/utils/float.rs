pub trait FloatEx {
    fn is_not_nan(&self) -> bool;
}

impl FloatEx for f32 {
    #[inline(always)]
    fn is_not_nan(&self) -> bool {
        !self.is_nan()
    }
}

impl FloatEx for f64 {
    #[inline(always)]
    fn is_not_nan(&self) -> bool {
        !self.is_nan()
    }
}
