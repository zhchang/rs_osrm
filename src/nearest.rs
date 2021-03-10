#![allow(dead_code)]

use crate::general::CGeneralOptions;
use crate::general::Coordinate;
use crate::general::GeneralOptions;
use crate::{Osrm, Status};
use core::slice;
use std::borrow::ToOwned;
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_double, c_int, c_longlong};
use std::ptr::null;

#[link(name = "c_osrm")]
extern "C" {
    fn nearest_result_destroy(result: *mut CNearestResult);

    fn osrm_nearest(
        osrm: *mut c_void,
        request: *mut CNearestRequest,
        result: *mut *mut CNearestResult,
    ) -> Status;
}

#[repr(C)]
#[derive(Clone)]
pub struct CNearestWaypoint {
    nodes: [c_longlong; 2],
    hint: *const c_char,
    distance: c_double,
    name: *const c_char,
    location: [c_double; 2],
}

pub struct NearestWaypoint {
    pub nodes: [i64; 2],
    pub hint: Option<String>,
    pub distance: f64,
    pub name: String,
    pub location: [f64; 2],
}

impl NearestWaypoint {
    pub fn new(c_waypoints: &CNearestWaypoint) -> NearestWaypoint {
        let mut hint: Option<String> = None;
        if c_waypoints.hint != null() {
            let c_hint_buf: *const c_char = c_waypoints.hint;
            let c_hint_str: &CStr = unsafe { CStr::from_ptr(c_hint_buf) };
            let hint_str_slice: &str = c_hint_str.to_str().unwrap();
            hint = Option::from(hint_str_slice.to_owned());
        }

        let c_name_buf: *const c_char = c_waypoints.name;
        let c_name_str: &CStr = unsafe { CStr::from_ptr(c_name_buf) };
        let name_str_slice: &str = c_name_str.to_str().unwrap();

        NearestWaypoint {
            nodes: c_waypoints.nodes,
            hint,
            distance: c_waypoints.distance,
            name: name_str_slice.to_owned(),
            location: c_waypoints.location,
        }
    }
}

#[repr(C)]
pub struct CNearestResult {
    code: *mut c_char,
    message: *mut c_char,
    waypoints: *mut CNearestWaypoint,
    number_of_waypoints: c_int,
}

pub struct NearestResult {
    pub code: Option<String>,
    pub message: Option<String>,
    pub waypoints: Option<Vec<NearestWaypoint>>,
}

impl NearestResult {
    //noinspection RsBorrowChecker
    pub fn new(c_reasult: &CNearestResult) -> NearestResult {
        let mut code: Option<String> = None;
        if c_reasult.code != std::ptr::null_mut() {
            let c_code_buf: *const c_char = c_reasult.code;
            let c_code_str: &CStr = unsafe { CStr::from_ptr(c_code_buf) };
            let code_str_slice: &str = c_code_str.to_str().unwrap();
            code = Option::from(code_str_slice.to_owned());
        }

        let mut message: Option<String> = None;
        if c_reasult.message != std::ptr::null_mut() {
            let c_message_buf: *const c_char = c_reasult.message;
            let c_message_str: &CStr = unsafe { CStr::from_ptr(c_message_buf) };
            let message_str_slice: &str = c_message_str.to_str().unwrap();
            message = Option::from(message_str_slice.to_owned());
        }

        let mut waypoints: Option<Vec<NearestWaypoint>> = None;

        if c_reasult.waypoints != std::ptr::null_mut() {
            let test_vec = unsafe {
                slice::from_raw_parts(c_reasult.waypoints, c_reasult.number_of_waypoints as usize)
                    .to_vec()
            };

            let mut rs_vec = Vec::new();
            for waypoint in &test_vec {
                rs_vec.push(NearestWaypoint::new(waypoint));
            }

            waypoints = Option::from(rs_vec);
        }

        NearestResult {
            code,
            message,
            waypoints,
        }
    }
}

#[repr(C)]
struct CNearestRequest {
    general_options: CGeneralOptions,
    number_of_results: c_int,
}

impl CNearestRequest {
    fn new(request: &mut NearestRequest) -> CNearestRequest {
        CNearestRequest {
            general_options: CGeneralOptions::new(&mut request.general_options),
            number_of_results: request.number_of_results as c_int,
        }
    }
}

pub struct NearestRequest {
    pub general_options: GeneralOptions,
    pub number_of_results: i32,
}

impl NearestRequest {
    pub fn new(latitude: f64, longitude: f64) -> NearestRequest {
        NearestRequest {
            general_options: GeneralOptions::new(&vec![Coordinate {
                latitude,
                longitude,
                name: None
            }]),
            number_of_results: 1,
        }
    }

    pub fn run(&mut self, osrm: &Osrm) -> (Status, NearestResult) {
        unsafe {
            let mut result: *mut CNearestResult = std::ptr::null_mut();
            let result_ptr: *mut *mut CNearestResult = &mut result;

            let status = osrm_nearest(
                *osrm.config,
                &mut CNearestRequest::new(self) as *mut CNearestRequest,
                result_ptr,
            );

            let converted_result = NearestResult::new(&(*result));

            nearest_result_destroy(result);

            (status, converted_result)
        }
    }
}
