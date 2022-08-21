use std::mem;

use bevy::prelude::Vec3;

use super::AABB;

/// intersect ray (p point,d direction) and AABB (min, max points)
///
/// return the intersection point and the `t` value where lerp(p, p+d, t) = intersection point
///
/// for segments: check if t is in [0‥1]
pub fn ray_aabb(
    [p, d]: [Vec3; 2],
    [min, max]: [Vec3; 2],
) -> Option<(Vec3, f32)> {
    let mut tmin = 0.0f32;
    let mut tmax = std::f32::MAX;

    for i in 0..3 {
        if d[i].abs() < std::f32::EPSILON {
            // ray is parallel to slab
            // no hit if origin not within the slab
            if p[i] < min[i] || p[i] > max[i] {
                return None;
            }
        } else {
            // compute intersection t value
            let ood = 1.0 / d[i];
            let mut t1 = (min[i] - p[i]) * ood;
            let mut t2 = (max[i] - p[i]) * ood;

            // make sure t1 < t2
            //
            if t1 > t2 {
                mem::swap(&mut t1, &mut t2)
            }

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            if tmin > tmax {
                return None;
            }
        }
    }

    Some((p + d * tmin, tmin))
}

pub fn aabb_aabb(a: &AABB, b: &AABB) -> bool {
    for i in 0..3 {
        if a.max[i] < b.min[i] || a.min[i] > b.max[i] {
            return false;
        }
    }
    true
}
