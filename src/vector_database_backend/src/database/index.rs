use crate::config::EMBEDDING_LENGTH;
use instant_distance::{Builder, HnswMap, Point};
use na::SVector;
use nalgebra::ComplexField;

pub fn generate_index(points: Vec<Vector>, values: Vec<String>) -> HnswMap<Vector, String> {
    Builder::default().build(points, values)
}

#[derive(Copy, Clone, Debug)] 
pub struct Vector {
    pub data: SVector<f64, EMBEDDING_LENGTH>
}

impl instant_distance::Point for Vector { 
    fn distance(&self, other: &Self) -> f32 {
        let diff = self.data - other.data;
        diff.dot(&diff).norm1() as f32 
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data 
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other) 
    }
}

impl From<Vec<f64>> for Vector {
    fn from(value: Vec<f64>) -> Self {
        let svec = SVector::from_vec(value);
        Vector { data:  svec}
    }
}

impl Vector {
    pub fn cos_sim(&self, other: &Vector) -> f64 {
        self.data.dot(&other.data) / (self.data.norm() * other.data.norm())
    }
}