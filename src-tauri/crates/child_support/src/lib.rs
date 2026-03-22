#![feature(portable_simd)]

extern crate core;

pub mod api {
    #[path = "../../../../src/api/api_response.rs"]
    pub mod api_response;
}

pub mod app {
    #[path = "../../../../src/app/app_error.rs"]
    pub mod app_error;
}

#[path = "../../../src/constant.rs"]
pub mod constant;

pub mod domain;

pub mod infrastructure;
