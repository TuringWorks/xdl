//! Camera system for 3D navigation

use glam::{Mat4, Vec3};
use winit::event::*;

/// Camera for 3D scene navigation
pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,

    // Interaction state
    rotation: Vec3, // yaw, pitch, roll
    distance: f32,
    mouse_pressed: bool,
    last_mouse_pos: (f32, f32),
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, aspect: f32) -> Self {
        let distance = (position - target).length();
        let offset = position - target;

        // Calculate initial rotation from position
        // yaw (azimuth): angle in XZ plane from +X axis
        let yaw = offset.z.atan2(offset.x);
        // pitch (elevation): angle from XZ plane
        let pitch = (offset.y / distance).asin();

        Self {
            position,
            target,
            up: Vec3::Y,
            aspect,
            fovy: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            rotation: Vec3::new(yaw, pitch, 0.0),
            distance,
            mouse_pressed: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.near, self.far)
    }

    /// Get the combined view-projection matrix
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Set the aspect ratio
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Handle input events
    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x as f32, position.y as f32);
                if self.mouse_pressed {
                    let delta_x = pos.0 - self.last_mouse_pos.0;
                    let delta_y = pos.1 - self.last_mouse_pos.1;

                    // Rotate camera - horizontal drag rotates around Y axis (yaw)
                    self.rotation.x -= delta_x * 0.005; // yaw (azimuth)
                    self.rotation.y -= delta_y * 0.005; // pitch (elevation)

                    // Clamp pitch to prevent flipping over the poles
                    let max_pitch = std::f32::consts::FRAC_PI_2 - 0.1; // 89 degrees
                    self.rotation.y = self.rotation.y.clamp(-max_pitch, max_pitch);
                }
                self.last_mouse_pos = pos;
                self.mouse_pressed
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                self.distance -= scroll * 0.1 * self.distance;
                self.distance = self.distance.clamp(0.5, 10.0);
                true
            }
            _ => false,
        }
    }

    /// Update camera position based on interaction
    pub fn update(&mut self) {
        // Spherical coordinates: yaw (azimuth) and pitch (elevation)
        let yaw = self.rotation.x;
        let pitch = self.rotation.y.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.1,
            std::f32::consts::FRAC_PI_2 - 0.1,
        );

        // Convert spherical to Cartesian coordinates
        // This creates proper orbital rotation around the target
        let x = self.distance * yaw.cos() * pitch.cos();
        let y = self.distance * pitch.sin();
        let z = self.distance * yaw.sin() * pitch.cos();

        self.position = self.target + Vec3::new(x, y, z);
    }

    /// Get camera uniform data for GPU
    pub fn uniform_data(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_proj_matrix().to_cols_array(),
            view_pos: [self.position.x, self.position.y, self.position.z, 1.0],
        }
    }
}

/// Camera uniform buffer data
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [f32; 16],
    pub view_pos: [f32; 4],
}
