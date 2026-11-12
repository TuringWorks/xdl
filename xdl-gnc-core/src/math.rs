// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

use crate::GncError;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }

    pub fn try_normalize(self) -> Result<Self, GncError> {
        if !self.is_finite() {
            return Err(GncError::NonFinite {
                context: "Vec3 normalization".to_string(),
            });
        }
        let norm = self.norm();
        if norm <= f64::EPSILON {
            return Err(GncError::ZeroLengthVector);
        }
        Ok(self / norm)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub const IDENTITY: Self = Self {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    pub fn from_axis_angle(axis: Vec3, angle_rad: f64) -> Result<Self, GncError> {
        if !angle_rad.is_finite() {
            return Err(GncError::NonFinite {
                context: "quaternion angle".to_string(),
            });
        }
        let axis = axis.try_normalize()?;
        let half = angle_rad * 0.5;
        let (sin_half, cos_half) = half.sin_cos();
        Ok(Self::new(
            cos_half,
            axis.x * sin_half,
            axis.y * sin_half,
            axis.z * sin_half,
        ))
    }

    pub fn norm_squared(self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn try_normalize(self) -> Result<Self, GncError> {
        if ![self.w, self.x, self.y, self.z]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(GncError::NonFinite {
                context: "quaternion normalization".to_string(),
            });
        }
        let norm = self.norm_squared().sqrt();
        if norm <= f64::EPSILON {
            return Err(GncError::ZeroLengthQuaternion);
        }
        Ok(Self::new(
            self.w / norm,
            self.x / norm,
            self.y / norm,
            self.z / norm,
        ))
    }

    pub const fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    pub fn rotate_vector(self, vector: Vec3) -> Result<Vec3, GncError> {
        let unit = self.try_normalize()?;
        let q_vector = Self::new(0.0, vector.x, vector.y, vector.z);
        let rotated = unit * q_vector * unit.conjugate();
        Ok(Vec3::new(rotated.x, rotated.y, rotated.z))
    }
}

impl Mul for Quaternion {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn cross_product_is_orthogonal() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(y);
        assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(z.dot(x), 0.0);
        assert_eq!(z.dot(y), 0.0);
    }

    #[test]
    fn quaternion_rotates_vector() {
        let rotation =
            Quaternion::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), FRAC_PI_2).expect("valid axis");
        let rotated = rotation
            .rotate_vector(Vec3::new(1.0, 0.0, 0.0))
            .expect("valid quaternion");
        assert!((rotated.x).abs() < 1e-12);
        assert!((rotated.y - 1.0).abs() < 1e-12);
        assert!((rotated.z).abs() < 1e-12);
    }
}
