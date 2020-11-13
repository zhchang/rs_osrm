use crate::general::c_string_to_option_string;
use crate::general::c_string_to_string;
use crate::general::CGeneralOptions;
use crate::general::COsrmRouteLeg;
use crate::general::Coordinate;
use crate::general::GeneralOptions;
use crate::general::RouteLeg;
use crate::route::AnnotationsType;
use crate::route::GeometriesType;
use crate::route::OverviewType;
use crate::Boolean;
use crate::Osrm;
use crate::Status;
use core::ffi::c_void;
use std::os::raw::c_char;
use std::os::raw::c_double;
use std::{ffi::CStr, os::raw::c_int, slice};

#[link(name = "c_osrm")]
extern "C" {
    fn match_result_destroy(result: *mut CMatchResult);

    fn osrm_match(
        osrm: *mut c_void,
        request: *mut CMatchRequest,
        result: *mut *mut CMatchResult,
    ) -> Status;
}

#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct CMatchWaypoint {
    hint: *const c_char,
    distance: c_double,
    name: *const c_char,
    location: [c_double; 2],
    matchings_index: c_int,
    waypoint_index: c_int,
    alternatives_count: c_int,
    snap_failed: c_int,
}

#[derive(Debug)]
pub struct MatchWaypoint {
    pub hint: Option<String>,
    pub distance: f64,
    pub name: String,
    pub location: [f64; 2],
    pub matchings_index: i32,
    pub waypoint_index: i32,
    pub alternatives_count: i32,
}

