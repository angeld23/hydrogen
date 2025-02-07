pub use hydrogen_core_proc_macro::DependencyProvider;

pub trait Dependency<D> {
    fn dep(&self) -> &D;
}

pub trait DependencyMut<D> {
    fn dep_mut(&mut self) -> &mut D;
}

impl<T, D> Dependency<D> for &T
where
    T: Dependency<D>,
{
    fn dep(&self) -> &D {
        (*self).dep()
    }
}

impl<T, D> Dependency<D> for &mut T
where
    T: Dependency<D>,
{
    fn dep(&self) -> &D {
        (**self).dep()
    }
}

impl<T, D> DependencyMut<D> for &mut T
where
    T: DependencyMut<D>,
{
    fn dep_mut(&mut self) -> &mut D {
        (*self).dep_mut()
    }
}
