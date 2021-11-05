// Name: FP32WithFlag - How to put 1 float32 and a flag inside a float32?
//
// Description: This Rust type struct, allows you to put a f32 and a boolean flag,
//              1 bit, inside a normal 32 bits float. It allows every type of f32
//              number except NAN. It uses the least significant bit of the precision
//              part of the f32 number as a bit flag.
//              The internal representation  of the number, passes from 23 bit
//              precision [b22...b0] to 22 bit precision [b22..b1].
//              The structure of a IEEE-754 Single precision floating point number is:
//
//              [b31...........b0]
//              bit 31        – Sign 1 bit
//              bit [30 a 23] – Exponent 8 bits.
//              bit [22 a 0]  – Fraction 23 bits.      
//
//              Wikipedia – IEEE-754 - Single-precision floating-point format
//              https://en.wikipedia.org/wiki/Single-precision_floating-point_format
//
//              The idea is to use this the lowest precision bit so that we at max we loose
//              one step of precision in the case of numbers that are terminated b0 in a
//              bit with the value 1 (ex: 3.3 f32), and that don't loose precision for
//              values that are terminated b0 in a  bit with the value 0 (zero), ex: 10.0
//              or 2.0 .
//              The use case of this type FP32WithFlag, is to store data in memory (so that
//              you can better use the different cache hierarchies size and you can lower
//              your needs of bandwidth between huge volumes of f32 in memory to the CPU).
//              All the intermediary calculations are made in IEEE fp32 and continue to be
//              accelerated by the CPU hardware, because when you get the get_val() it will
//              return a normal f32 value, and when you set the set_val() it will receive
//              a parameter with a normal f32 number.
//              you will also have for each number a get_flag() and a set_flag().
//              The only limitation I know of, is that you can't store NAN inside this
//              representation, because there are several NAN that are possible and I opted
//              to not test all bit patterns and correct them when you do an get_val().
//              This would imply, that I would have to reserve some precision bit's
//              in combination with a lowest exponent to store that values, that aren't
//              terminated in a NAN with the b0 bit to 0 (zero).
//              My implementation is independent of the endian (little endian vs big endian)
//              of the CPU, but I internally have chosen little endian to optimize for the
//              performance in the case of x86_64.
//
// Challenge: Thank you KammutierSpule, for giving me this challenge, is a cool problem that
//            I never thought about before.
//
// Date: 2021.11.05
//
// License: MIT Open Source license.
//


const SMALL_BYTE: usize = 0;

#[derive(Debug, Copy, Clone)]
pub struct FP32WithFlag {
    // Independent of machine, little endian representation of the float. 
    num_ar: [u8; 4],
}

impl FP32WithFlag {
    pub fn new(val: f32, flag: bool) -> Self {
        assert!(!val.is_nan());
        // Independent of machine, little endian representation of the float. 
        let mut new_val = val.to_le_bytes();
        if flag {
            new_val[SMALL_BYTE] = FP32WithFlag::set_bit(new_val[SMALL_BYTE], 0);
        } else {
            new_val[SMALL_BYTE] = FP32WithFlag::clear_bit(new_val[SMALL_BYTE], 0);
        }
        FP32WithFlag {
            // Put flag in bit b0.
            num_ar: new_val,
        }
    }

    pub fn get_val(& self) -> f32 {
        let mut new_num = self.num_ar.clone();
        // TODO: Here I have to put the conditions for +-0 and +-infinity.
        //       R: It already passes all the conditions.
        new_num[SMALL_BYTE] = FP32WithFlag::clear_bit(new_num[SMALL_BYTE], 0);
        f32::from_le_bytes(new_num)
    }

    pub fn set_val(& mut self, val: f32) -> Result<(), String> {
        if val.is_nan() {
            return Err("Error: FP32WithFlag.set_val() - val is NAN!".to_string());
        }
        // Independent of machine, little indian representation of the float. 
        let mut new_val = val.to_le_bytes();
        if FP32WithFlag::check_bit(self.num_ar[SMALL_BYTE], 0) == 1 {
            new_val[SMALL_BYTE] = FP32WithFlag::set_bit(new_val[SMALL_BYTE], 0);
        } else {
            new_val[SMALL_BYTE] = FP32WithFlag::clear_bit(new_val[SMALL_BYTE], 0);
        }        
        self.num_ar = new_val;
        Ok(())
    }

    pub fn get_flag(& self) -> bool {
        if FP32WithFlag::check_bit(self.num_ar[SMALL_BYTE], 0) == 1 {
            true
        } else {
            false
        }        
    }