impl MatchWaypoint {
    pub(crate) fn new(c_match: &CMatchWaypoint) -> MatchWaypoint {
        MatchWaypoint {
            hint: c_string_to_option_string(c_match.hint),
            distance: c_match.distance,
            name: c_string_to_string(c_match.name),
            location: c_match.location,
            matchings_index: c_match.matchings_index,
            waypoint_index: c_match.waypoint_index,
            alternatives_count: c_match.alternatives_count,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct CMatchRoute {
    pub(crate) duration: c_double,
    pub(crate) distance: c_double,
    pub(crate) weight_name: *const c_char,
    pub(crate) weight: c_double,
    pub(crate) geometry: *const c_char,
    pub(crate) legs: *const COsrmRouteLeg,
    pub(crate) number_of_legs: c_int,
    pub(crate) confidence: c_double,
}

#[derive(Debug)]
pub struct MatchRoute {
    pub duration: f64,
    pub distance: f64,
    pub weight_name: Option<String>,
    pub weight: f64,
    pub geometry: Option<String>,
    pub legs: Vec<RouteLeg>,
    pub number_of_legs: i32,
    pub confidence: f64,
}

impl MatchRoute {
    pub(crate) fn new(c_route: &CMatchRoute) -> MatchRoute {
        let mut weight_name: Option<String> = None;
        println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute");
        if c_route.weight_name != std::ptr::null() {
            println!(
                "inside rsc_osrm | MatchRoute::new to convert CMatchRoute | before weight_name {:?}", c_route.weight_name
            );
            weight_name = Option::from(c_string_to_string(c_route.weight_name));
            println!(
                "inside rsc_osrm | MatchRoute::new to convert CMatchRoute | after weight_name"
            );
        }

        let mut geometry: Option<String> = None;
        if c_route.geometry != std::ptr::null() {
            println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | before geometry");
            geometry = Option::from(c_string_to_string(c_route.geometry));
            println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | after geometry");
        }

        let mut legs: Vec<RouteLeg> = Vec::new();

        if c_route.legs != std::ptr::null_mut() {
            println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | before legs");
            let legs_vec = unsafe {
                slice::from_raw_parts(c_route.legs, c_route.number_of_legs as usize).to_vec()
            };
            println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | after legs from raw parts");

            for leg in legs_vec {
                println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | before legs to route leg | {:?}", leg );
                legs.push(leg.to_route_leg());
                println!("inside rsc_osrm | MatchRoute::new to convert CMatchRoute | after legs to route leg ");
            }
        }

        MatchRoute {
            duration: c_route.duration,
            distance: c_route.distance,
            weight_name,
            weight: c_route.weight,
            geometry,
            legs: legs,
            number_of_legs: 0,
            confidence: c_route.confidence,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub enum Annotations {
    NONE = 0,
    DURATION = 1,
    DISTANCE = 2,
    ALL = 3,
}

#[repr(C)]
#[derive(Clone)]
pub enum FallbackCoordinate {
    INPUT = 0,
    SNAPPED = 1,
}

#[repr(C)]
#[derive(Clone)]
pub enum Gap {
    Split = 0,
    Ignore = 1,
}

#[repr(C)]
struct CMatchRequest {
    general_options: CGeneralOptions,
    steps: Boolean,
    geometries: GeometriesType,
    annotations: Boolean,
    annotations_type: AnnotationsType,
    overview: OverviewType,
    timestamps: *const c_int,
    gaps: Gap,
    tidy: Boolean,
    waypoints: *const c_int,
    number_of_waypoints: c_int,
}

impl CMatchRequest {
    fn new(request: &mut MatchRequest) -> CMatchRequest {
        let mut c_request = CMatchRequest {
            general_options: CGeneralOptions::new(&mut request.general_options),
            steps: Boolean::from(request.steps),
            geometries: request.geometries.clone(),
            annotations: Boolean::from(request.annotations),
            annotations_type: request.annotations_type.clone(),
            overview: request.overview.clone(),
            timestamps: std::ptr::null(),
            gaps: request.gaps.clone(),
            tidy: Boolean::from(request.tidy),
            waypoints: std::ptr::null(),
            number_of_waypoints: 0,
        };

        match &request.timestamps {
            Some(timestamps) => {
                c_request.timestamps = timestamps.as_ptr();
            }
            None => {}
        }

        match &request.waypoints {
            Some(waypoints) => {
                c_request.waypoints = waypoints.as_ptr();
                c_request.number_of_waypoints = waypoints.len() as c_int;
            }
            None => {}
        }

        c_request
    }
}

pub struct MatchRequest {
    general_options: GeneralOptions,
    steps: bool,
    geometries: GeometriesType,
    annotations: bool,
    annotations_type: AnnotationsType,
    overview: OverviewType,
    timestamps: Option<Vec<i32>>,
    gaps: Gap,
    tidy: bool,
    waypoints: Option<Vec<i32>>,
}

impl MatchRequest {
    pub fn new(coordinates: &Vec<Coordinate>) -> MatchRequest {
        MatchRequest {
            general_options: GeneralOptions::new(coordinates),
            steps: false,
            geometries: GeometriesType::Polyline,
            annotations: false,
            annotations_type: AnnotationsType::None,
            overview: OverviewType::Simplified,
            timestamps: None,
            gaps: Gap::Split,
            tidy: false,
            waypoints: None,
        }
    }

    pub fn radiuses(&mut self, val: &Option<Vec<Option<f64>>>) -> &mut MatchRequest {
        self.general_options.radiuses(val);
        self
    }

    pub fn timestamps(&mut self, val: &Option<Vec<i32>>) -> &mut MatchRequest {
        self.timestamps = val.clone();
        self
    }

    pub fn run(&mut self, osrm: &Osrm) -> (Status, MatchResult) {
        unsafe {
            let mut result: *mut CMatchResult = std::ptr::null_mut();
            let result_ptr: *mut *mut CMatchResult = &mut result;

            let status = osrm_match(
                *osrm.config,
                &mut CMatchRequest::new(self) as *mut CMatchRequest,
                result_ptr,
            );
            println!("inside rsc_osrm: after osrm match: {:?}", &(*result));

            let converted_result = MatchResult::new(&(*result));
            println!(
                "inside rsc_osrm: after convert result: {:?}",
                converted_result
            );

            match_result_destroy(result);
            println!("inside rsc_osrm: after destroy result");

            (status, converted_result)
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct CMatchResult {
    code: *const c_char,
    message: *const c_char,
    waypoints: *const CMatchWaypoint,
    number_of_waypoints: c_int,
    routes: *const CMatchRoute,
    number_of_routes: c_int,
}

#[derive(Debug)]
pub struct MatchResult {
    pub code: Option<String>,
    pub message: Option<String>,
    pub tracepoints: Vec<Option<MatchWaypoint>>,
    pub matchings: Vec<MatchRoute>,
}

impl MatchResult {
    fn new(c_reasult: &CMatchResult) -> MatchResult {
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

        let mut waypoints: Vec<Option<MatchWaypoint>> = Vec::new();
        if c_reasult.waypoints != std::ptr::null_mut() {
            let waypoints_vec = unsafe {
                println!("inside rsc_osrm: before slice::from_raw_parts");
                let temp = slice::from_raw_parts(
                    c_reasult.waypoints,
                    c_reasult.number_of_waypoints as usize,
                )
                .to_vec();
                println!("inside rsc_osrm: after slice::from_raw_parts");
                temp
            };

            for waypoint in &waypoints_vec {
                println!("inside rsc_osrm: before MatchWaypoint::new: {:?}", waypoint);
                match waypoint.snap_failed {
                    0 => {
                        waypoints.push(Some(MatchWaypoint::new(waypoint)));
                    }
                    _ => {
                        waypoints.push(None);
                    }
                }
                println!("inside rsc_osrm: after MatchWaypoint::new");
            }
        }

        println!("inside rsc_osrm: before Vec<MatchRoute>::new");
        let mut routes: Vec<MatchRoute> = Vec::new();
        if c_reasult.routes != std::ptr::null_mut() {
            let routes_vec = unsafe {
                println!(
                    "inside rsc_osrm: before MatchRoute | slice::from_raw_parts | c_reasult {:?} ",
                    c_reasult
                );
                let temp =
                    slice::from_raw_parts(c_reasult.routes, c_reasult.number_of_routes as usize)
                        .to_vec();
                println!("inside rsc_osrm: after MatchRoute | slice::from_raw_parts");
                temp
            };

            for route in routes_vec {
                println!(
                    "inside rsc_osrm: before MatchRoute::new | route: {:?}",
                    &route
                );
            }

            for route in routes_vec {
                println!(
                    "inside rsc_osrm: before MatchRoute::new | route: {:?}",
                    &route
                );
                routes.push(MatchRoute::new(&route));
                println!(
                    "inside rsc_osrm: after MatchRoute::new | route: {:?}",
                    &route
                );
            }
        }

        MatchResult {
            code,
            message,
            tracepoints: waypoints,
            matchings: routes,
        }
    }
}
