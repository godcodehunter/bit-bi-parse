use std::ops::{Index, IndexMut};

/// Maximum value that N bits can store
pub fn bits_to_max_hold(bit_size: u32) -> u32 {
    (1 << bit_size) - 1
}

#[test]
fn check_bits_to_max_hold() {
    assert_eq!(63, bits_to_max_hold(6));
}

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
    let ahead_empty_bit = source_len * 8 - bit_size;
    
    // Calculate the index of partial filled byte
    let pf_byte_index = ahead_empty_bit / 8;
    
    // Calculate the number of empty bit in at 
    // the beginning of partial filled byte
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
    let empty_in_start_of_pf = ahead_empty_bit % 8;
    
    // We iterate byte by byte and check the following condition:
    // |eb|eb|eb|pb|fb|fb|
    //
    // eb: should be empty
    // pb: meet the requirements of the mask
    // fb: not checked
    for (index, byte) in source.into_iter().enumerate() {
        // eb: should be empty
        if index < pf_byte_index && *byte != 0 {
            return false;
        }
        
        // pb: meet the requirements of the mask
        if index == pf_byte_index {
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
            return (byte & 0b11111111 << (8 - empty_in_start_of_pf) ) == 0 
        }
    }   
    unreachable!()
}

mod tests_is_in_range {
    use super::*;
    
    #[test]
    fn check_pf() {
        let source = [0b00000000u8, 0b00011111u8];
        assert!(!is_in_range(4, &source, source.len()));
        assert!(is_in_range(5, &source, source.len()));
    }

    #[test]
    fn check_eb() {
        let source = [0b00001000u8, 0b00011111u8];
        assert!(!is_in_range(5, &source, source.len()));
    }
}

