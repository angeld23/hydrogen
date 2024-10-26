use crate::bounding_box::{bbox, BBox2};
use cgmath::{ElementWise, Vector2};
use derive_more::*;
use std::mem;

pub fn rect_fits(container: Vector2<u32>, inner: Vector2<u32>) -> bool {
    container.x >= inner.x && container.y >= inner.y
}

#[derive(Debug, Clone, Copy, From, Into)]
pub struct UVHelper(pub u32, pub u32);

impl UVHelper {
    pub fn bbox(self, corner_0: impl Into<(u32, u32)>, corner_1: impl Into<(u32, u32)>) -> BBox2 {
        let corner_0: (u32, u32) = corner_0.into();
        let corner_1: (u32, u32) = corner_1.into();

        bbox!(
            (
                corner_0.0 as f32 / self.0 as f32,
                corner_0.1 as f32 / self.1 as f32
            ),
            (
                corner_1.0 as f32 / self.0 as f32,
                corner_1.1 as f32 / self.1 as f32
            )
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FourCorners<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PackedSection {
    pub layer_index: u32,
    pub uv: BBox2,
}

impl From<BBox2> for PackedSection {
    fn from(value: BBox2) -> Self {
        Self {
            layer_index: 0,
            uv: value,
        }
    }
}

impl PackedSection {
    pub fn local_uv(self, local_uv: BBox2) -> Self {
        let [min, size]: [Vector2<f32>; 2] = [self.uv.min().into(), self.uv.size().into()];
        let [local_min, local_max]: [Vector2<f32>; 2] =
            [local_uv.min().into(), local_uv.max().into()];

        Self {
            layer_index: self.layer_index,
            uv: bbox!(
                min + local_min.mul_element_wise(size),
                min + local_max.mul_element_wise(size)
            ),
        }
    }

    pub fn local_point(self, local_point: Vector2<f32>) -> Vector2<f32> {
        let [min, size]: [Vector2<f32>; 2] = [self.uv.min().into(), self.uv.max().into()];
        min + local_point.mul_element_wise(size)
    }

    /// width / height
    pub fn aspect_ratio(self) -> f32 {
        self.uv.size()[0] / self.uv.size()[1]
    }

    pub fn unoriented(self) -> OrientedSection {
        self.into()
    }

    pub fn oriented(self, flipped: bool, clockwise_rotations: u8) -> OrientedSection {
        OrientedSection {
            section: self,
            flipped,
            clockwise_rotations,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientedSection {
    pub section: PackedSection,
    pub flipped: bool,
    pub clockwise_rotations: u8,
}

impl From<PackedSection> for OrientedSection {
    fn from(value: PackedSection) -> Self {
        Self {
            section: value,
            flipped: false,
            clockwise_rotations: 0,
        }
    }
}

impl From<BBox2> for OrientedSection {
    fn from(value: BBox2) -> Self {
        PackedSection::from(value).into()
    }
}

impl OrientedSection {
    pub fn flipped(section: PackedSection) -> Self {
        Self {
            section,
            flipped: true,
            clockwise_rotations: 0,
        }
    }

    pub fn rotated(section: PackedSection, clockwise_rotations: u8) -> Self {
        Self {
            section,
            flipped: false,
            clockwise_rotations,
        }
    }

    pub fn with_flipped(mut self, flipped: bool) -> Self {
        self.flipped = flipped;
        self
    }

    pub fn with_rotations(mut self, clockwise_rotations: u8) -> Self {
        self.clockwise_rotations = clockwise_rotations;
        self
    }

    pub fn uv_corners(self) -> FourCorners<[f32; 2]> {
        let uv = self.section.uv;

        let mut top_left = uv.get_corner([false, false]);
        let mut bottom_right = uv.get_corner([true, true]);
        let mut top_right = uv.get_corner([true, false]);
        let mut bottom_left = uv.get_corner([false, true]);

        if self.flipped {
            mem::swap(&mut top_left, &mut top_right);
            mem::swap(&mut bottom_left, &mut bottom_right);
        }

        for _ in 0..self.clockwise_rotations.rem_euclid(4) {
            let temp_top_left = top_left;

            top_left = bottom_left;
            bottom_left = bottom_right;
            bottom_right = top_right;
            top_right = temp_top_left;
        }

        FourCorners {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn local_uv(mut self, local_uv: BBox2) -> Self {
        self.section = self.section.local_uv(local_uv);
        self
    }
}
