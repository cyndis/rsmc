use lmath;
pub use lmath::vec::BaseVec2;
pub use lmath::vec::BaseVec3;
pub use lmath::mat::BaseMat4;
pub use lmath::mat::BaseMat;
pub use lmath::quat::Quat;

pub type Vec2f = lmath::vec::Vec2<float>;
pub type Vec3f = lmath::vec::Vec3<float>;
pub type Mat4f = lmath::mat::Mat4<float>;
pub type Quatf = lmath::quat::Quat<float>;

pub trait TripleFloor<Out> {
    fn floor(&self) -> (Out,Out,Out);
}

impl TripleFloor<int> for (float,float,float) {
    fn floor(&self) -> (int,int,int) {
        let (x,y,z) = *self;
        (float::floor(x as f64) as int, float::floor(y as f64) as int, float::floor(z as f64) as int)
    }
}