/// Writes N bits from source to target by bit offset
/// 
/// **PANIC**: If requested bit_size large than source bit size
///
/// **NOTE**: For the source, it does not check if the value exceeds the possible range,
/// that is, the most significant bits, that out of `bit_size`, are simply discarded.
/// 
/// **NOTE**: It is assumed that the target is prepared for writing, i.e., 
/// for example, no cleaning is applied
pub fn bit_write<T, S>(
    target: &mut T,
    bit_offset: usize,
    bit_size: usize,
    source: &S,
    byte_source_len: usize,
) where
    T: IndexMut<usize, Output = u8>,
    S: Index<usize, Output = u8>,
{
    if bit_size == 0 {
        return;
    }

    assert!(
        bit_size <= byte_source_len * 8,
        "bit_size large than source bit size"
    );

    // The index of the first byte of bytes to which 
    // the recording will be performed
    let start_byte_index = bit_offset / 8;

    // If we imagine that `bit_offset` is 3, then 
    // there are 5 slots (bits in which we write)
    //              |
    //              |
    //              ----------
    //  ... # |1|0|1|0|0|0|0|0| # |0|0|0|0|0|0|0|0| # ...
    //       \_________________/ \_________________/  
    //               |                   \
    //   first partially affected byte   next affected byte
    //
    let slots_at_start_byte = 8 - bit_offset % 8;

    // The number of bytes to which the recording will be performed
    //     |
    //     \-----------
    // |b|b|pa|a|a|a|pa|b|b|
    //
    // NOTE: Minimum 1, since `bit_size` > 0
    let mut affected_bytes_num = 1;

    // NOTE: we use here saturating subtraction, because
    // we have a situation where there are enough 
    // slots in the first partially affected byte for recording
    let remainder = bit_size.saturating_sub(slots_at_start_byte);
    
    // Check the situation described in the note above 
    if remainder != 0 {
        // Add affected byte
        affected_bytes_num += remainder / 8;

        // If exist remainder, add last partially affected byte
        if remainder % 8 > 0 {
            affected_bytes_num += 1;
        }
    }

    // We calculate the length of the written body in bytes (rounding up)
    let mut meaningful_len = bit_size / 8;
    if bit_size % 8 > 0 {
        meaningful_len += 1;
    }

    // Counter of the number of slots already occupied 
    // in the current byte. Here we initialize for
    // first partially affected byte
    //
    //        for ex 3 occupied slots, so `fullness` == 3
    // ------/
    // |1|1|1|0|0|0|0|0| 
    //
    let mut fullness = bit_offset % 8;
    
    // A counter that counts the number bits remaining for recording.
    let mut cursor = bit_size;
    
    // Iterate affected bytes
    //
    // |b|b|pa|a|a|a|pa|b|b|
    //      ^  ^ ^ ^ ^
    //      --------->
    let last_byte_index = start_byte_index + affected_bytes_num;
    let iter_range = start_byte_index..last_byte_index;
    for target_index in iter_range {
        loop {
       
            // Calculate index of first byte being written
            // NOTE: `source_len` can be bigger than `meaningful_len`, 
            // so the most significant byte is simply discarded. 
            let mut source_index = byte_source_len - meaningful_len;
            
            // Number of bits already written
            let already_written = bit_size - cursor;

            /*
                CALCULATE now current index of byte being written
            */

            // If `already_written` === 0, we haven't recorded anything yet.
            //
            // `already_written` >= `bit_size` % 8, handle that we write
            // affected significant bit from first partially affected byte
            // from source
            //
            //  |b|b|pa|a|a|a|pa|b|b| <--- target
            //       ^
            //       |------        --- bit from source thats we write
            //              \      /
            //              -----------
            //   ... # |1|0|1|1|0|0|1|1| # |1|0|1|1|0|0|1|1| # ... <-- source
            //              -------------
            //              |
            //              bit for which will be recorded
            //              from first partially affected byte
            //
            //
            if already_written >= bit_size % 8 && already_written > 0 {
                // If record bit from first partially affected byte
                // from source add to index one
                source_index += 1;

                // If you wrote more than 1 byte, add the remaining ones
                if already_written / 8 > 1 {
                    source_index += already_written / 8 - 1
                }
            }

            /*
                CALCULATE END
            */

            // The available number of slots to which we will write 
            // in the current byte in the TARGET
            let slots_in_target_byte = if fullness != 0 { 8 - fullness } else { 8 };
            
            // Available for printing bit slots from SOURCE!
            // The calculation algorithm is as follows if the remainder is zero. 
            // We can write a whole byte, if there is a remainder, then it is 
            // equal to the number of slots that be printed to target
            let available_for_print = if cursor % 8 != 0 { cursor % 8 } else { 8 };

            let write_size;
            
            let mask = !0b11111111u8.checked_shl(available_for_print as u32).unwrap_or_default();

            // We handle the situation when there are more slots in TARGET than 
            // slots in SOURCE. That is, we can write all the bits from the 
            // SOURCE byte to the TARGET byte.
            if slots_in_target_byte >= available_for_print {
                // We will record the `available` number of bits, track it
                write_size = available_for_print;
                // Also let's change the `fullness` by the amount available
                fullness += available_for_print;
                
                // For example:
                //
                // TARGET
                // 
                //      already recorded (fullness == 3)
                //       |
                //  ------
                // |1|1|1|0|0|0|0|0| 
                //        ---------
                //                 \
                //                 slots_in_target_byte
                // SOURCE
                //
                // |1|0|1|1|0|0|1|1|
                //  ------- --------
                // |                \
                // |               available_for_print
                // already printed
                //
                // So we should:
                //  - remove already printed bit by mask (because otherwise there will be an intersection)
                //  - shift printed byte by one to left
                //
                let shift = slots_in_target_byte - available_for_print;
                target[target_index] |= (mask & source[source_index]) << shift;

            // There are not enough slots in the TARGET byte, 
            // we can only write part of the SOURCE
            } else {
                // We will record the `slots_in_target_byte` number of bits, track it
                write_size = slots_in_target_byte;
                fullness = 8;

                // For example:
                //   
                // TARGET
                //
                //      already recorded (fullness == 3)
                //       |
                //  ------------
                //  |1|1|1|1|1|1|0|0|  
                //               ----
                //                   \        
                //                 slots_in_target_byte 
                //
                //
                // SOURCE
                //
                // |1|0|1|1|0|0|1|1|
                // ---- -------------
                // \                 \
                //  already printed  available_for_print
                //
                // So we should shift printed byte by four to right
                //
                // SOURCE
                //
                // |0|0|0|0|1|0|1|1|
                //
                // Then apply mask 
                //
                //
                // |0|0|0|0|0|0|1|1|
                //
                //
                let shift = available_for_print - slots_in_target_byte;
                target[target_index] |= (mask & source[source_index]) >> shift;
            }

            // Reduce by the amount of written
            cursor -= write_size;

            // We have written all the bits, we are finishing the procedure
            if cursor == 0 {
                return;
            }

            // We have completely filled the byte in TARGET, 
            // there is nowhere else to write, we move on 
            // to the next byte in TARGET
            if fullness == 8 {
                fullness = 0;
                break;
            }
        }
    }
}

