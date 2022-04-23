use zt_sys::controller::*;
use failure::Fail;

#[derive(Debug, Fail, FromPrimitive)]
pub enum FatalError {
    #[fail(display = "out of memory")]
    OutOfMemory = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_OUT_OF_MEMORY as isize,
    #[fail(display = "data store failed")]
    DataStoreFailed = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_DATA_STORE_FAILED as isize,
    #[fail(display = "internal error")]
    Internal = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_INTERNAL as isize,
}

#[derive(Debug, Fail, FromPrimitive)]
pub enum NetworkError {
    #[fail(display = "network not found")]
    NotFound = RZTC_NetworkErrorCode_NC_ERROR_OBJECT_NOT_FOUND as isize,
    #[fail(display = "access denied")]
    AccessDenied = RZTC_NetworkErrorCode_NC_ERROR_ACCESS_DENIED as isize,
    #[fail(display = "internal server error")]
    InternalServerError = RZTC_NetworkErrorCode_NC_ERROR_INTERNAL_SERVER_ERROR as isize,
    #[fail(display = "authentication required")]
    AuthenticationRequired = RZTC_NetworkErrorCode_NC_ERROR_AUTHENTICATION_REQUIRED as isize,
}

#[derive(Debug, Fail, FromPrimitive)]
pub enum SignatureError {
    #[fail(display = "signer has no keypair")]
    NoKeypair,
}
