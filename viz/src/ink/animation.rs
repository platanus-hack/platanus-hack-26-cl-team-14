//! Rigid part poses from Ink Studio's `K.rig.set` and `K.rig.apply`.
use super::sculpt::Splat;
use bevy::math::{DMat3, DVec3, Mat3, Vec3};

/// Rotate one baked part about its rest pivot and move that pivot to the target.
pub fn pose_part(
    rest: &[Splat],
    posed: &mut [Splat],
    rotation: DMat3,
    pivot: DVec3,
    target: DVec3,
) {
    // The JS rig stores each joint transform in a Float32Array before applying it.
    let translation = (target - rotation * pivot).as_vec3().as_dvec3();
    let rotation = rotation.as_mat3().as_dmat3();
    posed.copy_from_slice(rest);
    for splat in posed {
        let position = Vec3::from_array(splat.position).as_dvec3();
        splat.position = (rotation * position + translation).as_vec3().to_array();

        let [xx, xy, xz, yy, yz, zz] = splat.covariance;
        let covariance = Mat3::from_cols_array(&[xx, xy, xz, xy, yy, yz, xz, yz, zz]).as_dmat3();
        let rotated = (rotation * covariance * rotation.transpose()).as_mat3();
        splat.covariance = [
            rotated.x_axis.x,
            rotated.y_axis.x,
            rotated.z_axis.x,
            rotated.y_axis.y,
            rotated.z_axis.y,
            rotated.z_axis.z,
        ];
    }
}
