use failure::Fail;

#[derive(Debug, Fail)]
pub enum InternalError {
    #[fail(display = "out of memory")]
    OutOfMemory,
    #[fail(display = "data store failed")]
    DataStoreFailed,
    #[fail(display = "internal error")]
    Internal,
    #[fail(display = "unsupported operation")]
    UnsupportedOperation,
    #[fail(display = "bad parameter")]
    BadParameter,
}

// #[derive(Debug, Fail)]
// pub enum NetworkError {
//     #[fail(display = "network not found")]
//     NotFound,
// }
