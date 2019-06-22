use nalgebra as na;
use super::super::config::*;
use super::super::error::*;
use super::super::geometry::traits::{Plane, Transform};

// extrapolating state vector
pub fn state_vector (
    jacobian: &Mat5, 
    state_vector: &Vec5
    ) -> Vec5 {

    return jacobian * state_vector
}


// prediction of covariance matrix C
pub fn covariance_matrix(
    jacobian: &Mat5, 
    previous_covariance: &Mat5
    )-> Mat5{
    return jacobian * previous_covariance * jacobian.transpose()
}

// just below eq. 7
// residual covariance of predicted results
pub fn residual_mat(
    V: &Mat2, 
    H: &Mat2x5, 
    C: &Mat5) -> Mat2 {
    return V + (H*C * H.transpose())
}

pub fn residual_vec(
    m_k: &Vec2,
    H_k: &Mat2x5,
    pred_state_vec: &Vec5) -> Vec2 {

    let prod = H_k * pred_state_vec;
    let diff = m_k - prod;

    return diff;
}

/// Calculates the predicted location of the hit on the following sensor
// based on this equation set https://i.imgur.com/mWC0qkj.png
fn linear_state_vector<T: Transform + Plane>(start_sensor: &T, 
                                        end_sensor: &T, 
                                        prev_filt_state_vec: &Vec5,
                                        phi: Real,
                                        theta: Real) -> Result<Vec5, SensorError> {
    
    get_unchecked!{
        prev_filt_state_vec[0] => start_local_x_hit,
        prev_filt_state_vec[1] => start_local_y_hit,
        prev_filt_state_vec[2] => theta,
        prev_filt_state_vec[3] => phi
    }

    let start_local_point = P3::new(*start_local_x_hit, *start_local_y_hit, 0.0);
    let start_global_point = start_sensor.to_global(start_local_point);

    let cos_phi = phi.cos();
    let x_slope = cos_phi * theta.cos();
    let y_slope = cos_phi * theta.sin();
    let z_slope = phi.sin();

    // used so we can be generic over planar sensors
    let normal = end_sensor.plane_normal_vec();

    // calculate a generic numerator used repetitively later
    let gen_num_1 = normal.x * start_global_point.x;
    let gen_num_2 = normal.y * start_global_point.y;
    let gen_num_3 = normal.z * start_global_point.z;
    let gen_num = gen_num_1 + gen_num_2 + gen_num_3;

    // generic denominator 
    let gen_den_1 = normal.x * x_slope;
    let gen_den_2 = normal.y * y_slope;
    let gen_den_3 = normal.z * z_slope;
    let gen_den = gen_den_1 + gen_den_2 + gen_den_3;

    let gen_division = gen_num / gen_den;

    // calculate predicted points of intersection on ending plane
    let pred_x = start_global_point.x - (x_slope * gen_division);
    let pred_y = start_global_point.y - (y_slope * gen_division);
    let pred_z = start_global_point.z - (z_slope * gen_division);

    let global_pred_point = P3::new(pred_x, pred_y, pred_z);
    let local_pred_point  = end_sensor.to_local(global_pred_point);

    // check if the predicted point is on the sensor
    if end_sensor.contains_from_local(&local_pred_point) {
        // might be able to avoid cloning here
        let mut new_state_vec = prev_filt_state_vec.clone();
        new_state_vec[0] =local_pred_point.x;
        new_state_vec[1] =local_pred_point.y; 

        Ok(new_state_vec)
    }
    else {
        Err(SensorError::OutsideSensorBounds)
    }
}