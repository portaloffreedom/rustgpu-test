#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::Image;
use spirv_std::glam::{uvec2, UVec2, UVec3};
use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::glam::Vec3Swizzles;

type Image2d = Image!(2D, type=f32, sampled=false, depth=false);

#[spirv(compute(threads(8,8)))]
pub fn fractal(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] _image: &mut Image2d,
    #[spirv(flat, descriptor_set = 0, binding = 1)] _dimensions: UVec2,
) {
    let dimensions: UVec2 = uvec2(1024, 1024);//image.query_size();
    let norm_coordinates: Vec2 = (id.xy().as_vec2() + vec2(0.5, 0.5)) / dimensions.as_vec2();
    let c: Vec2 = (norm_coordinates - vec2(0.5, 0.5)) * 2.0 - vec2(1.0, 0.0);

    let mut z: Vec2 = vec2(0.0, 0.0);
    let mut i: f32 = 0.0f32;
    while i<1.0 {
        z = vec2(
            z.x*z.x - z.y*z.y + c.x,
            z.y*z.y + z.x*z.x + c.y,
        );

        if z.length() > 4.0 {
            break;
        }
        
        i+= 0.005;
    }

    let _to_write: Vec4 = vec4(i, i, i, 1.0);
    // unsafe {
        // image.write(id.xy(), to_write);
    // }
}