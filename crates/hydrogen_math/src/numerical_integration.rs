use std::{
    convert::FloatToInt,
    ops::{Add, AddAssign, Div, Mul},
};

use cgmath::num_traits::{Euclid, Float};

use crate::float_ext::NextFloat;

pub fn runge_kutta_step<T, F>(
    initial_value: T,
    initial_time: F,
    time_step: F,
    mut derivative: impl FnMut(F, T) -> T,
) -> T
where
    T: Copy + Add<Output = T> + AddAssign + Mul<F, Output = T> + Div<F, Output = T>,
    F: Float + NextFloat,
{
    let two = F::one() + F::one();
    let six = two + two + two;

    let k_1 = derivative(initial_time, initial_value);
    let k_2 = derivative(
        initial_time + time_step / two,
        initial_value + k_1 * time_step / two,
    );
    let k_3 = derivative(
        initial_time + time_step / two,
        initial_value + k_2 * time_step / two,
    );
    let k_4 = derivative(initial_time + time_step, initial_value + k_3 * time_step);

    initial_value + (k_1 + k_2 * two + k_3 * two + k_4) * (time_step / six)
}

pub fn runge_kutta_evaluate<T, F>(
    time: F,
    initial_value: T,
    initial_time: F,
    mut step_size: F,
    mut derivative: impl FnMut(F, T) -> T,
) -> T
where
    T: Copy + Add<Output = T> + AddAssign + Mul<F, Output = T> + Div<F, Output = T>,
    F: Float + NextFloat + Euclid + FloatToInt<u32> + AddAssign,
{
    let time = time.max(initial_time);

    let mut current_value = initial_value;
    let mut current_time = initial_time;

    let step_count = ((time - initial_time) / step_size).to_u32().unwrap() + 1;

    for i in 0..step_count {
        if i == step_count - 1 {
            step_size = (time - initial_time).rem_euclid(&step_size);
        }

        current_value = runge_kutta_step(current_value, current_time, step_size, &mut derivative);
        current_time += step_size;
    }

    current_value
}
