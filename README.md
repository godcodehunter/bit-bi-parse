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
			bit_write(&mut target, &mut offset, 6, self.field1);
			bit_write(&mut target, &mut offset, 32, self.field2);
			bit_write(&mut target, &mut offset, 4, self.field3);
			bit_write(&mut target, &mut offset, 64, self.field4);
			bit_write(&mut target, &mut offset, 6, self.field5);
			
			target
		}
	}
```