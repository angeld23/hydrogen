use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait DynPartialEq: AsAny {
    fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T> DynPartialEq for T
where
    T: PartialEq<T> + 'static,
{
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<T>() == Some(self)
    }
}

impl PartialEq for Box<dyn DynPartialEq> {
    fn eq(&self, other: &Self) -> bool {
        (**self).dyn_eq((**other).as_any())
    }
}

#[macro_export]
macro_rules! downcast {
    ($value:ident, $ty:ty) => {
        $value.as_any().downcast_ref::<$ty>().unwrap()
    };
}

#[macro_export]
macro_rules! downcast_mut {
    ($value:ident, $ty:ty) => {
        $value.as_any_mut().downcast_mut::<$ty>().unwrap()
    };
}
