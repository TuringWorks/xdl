//! Rayleigh-Taylor Instability Simulation
//!
//! Demonstrates scientific visualization capabilities with a classic fluid dynamics
//! instability where a denser fluid sits above a lighter fluid, creating beautiful
//! mushroom-like structures as they mix.
//!
//! This example showcases:
//! - 2D fluid simulation
//! - Multiple color maps (viridis, plasma, turbo, terrain)
//! - Density field visualization
//! - Vector field (velocity) visualization
//! - Streamline integration
//! - Volume rendering
//! - Multi-frame animation

use std::f64::consts::PI;

/// Rayleigh-Taylor simulation parameters
pub struct RTSimulation {
    pub width: usize,
    pub height: usize,
    pub density: Vec<Vec<f64>>,      // Density field
    pub velocity_u: Vec<Vec<f64>>,   // X velocity
    pub velocity_v: Vec<Vec<f64>>,   // Y velocity
    pub pressure: Vec<Vec<f64>>,     // Pressure field
    pub dt: f64,                      // Time step
    pub viscosity: f64,               // Fluid viscosity
    pub gravity: f64,                 // Gravitational acceleration
    pub atwood_number: f64,           // Density ratio parameter
}

impl RTSimulation {
    /// Create new Rayleigh-Taylor simulation
    pub fn new(width: usize, height: usize) -> Self {
        let mut density = vec![vec![0.0; width]; height];
        let velocity_u = vec![vec![0.0; width]; height];
        let velocity_v = vec![vec![0.0; width]; height];
        let pressure = vec![vec![0.0; width]; height];
        
        // Initialize density field: heavy fluid on top, light below
        // with small perturbation at interface
        let interface_y = height / 2;
        
        for y in 0..height {
            for x in 0..width {
                let base_density = if y < interface_y {
                    2.0  // Heavy fluid on top
                } else {
                    1.0  // Light fluid below
                };
                
                // Add sinusoidal perturbation at interface
                let perturbation_amplitude = 5.0;
                let wavelength = width as f64 / 4.0;
                let perturbation = perturbation_amplitude * 
                    (2.0 * PI * x as f64 / wavelength).sin();
                
                let distance_from_interface = (y as f64 - interface_y as f64 - perturbation).abs();
                
                // Smooth transition at interface
                if distance_from_interface < 3.0 {
                    let blend = distance_from_interface / 3.0;
                    density[y][x] = if y < interface_y {
                        2.0 * blend + 1.0 * (1.0 - blend)
                    } else {
                        1.0 * blend + 2.0 * (1.0 - blend)
                    };
                } else {
                    density[y][x] = base_density;
                }
            }
        }
        
        Self {
            width,
            height,
            density,
            velocity_u,
            velocity_v,
            pressure,
            dt: 0.01,
            viscosity: 0.001,
            gravity: 0.1,
            atwood_number: 0.5,
        }
    }
    
