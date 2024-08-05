use super::{tbff::TBFFooter, tbfh::TBFHeader};

pub struct TabTbf {
    _filename: String,
    _tbfh: TBFHeader,
    _app_binary: Vec<u8>,
    _tbff: TBFFooter,
}

impl TabTbf {}
