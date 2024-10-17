use super::data::{DataFormatType, DataPreviewOptions, Endianness};

macro_rules! hex {
  ($ty:ty) => {hex!($ty => |x| format!("{x:x}"))};

  ($ty:ty => $fmt:expr) => {
    (|bytes: &[u8], endian| {
      ($fmt)(match endian {
        Endianness::Big => <$ty>::from_be_bytes(bytes.try_into().unwrap()),
        Endianness::Little => <$ty>::from_le_bytes(bytes.try_into().unwrap()),
      })
    })
  };
}

/// Turn a provided slice into a decimal [`String`] representing it's value, interpretation is based on the provided
/// [`crate::option_data::DataPreviewOptions`].
///
/// The provided `bytes` slice is expected to have the appropriate amount of bytes, or else the function will panic.
pub fn bytes_to_hex(data_preview: DataPreviewOptions, bytes: &[u8]) -> String {
  let endian = data_preview.selected_endianness;
  match data_preview.selected_data_format {
    DataFormatType::U8 => hex!(u8)(bytes, endian),
    DataFormatType::U16 => hex!(u16)(bytes, endian),
    DataFormatType::U32 => hex!(u32)(bytes, endian),
    DataFormatType::U64 => hex!(u64)(bytes, endian),
    DataFormatType::I8 => hex!(i8)(bytes, endian),
    DataFormatType::I16 => hex!(i16)(bytes, endian),
    DataFormatType::I32 => hex!(i32)(bytes, endian),
    DataFormatType::I64 => hex!(i64)(bytes, endian),
    DataFormatType::F32 => {
      hex!(f32 => |x| format!("{:x}", x as u64))(bytes, endian)
    }
    DataFormatType::F64 => {
      hex!(f64 => |x| format!("{:x}", x as u64))(bytes, endian)
    }
  }
}
