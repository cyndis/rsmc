pub use lmath::mat::*;
pub use lmath::vec::*;

pub use lmath::vec::Vec2f;
pub use lmath::vec::Vec3f;
pub use lmath::mat::Mat4f;
pub use lmath::quat::Quatf;

pub trait TripleFloor<Out> {
    fn floor(&self) -> (Out,Out,Out);
}

impl TripleFloor<int> for (float,float,float) {
    fn floor(&self) -> (int,int,int) {
        let (x,y,z) = *self;
        (float::floor(x as f64) as int, float::floor(y as f64) as int, float::floor(z as f64) as int)
    }
}

pub trait TripleFloat {
    fn to_float(&self) -> (float, float, float);
}

impl TripleFloat for (int, int, int) {
    fn to_float(&self) -> (float, float, float) {
        let (x, y, z) = *self;
        (x as float, y as float, z as float)
    }
}

pub trait MinIndex {
    fn min_index(&self) -> uint;
}

impl<'self, A: Copy + Ord> MinIndex for &'self [A] {
    fn min_index(&self) -> uint {
        let mut smallest: Option<(A, uint)> = None;

        for self.eachi |i, &v| {
            smallest = match (v, smallest) {
                (v, None) => Some((v, i)),
                (v, Some((xv, _xi))) if v < xv => Some((v, i)),
                (_, Some(x)) => Some(x)
            }
        }

        match smallest {
            Some((_v, i)) => i,
            _ => fail!(~"empty or something")
        }
    }
}
