fn main() {
    loop {
        let s = get_input();
        let s = s.trim();

        let num = make_u8(s);
        println!("{:?}\n", num);
    }
}

#[inline(never)]
fn get_input() -> String {
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("failed to read line");
    buf
}

use std::num::Wrapping;

fn make_u8(s: &str) -> Option<u8> {
    if s.is_empty() || s.len() > 3 {
        return None;
    }
    let bytes = s.as_bytes();

    // using a union avoids branching on the length to initialize each byte
    // of the u32 interpretation. not sure it's better as it makes call to
    // memcpy. could let caller deal with it...
    let mut working = unsafe {
        #[repr(C)]
        union U {
            bytes: [u8; 4],
            num: u32,
        }
        // could use uninit here to avoid initialization...
        let mut u = U { num: 0 };
        u.bytes[..s.len()].copy_from_slice(&bytes[..s.len()]);
        u.num
    };

    working ^= 0x30303030;

    working <<= (4 - s.len()) * 8;

    // Wrapping prevents panics on overflow.
    let mult = Wrapping(0x640a01) * Wrapping(working);
    // unwrap it now (could just use .0 but this is more explicit)
    let Wrapping(mult) = mult;

    let num = (mult >> 24) as u8;

    let partial_check = Wrapping(0x06060606) + Wrapping(working);
    let Wrapping(partial_check) = partial_check;
    let all_digits = (working | partial_check) & 0xF0F0F0F0 == 0;
    let swapped = u32::from_be_bytes(working.to_le_bytes());

    if !all_digits || swapped > 0x00020505 {
        return None;
    }

    Some(num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_8_bit_values_pass() {
        for i in 0..=255 {
            let s = format!("{}", i);
            let num = make_u8(&s).unwrap();
            assert_eq!(num, i);
        }
    }

    #[test]
    fn all_3_digit_values_greater_than_255_fail() {
        for i in 256..=999 {
            let s = format!("{}", i);
            let num = make_u8(&s);
            assert!(num.is_none(), "failed for {}", s);
        }
    }

    #[test]
    fn a_4_digit_string_fails() {
        // will also fail on "0000"...
        let s = "1000";
        let num = make_u8(s);
        assert!(num.is_none(), "failed for {}", s);
    }

    #[test]
    fn an_empty_string_fails() {
        let s = "";
        let num = make_u8(s);
        assert!(num.is_none(), "failed for {}", s);

        let lt_zero = (0x00..0x30).collect::<Vec<u8>>();
        let gt_nine = (0x3a..=0x7f).collect::<Vec<u8>>();
        let non_numeric: Vec<u8> = [lt_zero, gt_nine].concat();

        for c in non_numeric {
            let c = c as char;

            for d in '0'..='9' {
                for e in '0'..='9' {
                    let s = format!("{}{}{}", d, e, c);
                    let num = make_u8(&s);
                    assert!(num.is_none(), "failed for {}", s);
                }
            }
        }
    }

    #[test]
    fn non_numeric_bytes_in_any_position_fail() {
        let lt_zero = (0x00..0x30).collect::<Vec<u8>>();
        let gt_nine = (0x3a..=0xff).collect::<Vec<u8>>();
        let non_numeric: Vec<u8> = [lt_zero, gt_nine].concat();

        let mut u = {
            #[repr(C)]
            union U {
                bytes: [u8; 4],
                num: u32,
            }
            U { num: 0 }
        };

        for pos in 0..3 {
            for &c in non_numeric.iter() {
                unsafe {
                    u.bytes[pos] = c;
                }
                for d in '0'..='9' {
                    unsafe {
                        u.bytes[(pos + 1) % 3] = d as u8;
                    }
                    for e in '0'..='9' {
                        unsafe {
                            u.bytes[(pos + 2) % 3] = e as u8;
                        }
                        let s = unsafe { std::str::from_utf8_unchecked(&u.bytes[0..3]) };
                        let num = make_u8(s);
                        assert!(num.is_none(), "failed for {}", s);
                    }
                }
            }
        }
    }
}
