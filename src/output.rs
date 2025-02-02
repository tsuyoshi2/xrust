use crate::qname::QualifiedName;
use core::fmt;

/// An output definition. See XSLT v3.0 26 Serialization
#[derive(Clone, Debug)]
pub struct OutputDefinition {
    name: Option<QualifiedName>, // TODO: EQName
    indent: bool,
    // TODO: all the other myriad output parameters
}

impl OutputDefinition {
    pub fn new() -> OutputDefinition {
        OutputDefinition {
            name: None,
            indent: false,
        }
    }
    pub fn get_name(&self) -> Option<QualifiedName> {
        self.name.clone()
    }
    pub fn set_name(&mut self, name: Option<QualifiedName>) {
        match name {
            Some(n) => {
                self.name.replace(n);
            }
            None => {
                self.name = None;
            }
        }
    }
    pub fn get_indent(&self) -> bool {
        self.indent
    }
    pub fn set_indent(&mut self, ind: bool) {
        self.indent = ind;
    }
}
impl fmt::Display for OutputDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.indent {
            f.write_str("indent output")
        } else {
            f.write_str("do not indent output")
        }
    }
}
