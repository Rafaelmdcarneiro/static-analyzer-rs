use crate::codemap::CodeMap;//{CodeMap, FileMap, Span};

/// The driver is in charge of orchestrating the whole analysis process and
/// making sure all the bits and pieces integrate nicely.
#[derive(Debug)]
pub struct Driver {
    codemap: CodeMap,
}

impl Driver {
    /// Create a new driver.
    pub fn new() -> Driver {
        Driver {
            codemap: CodeMap::new(),
        }
    }

    /// Get access to the driver's `CodeMap`.
    pub fn codemap(&mut self) -> &mut CodeMap {
        &mut self.codemap
    }
}

impl Default for Driver {
    fn default() -> Driver {
        Driver::new()
    }
}