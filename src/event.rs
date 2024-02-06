use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(PartialEq, Eq)]
pub enum EventTriggerResult {
    Continue,
    Unregister,
}

#[allow(type_alias_bounds)]
type EventCallback<V: ?Sized> = Box<dyn FnMut(&V) -> EventTriggerResult>;

pub struct EventRegister<H, V>
where
    H: Eq + Hash,
    V: ?Sized,
{
    listeners: RefCell<HashMap<H, Vec<EventCallback<V>>>>,
}

impl<H, V> Default for EventRegister<H, V>
where
    H: Eq + Hash,
    V: ?Sized,
{
    fn default() -> Self {
        Self {
            listeners: RefCell::default(),
        }
    }
}

impl<H, V: ?Sized> EventRegister<H, V>
where
    H: Eq + Hash,
{
    pub fn connect<F>(&self, handle: H, f: F)
    where
        F: FnMut(&V) -> EventTriggerResult + 'static,
    {
        self.listeners
            .borrow_mut()
            .entry(handle)
            .or_default()
            .push(Box::new(f) as _)
    }

    pub fn trigger(&self, handle: &H, value: &V) {
        let Some((handle, mut listeners)) = self.listeners.borrow_mut().remove_entry(handle) else {
            return;
        };

        listeners.retain_mut(|listener| listener(value) == EventTriggerResult::Continue);

        if listeners.is_empty() {
            return;
        };

        match self.listeners.borrow_mut().entry(handle) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                let mut tail = std::mem::replace(entry, listeners);
                entry.append(&mut tail);
            }
            Entry::Vacant(entry) => {
                entry.insert(listeners);
            }
        };
    }
}
