use ndarray::IxDyn;
use ndrustfft::{ndfft, ndfft_r2c, ndifft, ndifft_r2c, FftHandler, Normalization, R2cFftHandler};
use num::complex::Complex64;

use super::mesh::Mesh;
use crate::field::{CField, RField};

/// Wrapper for real-to-complex FFTs over a multi-dimensional array.
/// The real-to-complex transformation is performed over the last axis of the arrays.
pub struct FFT {
    mesh: Mesh,
    r2c_handler: R2cFftHandler<f64>,
    c2c_handlers: Vec<FftHandler<f64>>,
    // Work buffers with half-size last dimension
    work1: CField,
    work2: CField,
}

impl FFT {
    pub fn new(mesh: Mesh, normalization: Option<Normalization<Complex64>>) -> Self {
        let r2c_dim = mesh.dimensions().last().unwrap();
        let r2c_handler = match &normalization {
            Some(norm) => R2cFftHandler::<f64>::new(*r2c_dim).normalization(norm.clone()),
            None => R2cFftHandler::<f64>::new(*r2c_dim),
        };

        let c2c_handlers = mesh
            .dimensions()
            .iter()
            .take(mesh.n_dim() - 1)
            .map(|dim| match &normalization {
                Some(norm) => FftHandler::<f64>::new(*dim).normalization(norm.clone()),
                None => FftHandler::<f64>::new(*dim),
            })
            .collect();

        let k_mesh = mesh.to_complex();
        let work1 = CField::zeros(IxDyn(k_mesh.dimensions()));
        let work2 = CField::zeros(IxDyn(k_mesh.dimensions()));

        Self {
            mesh,
            r2c_handler,
            c2c_handlers,
            work1,
            work2,
        }
    }

    pub fn forward(&mut self, input: &RField, output: &mut CField) {
        // Transform the real -> complex dimension first
        let r2c_axis = self.mesh.n_dim() - 1;
        ndfft_r2c(input, output, &mut self.r2c_handler, r2c_axis);

        // Transform the remaining complex -> complex axes
        self.work1.assign(output);
        for (axis, handler) in self.c2c_handlers.iter_mut().enumerate() {
            ndfft(&self.work1, output, handler, axis);
            self.work1.assign(output);
        }
    }

    pub fn inverse(&mut self, input: &CField, output: &mut RField) {
        // Transform the complex -> complex axes first
        self.work1.assign(input);
        self.work2.assign(input);
        for (axis, handler) in self.c2c_handlers.iter_mut().enumerate() {
            ndifft(&self.work1, &mut self.work2, handler, axis);
            self.work1.assign(&self.work2);
        }

        // Transform the accumulated work into the output array
        let r2c_axis = self.mesh.n_dim() - 1;
        ndifft_r2c(&self.work2, output, &mut self.r2c_handler, r2c_axis);
    }
}