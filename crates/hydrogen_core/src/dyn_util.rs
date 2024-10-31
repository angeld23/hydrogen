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
        other.downcast_ref::<T>().map_or(false, |item| self == item)
    }
}

impl PartialEq for Box<dyn DynPartialEq> {
    fn eq(&self, other: &Self) -> bool {
        (**self).dyn_eq((**other).as_any())
    }
}
