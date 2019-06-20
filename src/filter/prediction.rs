use nalgebra as na;
use super::super::config::*;

// extrapolating state vector
pub fn state_vector (jacobian: &Mat5, state_vector: &Vec5) -> Vec5 {
    return jacobian * state_vector
}

// extrapolation of the covariance matrix 
pub fn covariance_matrix(jacobian: &Mat5, previous_covariance: &Mat5)-> Mat5{
    return jacobian * previous_covariance * jacobian.transpose()
}

// just below eq. 7
// covariance of predicted results
pub fn residual_mat(V: &Mat2, H: &Mat2x5, C: &Mat5) -> Mat2 {
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