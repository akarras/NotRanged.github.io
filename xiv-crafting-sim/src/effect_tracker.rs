use crate::actions::Action;
use std::fmt::{Display, Formatter};
use std::iter::{FromIterator, Filter};
use std::slice::IterMut;
use serde::{Serialize};

/// Effect tracker is a key value store
/// Data is a contiguous slice of memory, if we somehow have more than abilities than space, please just increase this and don't look back.
#[derive(Default, Debug, Serialize, Clone)]
pub struct EffectData([Option<(Action, i8)>; 5]);


impl EffectData {
    pub(crate) fn get_mut(&mut self, action: Action) -> Option<&mut i8> {
        self.0.iter_mut().flat_map(|m| m).find(|(a, _)| *a==action).map(|(_, i)| i)
    }

    pub(crate) fn get(&self, action: Action) -> Option<&(Action, i8)> {
        self.0.iter().flat_map(|m| m).find(|(a, _)| *a==action)
    }

    pub(crate) fn remove(&mut self, action: Action) {
        // there shouldn't be multiple actions, but just in case use filter and clear all of them
        for value in self.0.iter_mut().filter(|m| m.map(|(a,_)| a.eq(&action)).unwrap_or_default()) {
            *value = None;
        }
    }

    /// Insert all values
    pub(crate) fn insert(&mut self, action: Action, value: i8) -> bool {
        if let Some(v) = self.0.iter_mut().find(|f| f.is_none()) {
            *v = Some((action, value));
            true
        } else {
            false
        }
    }

    pub(crate) fn iter_mut(&mut self) -> Filter<IterMut<Option<(Action, i8)>>, fn(&&mut Option<(Action, i8)>) -> bool> {
        self.0.iter_mut().filter(|m| m.is_some())
    }
}

impl Display for EffectData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for value in self.0 {
            if let Some((action, count)) = value {
                write!(f, "{:?}:{}", action, count)?; // using debug version of Action can help see BasicSynth2
            }
        }
        Ok(())
    }
}



impl FromIterator<(Action, i32)> for EffectData {
    fn from_iter<I: IntoIterator<Item=(Action, i32)>>(iter: I) -> Self {
        let mut effects = EffectData::default();
        for (a, v) in iter {
            effects.insert(a, v as i8);
        }
        effects
    }
}
