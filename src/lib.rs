#![allow(dead_code)]

use core::fmt::Display;
use std::ffi::{CStr, CString};
use std::{
    fmt,
    os::raw::{c_char, c_double, c_int, c_void},
};

pub mod general;
pub mod match_api;
pub mod nearest;
pub mod route;
pub mod table;
pub mod tile;
pub mod trip;

#[link(name = "c_osrm")]
extern "C" {
    fn osrm_create(config: *const CEngineConfig, return_value: *mut *mut COSRM);
    fn osrm_destroy_error_message(error_message: *const c_char);
    fn osrm_destroy(osrm: *mut c_void);
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct COSRM {
    pub(crate) obj: *mut c_void,
    pub(crate) error_message: *mut c_char,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum Status {
    Ok = 0,
    Error = 1,
}

impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        if self.eq(&Status::Ok) {
            write!(f, "Ok")
        } else {
            write!(f, "Error")
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub enum Algorithm {
    CH = 0,
    CoreCH = 1, // Deprecated, will be removed in v6.0
    MLD = 2,
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub enum Boolean {
    FALSE = 0,
    TRUE = 1,
}

impl Boolean {
    fn from(value: bool) -> Boolean {
        match value {
            true => Boolean::TRUE,
            false => Boolean::FALSE,
        }
    }
}

#[repr(C)]
pub(crate) struct CEngineConfig {
    pub storage_config: *const c_char,
    pub max_locations_trip: c_int,
    pub max_locations_viaroute: c_int,
    pub max_locations_distance_table: c_int,
    pub max_locations_map_matching: c_int,
    pub max_radius_map_matching: c_double,
    pub max_results_nearest: c_int,
    pub max_alternatives: c_int,
    pub use_shared_memory: Boolean,
    pub memory_file: *const c_char,
    pub use_mmap: Boolean,
    pub algorithm: Algorithm,
    pub verbosity: *const c_char,
    pub dataset_name: *const c_char,
}

impl CEngineConfig {
    pub(crate) fn new(path: &CString) -> CEngineConfig {
        CEngineConfig {
            storage_config: path.as_ptr(),
            max_locations_trip: -1,
            max_locations_viaroute: -1,
            max_locations_distance_table: -1,
            max_locations_map_matching: -1,
            max_radius_map_matching: -1.0,
            max_results_nearest: -1,
            max_alternatives: 3,
            use_shared_memory: Boolean::TRUE,
            memory_file: std::ptr::null(),
            use_mmap: Boolean::TRUE,
            algorithm: Algorithm::CH,
            verbosity: std::ptr::null(),
            dataset_name: std::ptr::null(),
        }
    }
}

pub struct EngineConfig {
    pub storage_config: String,
    pub c_storage_config: CString,
    pub max_locations_trip: i32,
    pub max_locations_viaroute: i32,
    pub max_locations_distance_table: i32,
    pub max_locations_map_matching: i32,
    pub max_radius_map_matching: f64,
    pub max_results_nearest: i32,
    pub max_alternatives: i32,
    pub use_shared_memory: bool,
    pub memory_file: Option<String>,
    c_memory_file: CString,
    pub use_mmap: bool,
    pub algorithm: Algorithm,
    pub verbosity: Option<String>,
    c_verbosity: CString,
    pub dataset_name: Option<String>,
    c_dataset_name: CString,
}

impl EngineConfig {
    pub fn new(path: &str) -> EngineConfig {
        EngineConfig {
            storage_config: path.to_string(),
            c_storage_config: CString::new(path.clone()).unwrap(),
            max_locations_trip: -1,
            max_locations_viaroute: -1,
            max_locations_distance_table: -1,
            max_locations_map_matching: -1,
            max_radius_map_matching: -1.0,
            max_results_nearest: -1,
            max_alternatives: 3,
            use_shared_memory: true,
            memory_file: None,
            c_memory_file: CString::default(),
            use_mmap: true,
            algorithm: Algorithm::CH,
            verbosity: None,
            c_verbosity: CString::default(),
            dataset_name: None,
            c_dataset_name: CString::default(),
        }
    }

    pub(crate) fn to_cengine_config(&mut self) -> CEngineConfig {
        self.c_storage_config = CString::new(self.storage_config.clone()).unwrap();
        let mut cengine_config = CEngineConfig::new(&self.c_storage_config);

        cengine_config.max_alternatives = self.max_alternatives;
        cengine_config.max_locations_viaroute = self.max_locations_viaroute;
        cengine_config.max_locations_distance_table = self.max_locations_distance_table;
        cengine_config.max_locations_map_matching = self.max_locations_map_matching;
        cengine_config.max_radius_map_matching = self.max_radius_map_matching;
        cengine_config.max_results_nearest = self.max_results_nearest;

        if self.use_shared_memory {
            cengine_config.use_shared_memory = Boolean::TRUE;
        } else {
            cengine_config.use_shared_memory = Boolean::FALSE;
        }

        match &self.memory_file {
            Some(memory_file_string) => {
                self.c_memory_file = CString::new(memory_file_string.clone()).unwrap();

                cengine_config.memory_file = self.c_memory_file.as_ptr();
            }
            None => {}
        }

        if self.use_mmap {
            cengine_config.use_mmap = Boolean::TRUE;
        } else {
            cengine_config.use_mmap = Boolean::FALSE;
        }

        cengine_config.algorithm = self.algorithm.clone();

        match &self.verbosity {
            Some(verbosity_string) => {
                self.c_verbosity = CString::new(verbosity_string.clone()).unwrap();

                cengine_config.verbosity = self.c_verbosity.as_ptr();
            }
            None => {}
        }

        match &self.dataset_name {
            Some(dataset_name_string) => {
                self.c_dataset_name = CString::new(dataset_name_string.clone())
                    .expect("to_CEngineConfig::dataset_name::new failed");

                cengine_config.dataset_name = self.c_dataset_name.as_ptr();
            }
            None => {}
        }

        cengine_config
    }
}

pub struct Osrm {
    config: Box<*mut c_void>,
}

impl Osrm {
    pub fn new(config: &mut EngineConfig) -> Result<Osrm, String> {
        let c_engine_config = config.to_cengine_config();
        unsafe {
            let mut result: *mut COSRM = std::ptr::null_mut();
            let result_ptr: *mut *mut COSRM = &mut result;
            osrm_create(&c_engine_config as *const CEngineConfig, result_ptr);

            if (*result).error_message != std::ptr::null_mut() {
                let c_name_buf: *const c_char = (*result).error_message;
                let c_name_str: &CStr = CStr::from_ptr(c_name_buf);

                match c_name_str.to_str() {
                    Ok(ok) => {
                        let name_str_slice = ok.to_string();

                        osrm_destroy_error_message((*result).error_message);

                        return Err(name_str_slice);
                    }
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }

            let r = Osrm {
                config: Box::new((*result).obj),
            };
            println!("osrm created {:p}", r.config);
            Ok(r)
        }
    }
}

impl Drop for Osrm {
    fn drop(&mut self) {
        unsafe {
            println!("osrm destroyed {:p}", self.config);
            osrm_destroy(*self.config);
        }
    }
}

unsafe impl Send for Osrm {}
unsafe impl Sync for Osrm {}