mod tests_bit_write {
    use super::*;

    #[test]
    fn heck_intersection() {
        let mut target = [0u8; 2];
        let source = u64::from_be_bytes([
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 
            0b00000000, 0b00000111, 0b11111111,
        ]);

        let b_source = source.to_be_bytes();
        bit_write(&mut target, 4, 11, &b_source, b_source.len());
        assert_eq!(target, [0b00001111, 0b11111110]);
    }

    #[test]
    fn heck_small() {
        let mut target = [0u8; 2];
        let source = u64::from_be_bytes([
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 
            0b00000000, 0b00000000, 0b00000111,
        ]);

        let b_source = source.to_be_bytes();
        bit_write(&mut target, 3, 3, &b_source, b_source.len());
        assert_eq!(target, [0b00011100, 0b00000000]);
    }
}

/// Reset bits to zero in the range of `bit_size` 
/// in target at the specified `bit_offset`
pub fn bit_clean<T>(
    target: &mut T,
    bit_offset: usize,
    bit_size: usize,
) where
    T: IndexMut<usize, Output = u8>
{
    if bit_size == 0 {
        return;
    }
    
    let start_byte_index = bit_offset / 8;
    let slots_at_start_byte = 8 - bit_offset % 8;
    
    let mut affected_bytes_num = 1;
    let remainder = bit_size.saturating_sub(slots_at_start_byte);
    if remainder != 0 {
        affected_bytes_num += remainder / 8;
        if remainder % 8 > 0 {
            affected_bytes_num += 1;
        }
    }

    // If we touch only one byte
    if affected_bytes_num == 1 {
        //
        // For example:
        //
        //    slots_at_start_byte
        //      \
        //       -----------
        //       |         |
        // |1|1|1|1|1|1|1|1|
        // |     |     |   
        //  ----- -----
        //    |     \
        //    |      bit_size (content)
        //     \      
        //   bit_offset % 8             
        //
        // So, mask composed from two part
        let mut mask = 0b00000000;
        // First we shift << slots_at_start_byte, for doesn't touch some ahead bits
        //
        //  |1|1|1|0|0|0|0|0|
        //
        mask |= 0b11111111u8.checked_shl(slots_at_start_byte as u32).unwrap_or_default();
        // Second we shift >> bit_offset % 8 + bit_size, for doesn't touch some postponed bits
        //
        // |0|0|0|0|0|0|1|1|
        //
        let rsh = bit_offset % 8 + bit_size;
        // End here finally
        //
        // |1|1|1|0|0|0|1|1|
        // 
        mask |= 0b11111111u8.checked_shr(rsh as u32).unwrap_or_default();

        target[start_byte_index] &= mask;
        
        return;
    }

    let last_byte_index = start_byte_index + affected_bytes_num;
    let iter_range = start_byte_index..last_byte_index;

    for target_index in iter_range {
        let mut mask = 0b00000000;
        if target_index ==  start_byte_index {
            mask = 0b11111111u8.checked_shl(slots_at_start_byte as u32).unwrap_or_default(); 
        }
        if target_index == last_byte_index - 1 && remainder % 8 > 0 {
            let slots_at_last_byte = remainder - remainder/8*8;
            mask = 0b11111111 >> slots_at_last_byte;
        }

        target[target_index] &= mask;
    }
}

