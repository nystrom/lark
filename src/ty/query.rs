use crate::ir::DefId;

/// Information that "type operations" need from the rest of the
/// system.
pub trait TyQueries {
    fn is_value_type(&self, name: DefId);
}
