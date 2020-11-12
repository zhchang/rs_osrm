use crate::Boolean;
use core::slice;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_short};
use std::ptr::null;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Bearing {
    pub bearing: c_short,
    pub range: c_short,
}

#[repr(C)]
#[derive(Clone)]
pub enum Approach {
    UNRESTRICTED,
    CURB,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmCoordinate {
    pub(crate) latitude: c_double,
    pub(crate) longitude: c_double,
}

impl COsrmCoordinate {
    pub(crate) fn to_coordinate(&self) -> Coordinate {
        Coordinate {
            latitude: self.latitude,
            longitude: self.longitude,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinate {
    pub(crate) fn to_ccoordinate(&self) -> COsrmCoordinate {
        COsrmCoordinate {
            latitude: self.latitude,
            longitude: self.longitude,
        }
    }
}

pub(crate) fn to_vec_ccoordinate(coordinates: &Vec<Coordinate>) -> Vec<COsrmCoordinate> {
    let mut return_vec = Vec::new();
    for coordinate in coordinates {
        return_vec.push(coordinate.to_ccoordinate());
    }

    return_vec
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmLanes {
    pub(crate) indications: *const *const c_char,
    pub(crate) valid: Boolean,
}

impl COsrmLanes {
    pub(crate) fn to_lanes(&self) -> Lanes {
        Lanes {
            indications: Vec::new(),
            valid: false,
        }
    }
}

#[derive(Debug)]
pub struct Lanes {
    pub indications: Vec<String>,
    pub valid: bool,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmIntersections {
    pub(crate) location: COsrmCoordinate,
    pub(crate) bearings: *const c_int,
    pub(crate) number_of_bearings: c_int,
    pub(crate) classes: *const *const c_char,
    pub(crate) number_of_classes: c_int,
    pub(crate) entry: *const Boolean,
    pub(crate) number_of_entries: c_int,
    pub(crate) intersection_in: c_int,
    pub(crate) intersection_out: c_int,
    pub(crate) lanes: *const COsrmLanes,
    pub(crate) number_of_lanes: c_int,
}

impl COsrmIntersections {
    pub(crate) fn to_intersections(&self) -> Intersections {
        let mut intersection = Intersections {
            location: self.location.to_coordinate(),
            bearings: Vec::new(),
            classes: Vec::new(),
            entry: Vec::new(),
            intersection_in: self.intersection_in,
            intersection_out: self.intersection_out,
            lanes: Vec::new(),
        };

        if self.bearings != std::ptr::null_mut() {
            intersection.bearings = unsafe {
                slice::from_raw_parts(self.bearings, self.number_of_bearings as usize).to_vec()
            };
        }

        if self.classes != std::ptr::null_mut() {
            let classes_vec: Vec<*const c_char> = unsafe {
                slice::from_raw_parts(self.classes, self.number_of_classes as usize).to_vec()
            };

            for class in classes_vec {
                intersection.classes.push(c_string_to_string(class));
            }
        }

        if self.entry != std::ptr::null_mut() {
            let boolean_vec = unsafe {
                slice::from_raw_parts(self.entry, self.number_of_entries as usize).to_vec()
            };

            for class in boolean_vec {
                intersection.entry.push(class == Boolean::TRUE);
            }
        }

        if self.lanes != std::ptr::null_mut() {
            let lanes_vec = unsafe {
                slice::from_raw_parts(self.lanes, self.number_of_lanes as usize).to_vec()
            };

            for lane in &lanes_vec {
                intersection.lanes.push(lane.to_lanes());
            }
        }

        intersection
    }
}

#[derive(Debug)]
pub struct Intersections {
    pub location: Coordinate,
    pub bearings: Vec<i32>,
    pub classes: Vec<String>,
    pub entry: Vec<bool>,
    pub intersection_in: i32,
    pub intersection_out: i32,
    pub lanes: Vec<Lanes>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmManeuver {
    pub(crate) bearing_before: c_int,
    pub(crate) bearing_after: c_int,
    pub(crate) coordinate: COsrmCoordinate,
    pub(crate) maneuver_type: *const c_char,
    pub(crate) modifer: *const c_char,
}

impl COsrmManeuver {
    pub(crate) fn to_maneuver(&self) -> Maneuver {
        Maneuver {
            bearing_before: self.bearing_before,
            bearing_after: self.bearing_after,
            coordinate: self.coordinate.to_coordinate(),
            maneuver_type: c_string_to_string(self.maneuver_type),
            modifer: c_string_to_option_string(self.modifer),
        }
    }
}

#[derive(Debug)]
pub struct Maneuver {
    pub bearing_before: i32,
    pub bearing_after: i32,
    pub coordinate: Coordinate,
    pub maneuver_type: String,
    pub modifer: Option<String>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmStep {
    pub(crate) distance: c_double,
    pub(crate) duration: c_double,
    pub(crate) geometry: *const c_char,
    pub(crate) weight: c_double,
    pub(crate) name: *const c_char,
    pub(crate) reference: *const c_char,
    pub(crate) pronunciation: *const c_char,
    pub(crate) exits: *const c_char,
    pub(crate) mode: *const c_char,
    pub(crate) metadata: *const COsrmManeuver,
    pub(crate) intersections: *const COsrmIntersections,
    pub(crate) number_of_intersections: c_int,
    pub(crate) rotary_name: *const c_char,
    pub(crate) rotary_pronunciation: *const c_char,
    pub(crate) driving_side: *const c_char,
}

impl COsrmStep {
    pub(crate) fn to_step(&self) -> Step {
        let mut step = Step {
            distance: self.distance,
            duration: self.duration,
            geometry: c_string_to_option_string(self.geometry),
            weight: self.weight,
            name: c_string_to_option_string(self.name),
            reference: c_string_to_option_string(self.reference),
            pronunciation: c_string_to_option_string(self.pronunciation),
            exits: c_string_to_option_string(self.exits),
            mode: c_string_to_option_string(self.exits),
            metadata: None,
            intersections: Vec::new(),
            rotary_name: c_string_to_option_string(self.exits),
            rotary_pronunciation: c_string_to_option_string(self.exits),
            driving_side: c_string_to_option_string(self.exits),
        };

        if self.metadata != std::ptr::null_mut() {
            step.metadata = unsafe { Option::from((*self.metadata).to_maneuver()) };
        }

        if self.intersections != std::ptr::null_mut() {
            let intersections_vec = unsafe {
                slice::from_raw_parts(self.intersections, self.number_of_intersections as usize)
                    .to_vec()
            };

            for intersection in &intersections_vec {
                step.intersections.push(intersection.to_intersections());
            }
        }

        step
    }
}

#[derive(Debug)]
pub struct Step {
    pub distance: f64,
    pub duration: f64,
    pub geometry: Option<String>,
    pub weight: c_double,
    pub name: Option<String>,
    pub reference: Option<String>,
    pub pronunciation: Option<String>,
    pub exits: Option<String>,
    pub mode: Option<String>,
    pub metadata: Option<Maneuver>,
    pub intersections: Vec<Intersections>,
    pub rotary_name: Option<String>,
    pub rotary_pronunciation: Option<String>,
    pub driving_side: Option<String>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmMetaData {
    pub(crate) datasource_names: *const *const c_char,
    pub(crate) number_of_datasource_names: c_int,
}

impl COsrmMetaData {
    pub(crate) fn to_meta_data(&self) -> MetaData {
        let mut meta_data = MetaData {
            datasource_names: Vec::new(),
        };

        if self.datasource_names != std::ptr::null_mut() {
            let datasources_vec = unsafe {
                slice::from_raw_parts(
                    self.datasource_names,
                    self.number_of_datasource_names as usize,
                )
                .to_vec()
            };

            for datasource in datasources_vec {
                meta_data
                    .datasource_names
                    .push(c_string_to_string(datasource));
            }
        }

        meta_data
    }
}

#[derive(Debug)]
pub struct MetaData {
    datasource_names: Vec<String>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmAnnotation {
    pub(crate) duration: *const c_double,
    pub(crate) distance: *const c_double,
    pub(crate) speed: *const c_double,
    pub(crate) weight: *const c_double,
    pub(crate) nodes: *const i64,
    pub(crate) datasources: *const c_int,
    pub(crate) metadata: *const COsrmMetaData,
    pub(crate) number_of_coordinates: c_int,
}

impl COsrmAnnotation {
    pub(crate) fn to_annotation(&self) -> Annotation {
        let mut annotation = Annotation {
            duration: Vec::new(),
            distance: Vec::new(),
            speed: Vec::new(),
            weight: Vec::new(),
            nodes: Vec::new(),
            datasources: Vec::new(),
            metadata: None,
        };

        if self.duration != std::ptr::null_mut() {
            annotation.duration = unsafe {
                slice::from_raw_parts(self.duration, self.number_of_coordinates as usize).to_vec()
            };
        }

        if self.distance != std::ptr::null_mut() {
            annotation.distance = unsafe {
                slice::from_raw_parts(self.distance, self.number_of_coordinates as usize).to_vec()
            };
        }

        if self.speed != std::ptr::null_mut() {
            annotation.speed = unsafe {
                slice::from_raw_parts(self.speed, self.number_of_coordinates as usize).to_vec()
            };
        }

        if self.weight != std::ptr::null_mut() {
            annotation.weight = unsafe {
                slice::from_raw_parts(self.weight, self.number_of_coordinates as usize).to_vec()
            };
        }

        if self.nodes != std::ptr::null_mut() {
            annotation.nodes = unsafe {
                slice::from_raw_parts(self.nodes, self.number_of_coordinates as usize).to_vec()
            };
        }

        if self.datasources != std::ptr::null_mut() {
            annotation.datasources = unsafe {
                slice::from_raw_parts(self.datasources, self.number_of_coordinates as usize)
                    .to_vec()
            };
        }

        if self.metadata != std::ptr::null_mut() {
            annotation.metadata = Option::from(unsafe { (*self.metadata).to_meta_data() });
        }

        annotation
    }
}

#[derive(Debug)]
pub struct Annotation {
    pub duration: Vec<f64>,
    pub distance: Vec<f64>,
    pub speed: Vec<f64>,
    pub weight: Vec<f64>,
    pub nodes: Vec<i64>,
    pub datasources: Vec<i32>,
    pub metadata: Option<MetaData>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmRouteLeg {
    pub(crate) annotation: *const COsrmAnnotation,
    pub(crate) duration: c_double,
    pub(crate) summary: *const c_char,
    pub(crate) weight: c_double,
    pub(crate) distance: c_double,
    pub(crate) steps: *const COsrmStep,
    pub(crate) number_of_steps: c_int,
}

impl COsrmRouteLeg {
    pub(crate) fn to_route_leg(&self) -> RouteLeg {
        let mut route_leg = RouteLeg {
            annotation: None,
            duration: self.duration,
            summary: c_string_to_option_string(self.summary),
            weight: self.weight,
            distance: self.distance,
            steps: Vec::new(),
        };

        if self.annotation != std::ptr::null_mut() {
            route_leg.annotation = Option::from(unsafe { (*self.annotation).to_annotation() });
        }

        if self.steps != std::ptr::null_mut() {
            let steps_vec = unsafe {
                slice::from_raw_parts(self.steps, self.number_of_steps as usize).to_vec()
            };

            for step in steps_vec {
                route_leg.steps.push(step.to_step());
            }
        }

        route_leg
    }
}

#[derive(Debug)]
pub struct RouteLeg {
    pub annotation: Option<Annotation>,
    pub duration: f64,
    pub summary: Option<String>,
    pub weight: f64,
    pub distance: f64,
    pub steps: Vec<Step>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COsrmRoute {
    pub(crate) duration: c_double,
    pub(crate) distance: c_double,
    pub(crate) weight_name: *const c_char,
    pub(crate) weight: c_double,
    pub(crate) geometry: *const c_char,
    pub(crate) legs: *const COsrmRouteLeg,
    pub(crate) number_of_legs: c_int,
}

impl COsrmRoute {
    pub(crate) fn to_route(&self) -> Route {
        let mut route = Route {
            duration: self.duration,
            distance: self.distance,
            weight_name: c_string_to_option_string(self.weight_name),
            weight: self.weight,
            geometry: c_string_to_option_string(self.geometry),
            legs: Vec::new(),
        };

        if self.legs != std::ptr::null_mut() {
            let legs_vec =
                unsafe { slice::from_raw_parts(self.legs, self.number_of_legs as usize).to_vec() };

            for leg in legs_vec {
                route.legs.push(leg.to_route_leg());
            }
        }

        route
    }
}

#[derive(Debug)]
pub struct Route {
    pub duration: f64,
    pub distance: f64,
    pub weight_name: Option<String>,
    pub weight: f64,
    pub geometry: Option<String>,
    pub legs: Vec<RouteLeg>,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct CGeneralOptions {
    pub(crate) coordinate: *const COsrmCoordinate,
    pub(crate) number_of_coordinates: c_int,
    pub(crate) bearings: *const *const Bearing,
    pub(crate) radiuses: *const *const c_double,
    pub(crate) generate_hints: Boolean,
    pub(crate) skip_waypoints: Boolean,
    pub(crate) hints: *const *const c_char,
    pub(crate) approach: *const *const Approach,
    pub(crate) exclude: *const *const c_char,
    pub(crate) number_of_excludes: c_int,
}

impl CGeneralOptions {
    pub(crate) fn new(option: &mut GeneralOptions) -> CGeneralOptions {
        option.c_coordinate = to_vec_ccoordinate(&option.coordinate);
        let mut general_c_option = CGeneralOptions {
            coordinate: option.c_coordinate.as_ptr(),
            number_of_coordinates: option.coordinate.len() as c_int,
            bearings: std::ptr::null(),
            radiuses: std::ptr::null(),
            generate_hints: Boolean::from(option.generate_hints),
            skip_waypoints: Boolean::from(option.skip_waypoints),
            hints: std::ptr::null(),
            approach: std::ptr::null(),
            exclude: std::ptr::null(),
            number_of_excludes: 0,
        };

        if option.bearings.is_some() {
            option.bearings_t.clear();

            for bearing in option.bearings.as_ref().unwrap() {
                match bearing {
                    Some(it) => {
                        option.bearings_t.push(it);
                    }
                    None => {
                        option.bearings_t.push(std::ptr::null());
                    }
                }
            }
            general_c_option.bearings = option.bearings_t.as_ptr();
        }

        if option.radiuses.is_some() {
            option.radiuses_t.clear();

            for radiuse in option.radiuses.as_ref().unwrap() {
                match radiuse {
                    Some(it) => {
                        option.radiuses_t.push(it);
                    }
                    None => {
                        option.radiuses_t.push(std::ptr::null());
                    }
                }
            }
            general_c_option.radiuses = option.radiuses_t.as_ptr();
        }

        if option.hints.is_some() {
            let mut c_hint_vec = Vec::new();
            for hint in option.hints.as_ref().unwrap() {
                c_hint_vec.push(CString::new(hint.clone()).unwrap());
            }
            option.c_hints = Option::from(c_hint_vec);
            general_c_option.hints =
                option.c_hints.as_ref().unwrap().as_ptr() as *const *const c_char;
        }

        if option.approach.is_some() {
            option.approach_t.clear();

            for approach in option.approach.as_ref().unwrap() {
                match approach {
                    Some(it) => {
                        option.approach_t.push(it);
                    }
                    None => {
                        option.approach_t.push(std::ptr::null());
                    }
                }
            }

            general_c_option.approach = option.approach_t.as_ptr();
        }

        if option.exclude.is_some() {
            let mut c_exclude_vec = Vec::new();
            for exclude in option.exclude.as_ref().unwrap() {
                c_exclude_vec.push(CString::new(exclude.clone()).unwrap());
            }
            option.c_exclude = Option::from(c_exclude_vec);
            general_c_option.exclude =
                option.c_exclude.as_ref().unwrap().as_ptr() as *const *const c_char;
            general_c_option.number_of_excludes = option.exclude.as_ref().unwrap().len() as c_int;
        }

        general_c_option
    }
}

#[derive(Clone)]
pub struct GeneralOptions {
    pub coordinate: Vec<Coordinate>,
    pub(crate) c_coordinate: Vec<COsrmCoordinate>,
    pub bearings: Option<Vec<Option<Bearing>>>,
    pub(crate) bearings_t: Vec<*const Bearing>,
    pub radiuses: Option<Vec<Option<f64>>>,
    pub(crate) radiuses_t: Vec<*const f64>,
    pub generate_hints: bool,
    pub skip_waypoints: bool,
    pub hints: Option<Vec<String>>,
    pub(crate) c_hints: Option<Vec<CString>>,
    pub approach: Option<Vec<Option<Approach>>>,
    pub(crate) approach_t: Vec<*const Approach>,
    pub exclude: Option<Vec<String>>,
    pub(crate) c_exclude: Option<Vec<CString>>,
}

impl GeneralOptions {
    pub fn new(coordinates: &Vec<Coordinate>) -> GeneralOptions {
        GeneralOptions {
            coordinate: coordinates.clone(),
            c_coordinate: to_vec_ccoordinate(&coordinates),
            bearings: None,
            bearings_t: vec![],
            radiuses: None,
            radiuses_t: vec![],
            generate_hints: true,
            skip_waypoints: false,
            hints: None,
            c_hints: None,
            approach: None,
            approach_t: vec![],
            exclude: None,
            c_exclude: None,
        }
    }

    pub fn radiuses(&mut self, val: &Option<Vec<Option<f64>>>) -> &mut GeneralOptions {
        self.radiuses = val.clone();
        self
    }
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct CWaypoint {
    pub(crate) hint: *const c_char,
    pub(crate) distance: c_double,
    pub(crate) name: *const c_char,
    pub(crate) location: [c_double; 2],
}

#[derive(Debug)]
pub struct Waypoint {
    pub hint: Option<String>,
    pub distance: f64,
    pub name: String,
    pub location: [f64; 2],
}

impl Waypoint {
    pub(crate) fn new(c_waypoints: &CWaypoint) -> Waypoint {
        let mut hint: Option<String> = None;
        if c_waypoints.hint != null() {
            hint = Option::from(c_string_to_string(c_waypoints.hint));
        }

        Waypoint {
            hint,
            distance: c_waypoints.distance,
            name: c_string_to_string(c_waypoints.name),
            location: c_waypoints.location,
        }
    }
}

pub(crate) fn c_string_to_string(c_string: *const c_char) -> String {
    if c_string == std::ptr::null_mut() {
        return "".to_string();
    }

    let c_str: &CStr = unsafe { CStr::from_ptr(c_string) };
    let c_str_slice: &str = c_str.to_str().unwrap();

    c_str_slice.to_owned()
}

pub(crate) fn c_string_to_option_string(c_string: *const c_char) -> Option<String> {
    if c_string == std::ptr::null_mut() {
        return None;
    }

    let c_str: &CStr = unsafe { CStr::from_ptr(c_string) };
    let c_str_slice: &str = c_str.to_str().unwrap();

    Option::from(c_str_slice.to_owned())
}
