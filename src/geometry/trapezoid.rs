extern crate nalgebra as na;
use na::{Point3, Matrix4 as Matrix, Vector3 as Vector};
use super::traits::{self, Transform, Plane};
use super::utils;

use super::super::config::*;

/// A struct for sensors of trapezoidal geometry
pub struct Trapezoid{
    points : [P3; 4],
    normal: Option<Vec3>, // the normal vector is not initially calculated
    tfm: Aff3
}

impl Trapezoid{
    /// This is the constructor for the rectangular geometry. It expects a 4x4 `nalgebra::Matrix4<f64>` that is invertible 
    /// and a 4 element array of `nalgebra::Point3<f64>`. If the matrix is not invertible it will return `Err(&str)`.
    /// The provided matrix should be an affine transformation for converting from R2->R3
    /// 
    /// # Examples
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor = kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// ```
    pub fn new(mut points: [P3; 4], tfm_matrix: Mat4) -> Result<Trapezoid, &'static str>{
        
        let affine_transform = Aff3::from_matrix_unchecked(tfm_matrix);

        match affine_transform.try_inverse(){
            Some(_x) => {
                let points = utils::organize_points(&mut points);
                Ok(Trapezoid{points: *points, normal:None, tfm: affine_transform})},
            None => return Err("matrix was not invertable")

        }
    }
}


impl Transform for Trapezoid{
    /// Converts a point in the global reference frame to a point in the local reference frame of the sensor.
    /// 
    /// # Examples
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// use kalman_rs::sensor_traits::Transform;
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor = kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// 
    /// let global_point = trap_sensor.to_global(na::Point3::new(1.0, 2.0, 0.0));
    /// ```
    fn to_global(&self, input_point: P3)-> P3{
        return self.tfm * input_point;
    }
    
    /// Converts a point in the local refernce frame of the sensor to the global reference frame.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// use kalman_rs::sensor_traits::Transform;
    /// 
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor = kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// 
    /// let local_point = trap_sensor.to_local(na::Point3::new(4.0, 5.0, 6.0));
    /// ```
    fn to_local(&self, input_point: P3) -> P3{
        self.tfm.inverse() * input_point
    }


    /// Checks if a local point is contained within the bounds of a sensor.
    /// NOTE: `plane()` must be called before checking for bounds of the sensor since the normal 
    /// vector must be calculated first. 
    /// # Examples
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// use kalman_rs::sensor_traits::Transform;
    /// 
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor = kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// 
    /// let is_point_on_sensor = trap_sensor.contains_from_local(&na::Point3::new(1.0, 6.0, 0.0));
    /// ```
    fn contains_from_local(&self, input: &P3) ->Result<bool, &'static str> {
        let xy_contains = utils::quadralateral_contains(&self.points, &input);
        let z_contains = self.on_plane(&input); 

        match z_contains{
            Ok(x) =>{
                if xy_contains && x {
                    return Ok(true)
                }
                Ok(false)
            },
            Err(x) => return Err(x)
        }
    }
}

impl Plane for Trapezoid{
    /// Calculate the current normal vector of the plane of the surface.
    /// NOTE: `plane()` must be called before `contains_from_local` since `contains_from_local`
    /// requires the normal vector to be defined
    /// # Examples
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// use kalman_rs::sensor_traits::Plane;
    /// 
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor = kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// 
    /// let normal_vector = trap_sensor.plane();
    /// ```
    fn plane(&mut self) -> Vec3{
        // calculate the normal vector of the surface if it has not been calculated before
        // if it has been calculated return the original calculation
        // this would need to change if the suface moves 
        match self.normal{
            Some(x)=>x,
            None =>{
                let normal_vector = utils::plane_normal_vector(&self.points[0], &self.points[1], &self.points[2]);
                self.normal = Some(normal_vector);
                normal_vector
            },
        }
    }
    /// Check if a given point is located on the same plane as the sensor
    /// NOTE: `plane()` must be called becuase the normal vector is not currently known
    /// # Examples
    /// ```
    /// use nalgebra as na;
    /// use na::Point3;
    /// use kalman_rs::sensor_traits::Plane;
    /// 
    /// let trapezoid_points = [Point3::new(0.0, 0.0, 0.0), 
    ///                         Point3::new(5.0,1.0,0.0), 
    ///                         Point3::new(5.0, 9.0,0.0), 
    ///                         Point3::new(0.0,10.0,0.0)];
    /// let tfm_matrix : na::Matrix4<f64>= na::Matrix4::new(1.0,5.0,7.0,2.0,  3.0,5.0,7.0,4.0,  8.0,4.0,1.0,9.0, 2.0,6.0,4.0,8.0);
    /// let mut trap_sensor =kalman_rs::Trapezoid::new(trapezoid_points, tfm_matrix).unwrap();
    /// 
    /// let on_sensor_plane = trap_sensor.on_plane(&Point3::new(1.0, 1.0, 0.0)); //true
    /// ```
    fn on_plane(&self, input_point: &P3) -> Result<bool, &'static str> {
        let pv = utils::vector3_from_points(&self.points[0], &input_point);
        match self.normal{
            Some(x) => {
                if x.dot(&pv) ==0.0 {
                return Ok(true)
                }
                Ok(false)
            }
            None => Err("self.plane() method must be called before normal vector can be used")
        }

    }
}
