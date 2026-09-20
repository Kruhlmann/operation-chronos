use fixed::types::I32F32;

pub type Scalar = I32F32;
pub const SCALAR_EPSILON: Scalar = Scalar::from_bits(16);

#[inline]
pub fn scalar_round_to_i32(v: Scalar) -> i32 {
    let half = Scalar::const_from_int(1).strict_div_int(2);
    let biased = if v >= Scalar::ZERO {
        v + half
    } else {
        v - half
    };
    biased.int().to_num::<i32>()
}

#[inline]
pub const fn scalar_from_int(value: i32) -> Scalar {
    Scalar::from_bits((value as i64) << 32)
}

#[inline]
pub fn scalar_from_render(value: f32) -> Scalar {
    Scalar::from_num(value)
}

#[inline]
pub fn scalar_to_render(value: Scalar) -> f32 {
    value.to_num::<f32>()
}

pub fn scalar_sqrt(value: Scalar) -> Scalar {
    if value <= Scalar::ZERO {
        return Scalar::ZERO;
    }
    let widened: i128 = (value.to_bits() as i128) << 32;
    let root = isqrt_i128(widened) as i64;
    Scalar::from_bits(root)
}

fn isqrt_i128(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let bits = 128 - value.leading_zeros() as i128;
    let mut estimate = 1i128 << ((bits + 1) / 2);
    loop {
        let next = (estimate + value / estimate) / 2;
        if next >= estimate {
            return estimate;
        }
        estimate = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_of_perfect_squares() {
        assert_eq!(scalar_sqrt(scalar_from_int(0)), scalar_from_int(0));
        assert_eq!(scalar_sqrt(scalar_from_int(1)), scalar_from_int(1));
        assert_eq!(scalar_sqrt(scalar_from_int(4)), scalar_from_int(2));
        assert_eq!(scalar_sqrt(scalar_from_int(144)), scalar_from_int(12));
    }

    #[test]
    fn sqrt_of_non_perfect_square_is_close() {
        let two = scalar_from_int(2);
        let root = scalar_sqrt(two);
        // sqrt(2) ~= 1.41421. Allow a couple of Q16.16 LSBs of error.
        let expected = Scalar::from_num(std::f64::consts::SQRT_2);
        let diff = (root - expected).abs();
        assert!(diff <= Scalar::from_bits(2), "diff={diff}");
    }

    #[test]
    fn sqrt_of_negative_is_zero() {
        assert_eq!(scalar_sqrt(scalar_from_int(-4)), scalar_from_int(0));
    }
}
