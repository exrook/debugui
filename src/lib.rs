pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(feature = "enabled")]
pub mod enabled;
#[cfg(feature = "enabled")]
pub use enabled::*;

#[cfg(not(feature = "enabled"))]
pub mod disabled;
#[cfg(not(feature = "enabled"))]
pub use disabled::*;

pub trait AsDebuggableParam {
    type Value: 'static + Send + Sync + Clone;
    fn get_value(&self) -> &Self::Value;
    fn set_value(&mut self, value: &Self::Value);
}

impl<T: AsDebuggableParam> AsDebuggableParam for &mut T {
    type Value = T::Value;
    fn get_value(&self) -> &Self::Value {
        T::get_value(self)
    }
    fn set_value(&mut self, new_value: &Self::Value) {
        T::set_value(self, new_value)
    }
}

macro_rules! clone_type {
    ($ty:ty) => {
        impl AsDebuggableParam for $ty {
            type Value = $ty;
            fn get_value(&self) -> &Self::Value {
                self
            }
            fn set_value(&mut self, new_value: &Self::Value) {
                self.clone_from(new_value);
            }
        }
    };
    ($($ty:ty),* $(,)?) => {
        $(clone_type!($ty);)*
    }
}

clone_type!(bool, char, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
clone_type!(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
