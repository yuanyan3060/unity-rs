#[derive(Default, Debug, Copy, Clone)]
pub struct RectF32 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&mut self) {
        let length = self.length();
        if length > 0.00001 {
            let inv_norm = 1.0 / length;
            self.x *= inv_norm;
            self.y *= inv_norm;
            self.z *= inv_norm;
        } else {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Matrix4x4 {
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

impl Matrix4x4 {
    pub fn from_array(array: [f32; 16]) -> Self {
        Self {
            m00: array[0],
            m10: array[1],
            m20: array[2],
            m30: array[3],
            m01: array[4],
            m11: array[5],
            m21: array[6],
            m31: array[7],
            m02: array[8],
            m12: array[9],
            m22: array[10],
            m32: array[11],
            m03: array[12],
            m13: array[13],
            m23: array[14],
            m33: array[15],
        }
    }
}

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_array(array: [f32; 4]) -> Self {
        Self::new(array[0], array[1], array[2], array[3])
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn from_array(array: [f32; 4]) -> Self {
        Self::new(array[0], array[1], array[2], array[3])
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}
