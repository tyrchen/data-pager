use snafu::Snafu;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Page size must be between 1-99. Got: {size}"))]
    InvalidPageSize { size: u64 },
    #[snafu(display("Source cannot be empty"))]
    InvalidSource,
    #[snafu(display("Invalid base64 string: {}", s))]
    Base64Decode {
        s: String,
        source: base64::DecodeSliceError,
    },
    #[snafu(display("Invalid UTF-8 string"))]
    InvalidUtf8 { source: std::str::Utf8Error },
    #[snafu(display("Invalid number: {s}"))]
    InvalidNumber {
        s: String,
        source: std::num::ParseIntError,
    },
}
