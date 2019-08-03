use super::super::config::*;
use super::super::filter;
use super::super::geometry::Rectangle;

use super::setup;
use setup::generate_linear_track;

use super::structs::{KFData, Residuals, State};

use filter::{constant_magnetic_field, linear, utils::SuperData};

use rand::rngs::SmallRng;
use rand::{thread_rng, SeedableRng};
use rand_distr::Normal;

use itertools::izip;

use rayon::{self, prelude::*};

/// Runs batches of kf calculations. Parallelization happens upstream
pub fn collect_stats(state: &State) -> Vec<(KFData<Rectangle>, SuperData)> {
    // ) -> () {
    let uncertainties = &state.stdevs;

    let diagonal_rng = Normal::new(uncertainties.diag_mean, uncertainties.diag_std).unwrap();
    let corner_rng = Normal::new(uncertainties.corner_mean, uncertainties.corner_std).unwrap();

    let mut base_rng = thread_rng();
    let small_rngs_iterator = std::iter::repeat(())
        .map(|_| SmallRng::from_rng(&mut base_rng).unwrap())
        .take(state.iterations);

    // create iterators of repetitve values
    take! {state.iterations;
        num_sensors, state.num_sensors,
        distances, state.sensor_distance,
        angles, state.angles,
        point_std, state.stdevs.point_std
    }

    // zip the iterators together
    let iter = izip! {num_sensors, distances, angles, point_std, small_rngs_iterator};
    // let iter = iter.collect::<Vec<_>>();

    // if we use non-linear tracks
    let b_field_calculations: bool = if state.b_field != Vec3::zeros() {
        true
    } else {
        false
    };

    if b_field_calculations {
        print!("doing runge kutta")
    }

    let kf_results_vec: Vec<(KFData<Rectangle>, SuperData)> = iter
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|(num_sensor, sensor_distance, angles, std_dev, rng)| {
            // println!{"finish one"}

            if b_field_calculations {
                stats_const_b(&state, rng)
            } else {
                stats_linear(&state, rng)
            }
        })
        .collect();

    kf_results_vec
}

fn stats_linear(state: &State, mut rng: SmallRng) -> (KFData<Rectangle>, SuperData) {
    let data = setup::generate_linear_track(&state, rng);
    let kf_outs = linear::run(
        &data.start,
        &data.cov,
        &data.smear_hits,
        &data.sensors,
        Some(&data.smear_initial_vector),
    );
    (data, kf_outs)
}

fn stats_const_b(state: &State, mut rng: SmallRng) -> (KFData<Rectangle>, SuperData) {
    let data = setup::generate_const_b_track(&state, rng);
    let kf_outs = constant_magnetic_field::run(
        &data.start,
        &data.cov,
        &data.smear_hits,
        &data.sensors,
        Some(&data.smear_initial_vector),
        &state.b_field,
    );
    (data, kf_outs)
}

/// Parallelizes calculating the difference between the truth value
/// of a point versus the kf predicted / filtered / smoothed value
/// of that point. The data from `collect_stats` can be directly
/// piped into this function
pub fn fetch_kf_residuals_all(
    create_statistics_data: &Vec<(KFData<Rectangle>, SuperData)>,
) -> Vec<(Vec<Vec2>, Vec<Vec2>, Vec<Vec2>)> {

    create_statistics_data
        .iter()
        .map(|(truth, kf_ver)| create_residuals(truth, kf_ver, false))
        .collect::<Vec<_>>()
}

/// Handles calculating all residuals of the truth hits vs KF outputs
/// and returns a struct of all smoothed / filtered / predicted residuals
fn create_residuals(
    truth_data: &KFData<Rectangle>,
    kf_data: &SuperData,
    pull_distr: bool,
) -> (Vec<Vec2>, Vec<Vec2>, Vec<Vec2>) {

    let truth_points = &truth_data.truth_hits;

    let len = kf_data.smth.state_vec.len();

    // predicted
    let pred_state_vec = &kf_data.pred.state_vec;
    let pred_res = calc_residual(pred_state_vec, truth_points, pull_distr);

    // filtered
    let filt_state_vec = &kf_data.filt.state_vec;
    let filt_res = calc_residual(filt_state_vec, truth_points, pull_distr);

    // smoothed
    let smth_state_vec = &kf_data.smth.state_vec;
    let smth_resid = calc_residual(smth_state_vec, truth_points, pull_distr);

    (pred_res, filt_res, smth_resid)
}

/// Calculates the residuals between truth points and their
/// smeared counterparts
pub fn smear_residuals(kf_data: &KFData<Rectangle>) -> Vec<Vec2> {
    let smears = &kf_data.smear_hits;
    let truths = &kf_data.truth_hits;

    truths
        .iter()
        .zip(smears.iter())
        .map(|(t, s)| t - s)
        .collect::<Vec<Vec2>>()
}

pub fn truth_kf_output_residuals(output: Vec<(KFData<Rectangle>, SuperData)>) -> Vec<Residuals> {
    //

    output
        .into_iter()
        .map(|(truth_data, kf_out)| {
            let truth_vals = truth_data.truth_hits.into_iter();
            let prediction = vec5_to_vec2_all(&kf_out.pred.state_vec).into_iter();
            let filtered = vec5_to_vec2_all(&kf_out.filt.state_vec).into_iter();
            let smoothed = vec5_to_vec2_all(&kf_out.smth.state_vec).into_iter();

            // truth_vals.zip(prediction);
            let zipped_iter = izip! {truth_vals, prediction, filtered, smoothed};

            let grouped_residuals = zipped_iter
                .map(|(truth, pred, filt, smth)| {
                    let p_ = truth - pred;
                    let f_ = truth - filt;
                    let s_ = truth - smth;

                    (s_, f_, p_)
                })
                .collect::<Vec<_>>();

            Residuals::new_grouped(grouped_residuals)
        })
        .collect::<Vec<_>>()
}

fn vec5_to_vec2_all(vector_sv: &Vec<Vec5>) -> Vec<Vec2> {
    vector_sv
        .into_iter()
        .map(|x| vec5_to_vec2_one(&x))
        .collect::<Vec<_>>()
}

fn vec5_to_vec2_one(vec: &Vec5) -> Vec2 {
    // let new_vec = Vec2::zeros();
    get_unchecked! {vector;vec;
        eLOC_0 => x,
        eLOC_1 => y
    }
    Vec2::new(*x, *y)
}

/// Residual between KF outputs and truth hits
fn calc_residual(state_vectors: &Vec<Vec5>, truth_points: &Vec<Vec2>, pull_distr: bool) -> Vec<Vec2> {
    let len = truth_points.len();
    let mut diff_vec = Vec::with_capacity(len);

    for i in 0..len {
        get_unchecked! {
            state_vectors[i] => curr_state_vec,
            truth_points[i] => curr_truth_point
        }

        let kf_hit = vec5_to_vec2_one(curr_state_vec);

        let diff = kf_hit - curr_truth_point;
        diff_vec.push(diff);
    }

    return diff_vec;
}