    pub fn set_flag(& mut self, flag: bool) {
        if flag {
            self.num_ar[SMALL_BYTE] = FP32WithFlag::set_bit(self.num_ar[SMALL_BYTE], 0);
        } else {
            self.num_ar[SMALL_BYTE] = FP32WithFlag::clear_bit(self.num_ar[SMALL_BYTE], 0);
        }        
    }

    #[inline(always)]
    fn set_bit(byte: u8, n_bit: u8) -> u8 {
        byte | ((1 as u8) << n_bit) 
    }

    #[inline(always)]
    fn clear_bit(byte: u8, n_bit: u8) -> u8 {
        byte & !( (1 as u8) << n_bit)
    }
 
    // Return 0 or 1.
    #[inline(always)]
    fn check_bit(byte: u8, n_bit: u8) -> u8 {
        (byte >> n_bit) & 1
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bit() {
        // Zero mask set.
        assert_eq!(FP32WithFlag::set_bit(0_u8, 0), 0x01);
        assert_eq!(FP32WithFlag::set_bit(0_u8, 1), 0x02);
        assert_eq!(FP32WithFlag::set_bit(0_u8, 2), 0x04);

        // 0xF0 mask set.
        assert_eq!(FP32WithFlag::set_bit(0xF0_u8, 0), 0xF1);
        assert_eq!(FP32WithFlag::set_bit(0xF0_u8, 1), 0xF2);
        assert_eq!(FP32WithFlag::set_bit(0xF0_u8, 2), 0xF4);
    }

    #[test]
    fn test_check_bit() {
        // Zero mask get.
        assert_eq!(FP32WithFlag::check_bit(0x00_u8, 0), 0x00);
        assert_eq!(FP32WithFlag::check_bit(0x00_u8, 1), 0x00);
        assert_eq!(FP32WithFlag::check_bit(0x00_u8, 2), 0x00);

        // Prepared mask get.
        assert_eq!(FP32WithFlag::check_bit(0x01_u8, 0), 0x01);
        assert_eq!(FP32WithFlag::check_bit(0x02_u8, 1), 0x01);
        assert_eq!(FP32WithFlag::check_bit(0x04_u8, 2), 0x01);
    }

    #[test]
    fn test_clear_bit() {
        // Zero mask clear.
        assert_eq!(FP32WithFlag::clear_bit(0_u8, 0), 0x00);
        
        // Prepared mask clear.
        assert_eq!(FP32WithFlag::clear_bit(0x01_u8, 0), 0x00);
        assert_eq!(FP32WithFlag::clear_bit(0x02_u8, 1), 0x00);
        assert_eq!(FP32WithFlag::clear_bit(0x04_u8, 2), 0x00);

        // 0xF0 mask set.
        assert_eq!(FP32WithFlag::clear_bit(0xF0_u8, 0), 0xF0);
        assert_eq!(FP32WithFlag::clear_bit(0xF0_u8, 1), 0xF0);
        assert_eq!(FP32WithFlag::clear_bit(0xF0_u8, 2), 0xF0);
    }

    #[test]
    fn test_test_set_get_flag() {
        let mut fp1_m = FP32WithFlag::new(10.0, true);
        let     fp2_m = FP32WithFlag::new(2.0, false);
        assert_eq!(fp1_m.get_flag(), true);
        assert_eq!(fp2_m.get_flag(), false);
        fp1_m.set_flag(false);
        assert_eq!(fp1_m.get_flag(), false);
        fp1_m.set_flag(true);
        assert_eq!(fp1_m.get_flag(), true);
    }

    #[test]
    fn test_test_set_get_val() {
        // 10.0_f32 is an exact number in the last bit has zero.
        let mut fp1_m = FP32WithFlag::new(10.0, true);
        // 2.0_f32 is an exact number in the last bit has zero.
        let     fp2_m = FP32WithFlag::new(2.0, false);
        // 3.3_f32 is not exact number in the last bit has one.
        // So it will not give the same number.
        let     fp3_m = FP32WithFlag::new(3.3, false);
        assert_eq!(fp1_m.get_val(), 10.0_f32);
        assert_eq!(fp2_m.get_val(),  2.0_f32);
        assert_ne!(fp3_m.get_val(),  3.3_f32);
        fp1_m.set_val(2.0);
        assert_eq!(fp1_m.get_val(),  2.0_f32);
        assert_eq!(fp1_m.get_flag(), true);
        fp1_m.set_flag(false);
        assert_eq!(fp1_m.get_val(),  2.0_f32);
        assert_eq!(fp1_m.get_flag(), false);
    }

    #[test]
    fn test_test_neg_zero() {
        let fp1_m = FP32WithFlag::new(-0.0, false);
        assert_eq!(fp1_m.get_val(), -0.0_f32);
        let fp1_m = FP32WithFlag::new(-0.0, true);
        assert_eq!(fp1_m.get_val(), -0.0_f32);

        let mut fp1_m = FP32WithFlag::new(10.0, false);
        fp1_m.set_val(-0.0);
        assert_eq!(fp1_m.get_val(), -0.0_f32);
        fp1_m.set_flag(true);
        assert_eq!(fp1_m.get_val(), -0.0_f32);
        assert_eq!(fp1_m.get_flag(), true);
    }

    #[test]
    fn test_test_pos_zero() {
        let fp1_m = FP32WithFlag::new(0.0, false);
        assert_eq!(fp1_m.get_val(), 0.0_f32);
        let fp1_m = FP32WithFlag::new(0.0, true);
        assert_eq!(fp1_m.get_val(), 0.0_f32);

        let mut fp1_m = FP32WithFlag::new(10.0, false);
        fp1_m.set_val(0.0);
        assert_eq!(fp1_m.get_val(), 0.0_f32);
        fp1_m.set_flag(true);
        assert_eq!(fp1_m.get_val(), 0.0_f32);
        assert_eq!(fp1_m.get_flag(), true);
    }

    #[test]
    fn test_test_neg_infnity() {
        let fp1_m = FP32WithFlag::new(f32::NEG_INFINITY, false);
        assert_eq!(fp1_m.get_val(), f32::NEG_INFINITY);
        let fp1_m = FP32WithFlag::new( f32::NEG_INFINITY, true);
        assert_eq!(fp1_m.get_val(), f32::NEG_INFINITY);

        let mut fp1_m = FP32WithFlag::new(10.0, false);
        fp1_m.set_val(f32::NEG_INFINITY);
        assert_eq!(fp1_m.get_val(), f32::NEG_INFINITY);
        fp1_m.set_flag(true);
        assert_eq!(fp1_m.get_val(), f32::NEG_INFINITY);
        assert_eq!(fp1_m.get_flag(), true);
    }

    #[test]
    fn test_test_pos_infnity() {
        let fp1_m = FP32WithFlag::new( f32::INFINITY, false);
        assert_eq!(fp1_m.get_val(), f32::INFINITY);
        let fp1_m = FP32WithFlag::new( f32::INFINITY, true);
        assert_eq!(fp1_m.get_val(), f32::INFINITY);

        let mut fp1_m = FP32WithFlag::new(10.0, false);
        fp1_m.set_val(f32::INFINITY);
        assert_eq!(fp1_m.get_val(), f32::INFINITY);
        fp1_m.set_flag(true);
        assert_eq!(fp1_m.get_val(), f32::INFINITY);
        assert_eq!(fp1_m.get_flag(), true);
    }

    #[test]
    #[should_panic]
    fn test_test_nan_create_struct() {
        // f32::NAN
        // NOTE: There are more then one NAN.
        let fp1_m = FP32WithFlag::new( f32::NAN, false);
    }

    #[test]
    fn test_test_nan_set_val() {
        // f32::NAN
        // NOTE: There are more then one NAN.
        let mut fp1_m = FP32WithFlag::new(10.0, false);
        let res = fp1_m.set_val(f32::NAN);
        assert!(res.is_err());
        assert_ne!(fp1_m.get_val().to_string(), f32::NAN.to_string());
    }

    // #[test]
    // fn test_test_nan() {
    //     // f32::NAN
    //     // NOTE: There are more then one NAN.
    //     let fp1_m = FP32WithFlag::new( f32::NAN, false);
    //     // NOTE: NAN in Rust can't be directly compared for equality.
    //     assert_eq!(fp1_m.get_val().to_string(), f32::NAN.to_string());
    //     let fp1_m = FP32WithFlag::new( f32::NAN, true);
    //     assert_eq!(fp1_m.get_val().to_string(), f32::INFINITY.to_string());

    //     let mut fp1_m = FP32WithFlag::new(10.0, false);
    //     fp1_m.set_val(f32::NAN);
    //     assert_eq!(fp1_m.get_val().to_string(), f32::NAN.to_string());
    //     fp1_m.set_flag(true);
    //     assert_eq!(fp1_m.get_val().to_string(), f32::NAN.to_string());
    //     assert_eq!(fp1_m.get_flag(), true);
    // }
    

}

