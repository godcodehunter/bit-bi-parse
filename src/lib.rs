use std::ops::{Index, IndexMut};

/// Checks that's current stored value, 
/// does not exceed the `bit_size` range  
pub fn is_in_range<'i>(
    bit_size: usize, 
    source: impl IntoIterator<Item = &'i u8>, 
    source_len: usize,
) -> bool {
    if bit_size == 0 {
        return true;
    }

    assert!(
        bit_size <= source_len * 8,
        "bit_size large than source bit size"
    );
    
    // For example for byte. If `bit_size` is 5, then first 
    // three bit's should be empty
    //
    // |0|0|0|1|0|1|1|1|
    // ------ 
    let empty_bit = source_len * 8 - bit_size;
    
    // Calculate the index of partial filled byte
    let pf_byte_num = empty_bit / 8;
    
    // Calculate the number of empty bit in at 
    // the beginning of last partial filled byte
    //  |eb|eb|eb|eb|pb|fb|fb|fb|
    //  ------------ -- --------
    //  |            |  \
    //  |            |   \- filled byte
    //  |            \
    //  |             \- partial filled
    //  \               
    //   full empty byte
    // 
    // NOTE: Content placed in to the second half 
    // of the partially filled byte and then to the 
    // filled byte
    let empty_in_last = empty_bit % 8;
    
    // We iterate byte by byte and check the following condition:
    // |eb|eb|eb|pb|fb|fb|
    //
    // eb: should be empty
    // pb: meet the requirements of the mask
    // fb: not checked
    for (index, byte) in source.into_iter().enumerate() {
        // eb: should be empty
        if index < pf_byte_num && *byte != 0 {
            return false;
        }
        
        // pb: meet the requirements of the mask
        if index == pf_byte_num {
            // For example `empty_in_last` is 3, then
            //
            // 0b11111111 << (8 - 3)
            // 0b11111111 << 5
            // 0b11100000
            // 
            // E - examinee
            // 0bEEEEEEEE 
            // 0b11100000
            // ----------
            // 0bEEE00000
            //   ---
            //      \
            //       and should be equal to 0
            return (byte & 0b11111111 << (8 - empty_in_last) ) == 0 
        }
    }   
    unreachable!()
}

/// Writes N bits from source to target by bit offset
///
/// **PANIC**: If requested bit_size large than source bit size
///
/// **NOTE**: For the source, it does not check if the value exceeds the possible range,
/// that is, the most significant bits are simply discarded
pub fn bit_write<T, S>(
    target: &mut T,
    bit_offset: usize,
    bit_size: usize,
    source: &S,
    source_len: usize,
) where
    T: IndexMut<usize, Output = u8>,
    S: Index<usize, Output = u8>,
{
    if bit_size == 0 {
        return;
    }

    assert!(
        bit_size <= source_len * 8,
        "bit_size large than source bit size"
    );

    let start_byte_index = bit_offset / 8;
    let slots_at_start_byte = 8 - bit_offset % 8;

    let mut affected_bytes = 1;
    let remainder = bit_size.saturating_sub(slots_at_start_byte);
    if remainder != 0 {
        affected_bytes += remainder / 8;

        if remainder % 8 > 0 {
            affected_bytes += 1;
        }
    }

    let mut meaningful_len = bit_size / 8;
    if bit_size % 8 > 0 {
        meaningful_len += 1;
    }
    let mut fullness = bit_offset % 8;
    let mut cursor = bit_size;
    for i in 0..affected_bytes {
        loop {
            let mut index = source_len - meaningful_len;
            let already_written = bit_size - cursor;
            if already_written >= bit_size % 8 && already_written > 0 {
                index += 1;
                if already_written / 8 > 1 {
                    index += already_written / 8 - 1
                }
            }

            let slots = if fullness != 0 { 8 - fullness } else { 8 };
            let available = if cursor % 8 != 0 { cursor % 8 } else { 8 };

            let write_size;
            
            let mask = !0b11111111u8.checked_shl(available as u32).unwrap_or_default();
            if slots >= available {
                write_size = available;
                fullness += available;
                
                let shift = slots - available;
                target[start_byte_index + i] |= (mask & source[index]) << shift;
            } else {
                write_size = slots;
                fullness = 8;

                let shift = available - slots;
                target[start_byte_index + i] |= (mask & source[index]) >> shift;
            }
            cursor -= write_size;

            if cursor == 0 {
                return;
            }
            if fullness == 8 {
                fullness = 0;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut target = [0u8; 2];
        let source = u64::from_be_bytes([
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000111,
            0b11111111,
        ]);

        let b_source = source.to_be_bytes();
        bit_write(&mut target, 4, 11, &b_source, b_source.len());
        assert_eq!(target, [0b00001111, 0b11111110]);
    }

    #[test]
    fn is_not_in_range() {
        let source = 0b00011111u8;
        let b_source = source.to_be_bytes();
        assert!(!is_in_range(4, &b_source, b_source.len()));
        assert!(is_in_range(5, &b_source, b_source.len()));
    }
}

pub fn bit_read<T, S>(
    source: &T,
    bit_offset: usize,
    bit_size: usize,
    target: &mut S,
    target_len: usize,
) where
T: IndexMut<usize, Output = u8>,
S: Index<usize, Output = u8>,
{
    if bit_size == 0 {
        return;
    }

    assert!(
        bit_size <= target_len * 8,
        "bit_size large than target bit size"
    );

    let start_byte_index = bit_offset / 8;
    
}