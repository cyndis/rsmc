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
