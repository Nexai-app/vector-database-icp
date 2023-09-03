use crate::config::EMBEDDING_LENGTH;
use instant_distance::{Builder, HnswMap};
use na::SVector;

pub fn generate_index(points: Vec<Vector>, values: Vec<String>) -> HnswMap<Vector, String> {
    Builder::default().build(points, values)
}

#[derive(Copy, Clone, Debug)] 
pub struct Vector {
    data: SVector<f32, EMBEDDING_LENGTH>
}

impl instant_distance::Point for Vector { 
    fn distance(&self, other: &Self) -> f32 {
        let dis = self.data - other.data;
        dis.dot(&dis)
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

impl From<Vec<f32>> for Vector {
    fn from(value: Vec<f32>) -> Self {
        let svec = SVector::from_vec(value);
        Vector { data:  svec}
    }
}

// impl Vector {
//     pub fn random() -> Self {
//         Vector { data: SVector::new_random() }
//     }
// }