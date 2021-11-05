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


mod fp32_with_flag;

use fp32_with_flag::FP32WithFlag;

fn main() {
    println!("**********************");
    println!("**  FP32 with flag  **");
    println!("**********************");

    let fp1_m = FP32WithFlag::new(10.0, true);
    let fp2_m = FP32WithFlag::new(2.0, false);
    let fp3_m = FP32WithFlag::new(3.3, true);
    
    println!("sizes(fp1_m) = {} bytes.", std::mem::size_of::<FP32WithFlag>());

    println!( "fp1_m({:.10}, true) = ({:.10}, {}) => dif: {:.10}", 10.0_f32, fp1_m.get_val(), fp1_m.get_flag(), (10.0_f32 - fp1_m.get_val()).abs());
    println!( "fp2_m({:.10}, false) = ({:.10}, {}) => dif: {:.10}", 2_f32, fp2_m.get_val(), fp2_m.get_flag(), (2.0_f32 - fp2_m.get_val()).abs());
    println!( "fp3_m({:.10}, false) = ({:.10}, {}) => dif: {:.10}", 3.3_f32, fp3_m.get_val(), fp3_m.get_flag(), (3.3_f32 - fp3_m.get_val()).abs());
    
    let fp4 = fp1_m.get_val() * fp2_m.get_val();
    let flag: bool = fp1_m.get_flag();
    if flag {
        println!( "fp_m: 10.0 * 2.0 = {}", fp4);
        println!( "fp32: 10.0 * 2.0 = {}", 10.0_f32 * 2.0_f32);
    }
    
    // Creation of the array with 100 modified fp32.
    let mut vec_tmp = [FP32WithFlag::new(7.0, false); 100];
    
    // Mark the flag of the fp32 elements that we want to be used in the calculation.
    for i in 0..vec_tmp.len() {
        if i % 2 == 0 {
            vec_tmp[i].set_flag(true);
        }
    }
    
    // Conditionally execute the calculation for each element.
    let mut accu: f32 = 0.0;
    for elm in & vec_tmp {
        if elm.get_flag() {
           accu += elm.get_val() * 2.0 + 128.0;
        }
    }
    println!("{}", accu);

}