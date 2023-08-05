#[derive(Default, Debug, Copy, Clone)]
pub struct RectF32{
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector2{
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector3{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector4{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Matrix4x4{
    pub m00: f32,
    pub m10: f32,
    pub m20: f32,
    pub m30: f32,

    pub m01: f32,
    pub m11: f32,
    pub m21: f32,
    pub m31: f32,

    pub m02: f32,
    pub m12: f32,
    pub m22: f32,
    pub m32: f32,

    pub m03: f32,
    pub m13: f32,
    pub m23: f32,
    pub m33: f32,
}