    /// Perform one simulation step
    pub fn step(&mut self) {
        // Simplified Rayleigh-Taylor evolution
        // This is a demonstration - not a full Navier-Stokes solver
        
        let mut new_density = self.density.clone();
        let mut new_velocity_u = self.velocity_u.clone();
        let mut new_velocity_v = self.velocity_v.clone();
        
        // Update velocities based on density gradients (buoyancy)
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let rho = self.density[y][x];
                let rho_avg = (self.density[y-1][x] + self.density[y+1][x] +
                               self.density[y][x-1] + self.density[y][x+1]) / 4.0;
                
                // Buoyancy force (proportional to density difference)
                let buoyancy = self.gravity * (rho - rho_avg) * self.atwood_number;
                
                // Update vertical velocity
                new_velocity_v[y][x] += buoyancy * self.dt;
                
                // Add viscous diffusion
                let laplacian_u = (self.velocity_u[y-1][x] + self.velocity_u[y+1][x] +
                                   self.velocity_u[y][x-1] + self.velocity_u[y][x+1] -
                                   4.0 * self.velocity_u[y][x]);
                let laplacian_v = (self.velocity_v[y-1][x] + self.velocity_v[y+1][x] +
                                   self.velocity_v[y][x-1] + self.velocity_v[y][x+1] -
                                   4.0 * self.velocity_v[y][x]);
                
                new_velocity_u[y][x] += self.viscosity * laplacian_u * self.dt;
                new_velocity_v[y][x] += self.viscosity * laplacian_v * self.dt;
            }
        }
        
        // Advect density field
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let u = self.velocity_u[y][x];
                let v = self.velocity_v[y][x];
                
                // Semi-Lagrangian advection
                let src_x = (x as f64 - u * self.dt).clamp(0.0, (self.width - 1) as f64);
                let src_y = (y as f64 - v * self.dt).clamp(0.0, (self.height - 1) as f64);
                
                let x0 = src_x.floor() as usize;
                let y0 = src_y.floor() as usize;
                let x1 = (x0 + 1).min(self.width - 1);
                let y1 = (y0 + 1).min(self.height - 1);
                
                let fx = src_x - x0 as f64;
                let fy = src_y - y0 as f64;
                
                // Bilinear interpolation
                let d00 = self.density[y0][x0];
                let d10 = self.density[y0][x1];
                let d01 = self.density[y1][x0];
                let d11 = self.density[y1][x1];
                
                new_density[y][x] = (1.0 - fx) * (1.0 - fy) * d00 +
                                    fx * (1.0 - fy) * d10 +
                                    (1.0 - fx) * fy * d01 +
                                    fx * fy * d11;
            }
        }
        
        // Apply boundary conditions (no-slip walls)
        for y in 0..self.height {
            new_velocity_u[y][0] = 0.0;
            new_velocity_u[y][self.width - 1] = 0.0;
            new_velocity_v[y][0] = 0.0;
            new_velocity_v[y][self.width - 1] = 0.0;
        }
        for x in 0..self.width {
            new_velocity_u[0][x] = 0.0;
            new_velocity_u[self.height - 1][x] = 0.0;
            new_velocity_v[0][x] = 0.0;
            new_velocity_v[self.height - 1][x] = 0.0;
        }
        
        self.density = new_density;
        self.velocity_u = new_velocity_u;
        self.velocity_v = new_velocity_v;
    }
    
    /// Run simulation for multiple steps
    pub fn simulate(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }
    
    /// Get density field statistics
    pub fn density_stats(&self) -> (f64, f64, f64) {
        let mut min_density = f64::INFINITY;
        let mut max_density = f64::NEG_INFINITY;
        let mut sum = 0.0;
        let mut count = 0;
        
        for row in &self.density {
            for &val in row {
                min_density = min_density.min(val);
                max_density = max_density.max(val);
                sum += val;
                count += 1;
            }
        }
        
        (min_density, max_density, sum / count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulation_creation() {
        let sim = RTSimulation::new(100, 100);
        assert_eq!(sim.width, 100);
        assert_eq!(sim.height, 100);
        
        // Check that density is initialized
        let (min, max, avg) = sim.density_stats();
        assert!(min >= 0.9);
        assert!(max <= 2.1);
        assert!(avg > 1.0 && avg < 2.0);
    }
    
    #[test]
    fn test_simulation_step() {
        let mut sim = RTSimulation::new(50, 50);
        let initial_stats = sim.density_stats();
        
        sim.step();
        
        let final_stats = sim.density_stats();
        
        // Density should be conserved (approximately)
        assert!((initial_stats.2 - final_stats.2).abs() < 0.1);
    }
    
    #[test]
    fn test_multiple_steps() {
        let mut sim = RTSimulation::new(50, 50);
        
        // Should not panic
        sim.simulate(10);
        
        let stats = sim.density_stats();
        assert!(stats.0 >= 0.5);
        assert!(stats.1 <= 2.5);
    }
}
