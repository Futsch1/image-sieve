use num_traits::{FromPrimitive, ToPrimitive};
use slint::{Model, ModelRc, SharedString};

/// Convert a value from a slint SharedString model to an enum by mapping the model index to the enum value.
pub fn model_to_enum<Enum>(model: &ModelRc<SharedString>, value: &SharedString) -> Enum
where
    Enum: FromPrimitive,
{
    let mut enum_value: Enum = FromPrimitive::from_usize(0).unwrap();
    for index in 0..model.row_count() {
        if model.row_data(index).unwrap() == value {
            enum_value = FromPrimitive::from_usize(index).unwrap()
        }
    }
    enum_value
}

/// Convert an enum to a value from a slint SharedString modelby mapping the enum value to the model index.
pub fn enum_to_model<Enum>(model: &ModelRc<SharedString>, value: &Enum) -> SharedString
where
    Enum: ToPrimitive,
{
    let enum_value: u32 = ToPrimitive::to_u32(value).unwrap();
    model.row_data(enum_value as usize).unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;
    use num_derive::{FromPrimitive, ToPrimitive};
    use slint::VecModel;

    #[derive(PartialEq, FromPrimitive, ToPrimitive, std::fmt::Debug)]
    enum Test {
        A = 0,
        B,
        C,
    }

    #[test]
    fn test_model_to_enum() {
        let model = VecModel::from(vec![
            SharedString::from("A"),
            SharedString::from("B"),
            SharedString::from("C"),
        ]);
        let model_rc = ModelRc::new(model);
        assert_eq!(
            model_to_enum::<Test>(&model_rc, &SharedString::from("A")),
            Test::A
        );
        assert_eq!(
            model_to_enum::<Test>(&model_rc, &SharedString::from("B")),
            Test::B
        );
        assert_eq!(
            model_to_enum::<Test>(&model_rc, &SharedString::from("C")),
            Test::C
        );
        assert_eq!(
            model_to_enum::<Test>(&model_rc, &SharedString::from("X")),
            Test::A
        );
    }

    #[test]
    fn test_enum_to_model() {
        let model = VecModel::from(vec![
            SharedString::from("A"),
            SharedString::from("B"),
            SharedString::from("C"),
        ]);
        let model_rc = ModelRc::new(model);
        assert_eq!(
            enum_to_model::<Test>(&model_rc, &Test::A),
            &SharedString::from("A")
        );
        assert_eq!(
            enum_to_model::<Test>(&model_rc, &Test::B),
            &SharedString::from("B")
        );
        assert_eq!(
            enum_to_model::<Test>(&model_rc, &Test::C),
            SharedString::from("C")
        );
    }
}
