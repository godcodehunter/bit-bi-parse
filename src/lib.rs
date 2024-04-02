use std::ops::IndexMut;

/// Writes N bits from source to target by bit offset
/// 
/// **NOTE**: For the source, it does not check if the value exceeds the possible range, 
/// that is, the most significant bits are simply discarded
pub fn bit_write<'c, C>(target: &mut C, bit_offset: &mut usize, bit_size: usize, source: u64) 
where C: IndexMut<usize, Output =u8> {
	if bit_size == 0 {
		return
	}

    let _data = source.to_be_bytes();

	let start_byte_index = *bit_offset / 8;

	let available_at_first_byte = if *bit_offset % 8 > 0 {
		8 - *bit_offset % 8
	} else {
        0
    };

	let mut record_length = 0;
	if available_at_first_byte > 0 {
		record_length += 1;
	}
	if bit_size - available_at_first_byte > 0 {
		record_length += (bit_size - available_at_first_byte) / 8;

		if (bit_size - available_at_first_byte) % 8 > 0 {
			record_length += 1;
		}
	}

	let mut meaningful_len = bit_size / 8;
	if bit_size % 8 > 0 {
		meaningful_len += 1;
	}
	let mut fullness = *bit_offset % 8;
	let mut cursor = bit_size;
	for i in 0..record_length {
		loop {
			let mut index = 8 - meaningful_len;
			let already_written = bit_size - cursor;
			if already_written >= bit_size % 8 && already_written > 0 {
				index += 1;
				if already_written / 8 > 1 {
					index += already_written / 8 - 1
				}
			}

			let available = if cursor % 8 != 0 {
				cursor % 8 
			} else {
                8
            };

			let write_size;
			let slots = if fullness != 0 {
				8 - fullness
			} else {
                8
            };

			if slots >= available {
				write_size = available;
				fullness += available;
				if fullness == 8 {
					fullness = 0;
				}

				let shift = slots - available;
				target[start_byte_index + i] |= _data[index] << shift;
			} else {
				write_size = slots;
				fullness = 0;

				let shift = available - write_size;
				target[start_byte_index + i] |= _data[index] >> shift;
			}
			cursor -= write_size;

			if fullness == 0 || cursor == 0 {
				break
			}
		}
	}
	*bit_offset += bit_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut target = [0u8; 2];
        let mut offset = 4;
        let source = u64::from_be_bytes([
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000111,
            0b11111111,
        ]);
        
        bit_write(&mut target, &mut offset, 11, source);
        assert_eq!(target, [0b00001111, 0b11111110]);
    }
}
