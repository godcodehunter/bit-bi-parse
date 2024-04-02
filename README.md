Very simple library for binary serialization/deserialization.

Example simple serialization:

```rust
	struct Sample {
		field1: u64,
		field2: u64,
		field3: u64,
		field4: u64,
		field5: u64,
	}

	impl Into<[u8; 14]> for Sample {
		fn into(self) -> [u8; 14] {
			let mut target = [0u8; 14];
			let mut offset = 0;

			let b_field1 = self.field1.to_be_bytes();
			bit_write(&mut target, offset, 6, &b_field1, b_field1.len());
			offset += 6;
			
			let b_field2 = self.field2.to_be_bytes();
			bit_write(&mut target, offset, 32, &b_field2, b_field2.len());
			offset += 32;

			let b_field3 = self.field3.to_be_bytes();
			bit_write(&mut target, offset, 4, &b_field3, b_field3.len());
			offset += 4;

			let b_field4 = self.field4.to_be_bytes();
			bit_write(&mut target, offset, 64, &b_field4, b_field4.len());
			offset += 64;

			let b_field5 = self.field5.to_be_bytes();
			bit_write(&mut target, offset, 6, &b_field5, b_field5.len());
			
			target
		}
	}
```