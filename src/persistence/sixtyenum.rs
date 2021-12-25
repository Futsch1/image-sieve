use num_traits::{FromPrimitive, ToPrimitive};
use sixtyfps::{Model, ModelHandle, SharedString};

pub fn model_to_enum<Enum>(model: &ModelHandle<SharedString>, value: &SharedString) -> Enum
where
    Enum: FromPrimitive,
{
    let mut enum_value: Enum = FromPrimitive::from_usize(0).unwrap();
    for index in 0..model.row_count() {
        if model.row_data(index) == value {
            enum_value = FromPrimitive::from_usize(index).unwrap()
        }
    }
    enum_value
}

pub fn enum_to_model<Enum>(model: &ModelHandle<SharedString>, value: &Enum) -> SharedString
where
    Enum: ToPrimitive,
{
    let enum_value: u32 = ToPrimitive::to_u32(value).unwrap();
    model.row_data(enum_value as usize)
}
