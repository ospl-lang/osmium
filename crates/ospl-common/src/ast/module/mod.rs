//! PersonP -- I basically vibe-coded this whole module.
//!            It's 8 PM and I'm FUCKING TIRED
//! 
//! temporarily disabled

use std::collections::HashMap;

use crate::ast::repr::Type;

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VScope {
    pub members: HashMap<String, Type>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VPathError {
    IntermediaryNotCreated,
    IndermediaryIsNotModule,
    MemberNotFound,
    EmptyPath,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VPath(pub Vec<String>);

impl From<String> for VPath {
    fn from(value: String) -> Self {
        let spl = value.split('/');
        let mut s = Self(Vec::new());
        
        for seg in spl {
            s.0.push(seg.to_string());
        }

        return s
    }
}

impl VScope {
    /// Get the container for the final element in the path
    pub fn get_container_mut<'a>(&'a mut self, path: &'a VPath) -> Result<(&'a mut VScope, &'a str), VPathError> {
        if path.0.is_empty() {
            return Err(VPathError::EmptyPath);
        }

        let mut current = self;
        for segment in &path.0[..path.0.len() - 1] {
            if let Some(Type::Module(scope)) = current.members.get_mut(segment) {
                current = scope;
            } else {
                return Err(VPathError::IndermediaryIsNotModule);
            }
        }

        return Ok((current, &path.0[path.0.len() - 1]))
    }

    /// Get the element given
    pub fn get_member_mut<'a>(&'a mut self, path: &'a VPath) -> Result<&'a mut Type, VPathError> {
        let (cont, last) = self.get_container_mut(path)?;
        let Some(x) = cont.members.get_mut(last)
            else { return Err(VPathError::MemberNotFound) };

        return Ok(x)
    }

    /// Create a new member at the given path, automatically creating intermediate modules
    pub fn create_member(&mut self, path: &VPath, value: Type) -> Result<&mut Type, VPathError> {
        let mut current = self;
        for segment in &path.0[..path.0.len() - 1] {
            current = match current.members.get_mut(segment) {
                Some(e) => match e {
                    Type::Module(scope) => scope,
                    _ => return Err(VPathError::IndermediaryIsNotModule),
                },
                None => return Err(VPathError::IntermediaryNotCreated)
            };
        }

        // FIX-UNWRAP: don't unwrap
        let last = path.0.last().unwrap().clone();
        current.members.insert(last.clone(), value);
        return Ok(current.members.get_mut(&last).unwrap())
    }
}