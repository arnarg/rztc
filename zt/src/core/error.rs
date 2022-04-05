use zt_sys::*;
use failure::Fail;

#[derive(Debug, Fail, FromPrimitive)]
pub enum FatalError {
    #[fail(display = "out of memory")]
    OutOfMemory = ZT_ResultCode_ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY as isize,
    #[fail(display = "data store failed")]
    DataStoreFailed = ZT_ResultCode_ZT_RESULT_FATAL_ERROR_DATA_STORE_FAILED as isize,
    #[fail(display = "internal error")]
    Internal = ZT_ResultCode_ZT_RESULT_FATAL_ERROR_INTERNAL as isize,
}

// #[derive(Debug, Fail)]
// pub enum NetworkError {
//     #[fail(display = "network not found")]
//     NotFound,
// }
