use std::{fmt::{Display, Formatter}, sync::Arc};

use hashbrown::HashMap;
use parking_lot::Mutex;

pub mod exports {
    pub use super::Stat;
    pub use super::Stats;
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Stat {
    f32(f32),
    i32(i32),
    u32(u32),
    usize(usize),
    String(String),
    Duration(std::time::Duration),
}

impl Display for Stat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::f32(value) => write!(f, "{}", value),
            Self::i32(value) => write!(f, "{}", value),
            Self::u32(value) => write!(f, "{}", value),
            Self::usize(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "{}", value),
            Self::Duration(value) => write!(f, "{:.2?}", value),
        }
    }
}

impl From<f32> for Stat {
    fn from(value: f32) -> Self {
        Self::f32(value)
    }
}

impl From<i32> for Stat {
    fn from(value: i32) -> Self {
        Self::i32(value)
    }
}

impl From<u32> for Stat {
    fn from(value: u32) -> Self {
        Self::u32(value)
    }
}

impl From<usize> for Stat {
    fn from(value: usize) -> Self {
        Self::usize(value)
    }
}

impl From<String> for Stat {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<std::time::Duration> for Stat {
    fn from(value: std::time::Duration) -> Self {
        Self::Duration(value)
    }
}

impl Stat {
    pub(crate) fn reset(&mut self) {
        match self {
            Self::f32(value) => *value = 0.0,
            Self::i32(value) => *value = 0,
            Self::u32(value) => *value = 0,
            Self::usize(value) => *value = 0,
            Self::String(value) => *value = String::new(),
            Self::Duration(value) => *value = std::time::Duration::new(0, 0),
        }
    }
}

static mut STATS: Option<Arc<Mutex<Stats>>> = None;

pub struct Stats {
    /// The data stored in the stats.
    /// (label, (value, reset_per_frame))
    data: HashMap<String, (Stat, bool)>,
}

pub trait StatsInternal {
    fn frame_reset(&mut self);

    fn initialize();
}

impl StatsInternal for Stats {
    fn frame_reset(&mut self) {
        self.data
            .iter_mut()
            .filter(|(_, (_, reset))| *reset)
            .for_each(|(_, (value, _))| value.reset());
    }

    fn initialize() {
        unsafe {
            STATS = Some(Arc::new(Mutex::new(Self {
                data: HashMap::new(),
            })));
        }
    }
}

impl Stats {
    pub fn global() -> parking_lot::MutexGuard<'static, Self> {
        unsafe {
            STATS
                .as_ref()
                .expect("Stats requested before initialization")
                .lock()
        }
    }

    pub fn set(&mut self, label: &str, value: Stat, reset_per_frame: bool) {
        self.data
            .insert(label.to_string(), (value, reset_per_frame));
    }

    pub fn get(&self, label: &str) -> Option<&(Stat, bool)> {
        self.data.get(label)
    }

    pub fn get_mut(&mut self, label: &str) -> Option<&mut (Stat, bool)> {
        self.data.get_mut(label)
    }

    pub fn get_mut_else_default(&mut self, label: &str, default: (Stat, bool)) -> &mut (Stat, bool) {
        self.data
            .entry(label.to_string())
            .or_insert(default)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Stat)> {
        self.data.iter().map(|(label, (value, _))| (label, value))
    }

    /// Data sorted by label.
    pub fn sorted_by_label(&self) -> Vec<(&String, &Stat)> {
        let mut data = self.data.iter().collect::<Vec<_>>();
        data.sort_by(|(a, _), (b, _)| a.cmp(b));
        data.into_iter().map(|(label, (value, _))| (label, value)).collect()
    }

    pub fn remove(&mut self, label: &str) -> Option<Stat> {
        self.data.remove(label).map(|(value, _)| value)
    }
}
