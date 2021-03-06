pub mod types;
pub mod helpers;
pub mod proofs;

extern crate libc;

use std::mem;
use std::slice::from_raw_parts;
use std::sync::Once;
use log::info;
use crate::types::{fil_PostConfig, fil_PubIn, fil_SnarkPostResponse, fil_VanillaProof, TaskInfo};
use ffi_toolkit::{catch_panic_response, FCPResponseStatus, raw_ptr, rust_str_to_c_str};
use crate::proofs::run_snark;

/// Protects the init off the logger.
static LOG_INIT: Once = Once::new();

/// Ensures the logger is initialized.
pub fn init_log() {
    LOG_INIT.call_once(|| {
        fil_logger::init();
    });
}

#[no_mangle]
pub unsafe extern "C" fn fil_snark_post(
    vanilla_proof: fil_VanillaProof,
    p_i: fil_PubIn,
    p_c: fil_PostConfig,
    replicas_len: libc::size_t,
)
    -> *mut fil_SnarkPostResponse {
    let parts = from_raw_parts(p_i.ptr, p_i.len);
    catch_panic_response(|| {
        init_log();
        info!("received one task");
        let mut response = fil_SnarkPostResponse::default();
        let ref task_info = TaskInfo {
            vanilla_proof_u8: vanilla_proof,
            pub_in_u8: p_i,
            post_config_u8: p_c,
            replicas_len,
        };

        match run_snark(task_info) {
            Ok(mut r) => {
                response.status_code = FCPResponseStatus::FCPNoError;
                response.proofs_ptr_0 = r.as_mut_ptr();
                response.proofs_len = r.len();
                mem::forget(r)
            }
            Err(e) => {
                response.status_code = FCPResponseStatus::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", e))
            }
        }

        raw_ptr(response)
    })
}

