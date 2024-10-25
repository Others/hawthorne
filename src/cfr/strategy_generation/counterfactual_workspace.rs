use crate::cfr::game_model::{UtilityForAllPlayers, VisibleInfo};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use thread_local::ThreadLocal;

#[derive(Debug)]
pub struct CounterfactualWorkspace<INFO: VisibleInfo> {
    pub(crate) workspace: ThreadLocal<RefCell<ThreadLocalWorkspace<INFO>>>,
}

impl<INFO: VisibleInfo> Default for CounterfactualWorkspace<INFO> {
    fn default() -> Self {
        Self {
            workspace: ThreadLocal::new(),
        }
    }
}

#[derive(Debug)]
pub struct ThreadLocalWorkspace<INFO: VisibleInfo> {
    counterfactual_buffer: FxHashMap<INFO, BufferedValue>,
    work_stack: Vec<(INFO::Gamestate, INFO)>,
}

impl<INFO: VisibleInfo> ThreadLocalWorkspace<INFO> {
    pub fn clear(&mut self) {
        self.counterfactual_buffer.clear();
        self.work_stack.clear();
    }

    pub fn get_buffered_value(
        &self,
        info: &INFO,
        stamp: BufferedValueTimestamp,
    ) -> Option<&BufferedValue> {
        if let Some(v) = self.counterfactual_buffer.get(info) {
            if v.update_timestamp == stamp {
                return Some(v);
            }
        }

        None
    }

    pub fn insert_buffered_value(&mut self, info: INFO, buffered_value: BufferedValue) {
        self.counterfactual_buffer.insert(info, buffered_value);
    }

    pub fn buffered_value_ready(&self, info: &INFO, stamp: BufferedValueTimestamp) -> bool {
        self.get_buffered_value(info, stamp).is_some()
    }

    pub fn push_stack(&mut self, gamestate: INFO::Gamestate, info: INFO) {
        self.work_stack.push((gamestate, info));
    }

    pub fn pop_stack(&mut self) -> Option<(INFO::Gamestate, INFO)> {
        self.work_stack.pop()
    }
}

impl<INFO: VisibleInfo> Default for ThreadLocalWorkspace<INFO> {
    fn default() -> Self {
        Self {
            counterfactual_buffer: FxHashMap::default(),
            work_stack: Vec::new(),
        }
    }
}

pub type BufferedValueTimestamp = (u32, usize);

#[derive(Debug, Clone)]
pub struct BufferedValue {
    pub(crate) utility: UtilityForAllPlayers,
    pub(crate) update_timestamp: BufferedValueTimestamp,
}
