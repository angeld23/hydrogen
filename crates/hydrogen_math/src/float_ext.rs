use cgmath::{ulps_eq, InnerSpace, Vector2, Vector3, Vector4, Zero};

pub trait IsSmall {
    fn is_small(&self) -> bool;
}

impl IsSmall for f32 {
    fn is_small(&self) -> bool {
        ulps_eq!(*self, 0.0)
    }
}

impl IsSmall for Vector2<f32> {
    fn is_small(&self) -> bool {
        self.magnitude2().is_small()
    }
}

impl IsSmall for Vector3<f32> {
    fn is_small(&self) -> bool {
        self.magnitude2().is_small()
    }
}

impl IsSmall for Vector4<f32> {
    fn is_small(&self) -> bool {
        self.magnitude2().is_small()
    }
}

pub trait AddWithEpsilon {
    fn add_with_epsilon(self, rhs: Self) -> Self;
}

impl AddWithEpsilon for f32 {
    fn add_with_epsilon(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return self;
        }

        rhs + if rhs.is_sign_positive() {
            self.next_up()
        } else {
            self.next_down()
        }
    }
}
