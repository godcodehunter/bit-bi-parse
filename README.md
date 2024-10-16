Very simple library for binary printer/parser.

![alt text](./doc/asserts/principle.png)

Example simple printer:

```rust
	struct Sample {
		field1: u64,
		field2: u64,
		field3: u64,
		field4: u64,
		field5: u64,
	}

	#[derive(Debug)]
	struct SamplePrinterError {
		field_name: String,
		value: u64,
		max_value: u64,
	}

	impl fmt::Display for SamplePrinterError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(
				f,
				"Error: field '{}' has value {}, which exceeds the maximum value {}.",
				self.field_name, self.value, self.max_value
			)
		}
	}

	impl Error for SamplePrinterError {}


	impl TryInto<[u8; 14]> for Sample {
		type Error = Vec<&'static str>;

		fn try_into(self) -> Result<[u8; 14], Self::Error> {
			/*
				CONVERT values to bytes for passing to the 
				verification function
			*/
			let b_field1 = self.field1.to_be_bytes();
			let b_field2 = self.field2.to_be_bytes();
			let b_field3 = self.field3.to_be_bytes();
			let b_field4 = self.field4.to_be_bytes();
			let b_field5 = self.field5.to_be_bytes();
			/*
				CONVERT END
			*/

			/*
				CHECK that stored values do not go beyond the storage 
				(there are enough bits for encoding)
			*/
			let mut errors = vec![];
			if !is_in_range(6, &b_field1, b_field1.len()) {
				errors.push(SamplePrinterError{
					field_name: "field1".to_string(),
					value: field1,
					max_value: bits_to_max_hold(6) as u64,
				});
			}
			if !is_in_range(32, &b_field2, b_field2.len()) {
				errors.push(SamplePrinterError{
					field_name: "field2".to_string(),
					value: field2,
					max_value: bits_to_max_hold(32) as u64,
				});
			}
			if !is_in_range(4, &b_field3, b_field3.len()) {
				errors.push(SamplePrinterError{
					field_name: "field3".to_string(),
					value: field3,
					max_value: bits_to_max_hold(4) as u64,
				});
			}
			if !is_in_range(64, &b_field4, b_field4.len()) {
				errors.push(SamplePrinterError{
					field_name: "field4".to_string(),
					value: field4,
					max_value: bits_to_max_hold(64) as u64,
				});
			}
			if !is_in_range(6, &b_field5, b_field5.len()) {
				errors.push(SamplePrinterError{
					field_name: "field5".to_string(),
					value: field5,
					max_value: bits_to_max_hold(6) as u64,
				});
			}

			if !errors.is_empty() {
				return Err(errors);
			}
			/*
				CHECK END
			*/

			/*
				WRITE to a byte array
			*/
			let mut target = [0u8; 14];
			let mut offset = 0;

			bit_write(&mut target, offset, 6, &b_field1, b_field1.len());
			offset += 6;

			bit_write(&mut target, offset, 32, &b_field2, b_field2.len());
			offset += 32;

			bit_write(&mut target, offset, 4, &b_field3, b_field3.len());
			offset += 4;

			bit_write(&mut target, offset, 64, &b_field4, b_field4.len());
			offset += 64;

			bit_write(&mut target, offset, 6, &b_field5, b_field5.len());
			/*
				WRITE END
			*/

			Ok(target)
		}
	}
```

Example simple parser:

<!-- ```rust
	struct Sample {
		field1: u64,
		field2: u64,
		field3: u64,
		field4: u64,
		field5: u64,
	}

	impl TryFrom<[u8; 14]> for Sample {
    	type Error = Vec<&'static str>;

    	fn try_from(bytes: [u8; 14]) -> Result<Self, Self::Error> {
			let mut target: Sample = Default::default();

			bit_read(&bytes, offset, 6, &mut bytes, b_field1_len);
        	offset += 6;

			bit_read(&bytes, offset, 32, &mut target.field2, b_field2_len);
        	offset += 32;

			bit_read(&bytes, offset, 4, &mut target.field3, b_field3_len);
        	offset += 4;

			bit_read(&bytes, offset, 64, &mut target.field4, b_field4_len);
        	offset += 64;

			bit_read(&bytes, offset, 6, &mut target.field5, b_field5_len);

			Ok(target)
		}
	}

``` -->

With use codegen macro:

```rust
	//TODO
```

Similar libraries:

- https://lib.rs/crates/nom
- https://lib.rs/crates/bitvec
- https://lib.rs/crates/bitstream-io + https://lib.rs/crates/tokio-bitstream-io
- https://lib.rs/crates/bitreader
- https://lib.rs/crates/bitter