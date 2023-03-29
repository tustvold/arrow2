use crate::array::BinaryArray;
use crate::bitmap::Bitmap;
use crate::datatypes::DataType;
use crate::offset::{Offset, OffsetsBuffer};
use arrow_data::{ArrayData, ArrayDataBuilder};

impl<O: Offset> BinaryArray<O> {
    /// Convert this array into [`ArrayData`]
    pub fn to_data(&self) -> ArrayData {
        let data_type = match O::IS_LARGE {
            true => arrow_schema::DataType::LargeBinary,
            false => arrow_schema::DataType::Binary,
        };

        let builder = ArrayDataBuilder::new(data_type)
            .len(self.offsets().len_proxy())
            .buffers(vec![
                self.offsets.clone().into_inner().into(),
                self.values.clone().into(),
            ])
            .nulls(self.validity.as_ref().map(|b| b.clone().into()));

        // Safety: Array is valid
        unsafe { builder.build_unchecked() }
    }

    /// Create this array from [`ArrayData`]
    pub fn from_data(data: &ArrayData) -> Self {
        let data_type: DataType = data.data_type().clone().into();
        match O::IS_LARGE {
            true => assert_eq!(data_type, DataType::LargeBinary),
            false => assert_eq!(data_type, DataType::Binary),
        };

        if data.len() == 0 {
            // Handle empty offsets
            return Self::new_empty(data_type);
        }

        let buffers = data.buffers();

        // Safety: ArrayData is valid
        let mut offsets = unsafe { OffsetsBuffer::new_unchecked(buffers[0].clone().into()) };
        if data.offset() != 0 {
            offsets.slice(data.offset(), data.len() + 1);
        }

        Self {
            data_type,
            offsets,
            values: buffers[1].clone().into(),
            validity: data.nulls().map(|n| Bitmap::from_null_buffer(n.clone())),
        }
    }
}
