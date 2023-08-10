use std::num::Wrapping;

pub(crate) fn down_scale_u16_to_u8(component: u16) -> u8 {
    let warp = Wrapping(component);
    (((warp * Wrapping(255)) + Wrapping(32895)) >> 16).0 as _
}