mod tests_bit_clean {
    use super::*;

    #[test]
    fn check_intersection() {
        let mut target = [
            0b00000111, 0b11111111, 0b11100000, 0b00000000
        ];
        
        bit_clean(&mut target, 5, 3 + 8 + 3);
        let expected = [0b00000000, 0b00000000, 0b00000000, 0b00000000];
        assert_eq!(expected, target);
    }

    #[test]
    fn check_without_last() {
        let mut target = [
            0b00000111, 0b11111111, 0b11100000, 0b00000000
        ];
        
        bit_clean(&mut target, 5, 3 + 8);
        let expected = [0b00000000, 0b00000000, 0b11100000, 0b00000000];
        assert_eq!(expected, target);
    }

    #[test]
    fn check_one_byte_small() {
        let mut target = [
            0b00000111, 0b11111111, 0b11100000, 0b00000000
        ]; 

        bit_clean(&mut target, 11, 3);
        let expected = [0b00000111, 0b11100011, 0b11100000, 0b00000000];
         
        assert_eq!(expected, target)
    }

    #[test]
    fn check_last_start() {
        let mut target = [
            0b00000111, 0b11111111, 0b11100000, 0b00000000
        ]; 

        bit_clean(&mut target, 16, 3);
        let expected = [0b00000111, 0b11111111, 0b00000000, 0b00000000];
        
        assert_eq!(expected, target)
    }

    #[test]
    fn check_last_end() {
        let mut target = [
            0b00000111, 0b11111111, 0b00000111, 0b00000000
        ]; 

        bit_clean(&mut target, 21, 3);
        let expected = [0b00000111, 0b11111111, 0b00000000, 0b00000000];
        
        assert_eq!(expected, target)
    }

    #[test]
    fn check_first_filled() {
        let mut target = [
            0b11111111, 0b11111111, 0b11111111, 0b00000000
        ]; 

        bit_clean(&mut target, 0, 16);
        let expected = [0b00000000, 0b00000000, 0b11111111, 0b00000000];
        
        assert_eq!(expected, target)
    }
}

/// Read N bits from source to target by bit offset
///
/// **NOTE**: It is assumed that the target is prepared for writing, i.e., 
/// for example, no cleaning is applied
pub fn bit_read<T, S>(
    source: &T,
    bit_offset: usize,
    bit_size: usize,
    target: &mut S,
    target_len: usize,
) where
    T: Index<usize, Output = u8>,
    S: IndexMut<usize, Output = u8>,
{
    if bit_size == 0 {
        return;
    }

    assert!(
        bit_size <= target_len * 8,
        "bit_size large than target bit size"
    );

    let start_byte_index = bit_offset / 8;
    let slots_at_start_byte = 8 - bit_offset % 8;

    let mut affected_bytes_num = 1;
    let remainder = bit_size.saturating_sub(slots_at_start_byte);
    if remainder != 0 {
        affected_bytes_num += remainder / 8;
        if remainder % 8 > 0 {
            affected_bytes_num += 1;
        }
    }

    let last_byte_index = start_byte_index + affected_bytes_num;
    let iter_range = start_byte_index..last_byte_index;
    for source_index in iter_range {
        loop {
            todo!();
            // target[0] |= source[source_index];
        }
    }
}