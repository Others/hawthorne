use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::data_for_known_infosets::DataForKnownInfosets;
use crate::cfr::strategy_generation::workspace_data::timestamp::Timestamp;
use bumpalo_herd::Member;
use crossbeam::queue::SegQueue;
use std::cell::{RefCell, RefMut};
use thread_local::ThreadLocal;

pub(crate) mod batch_item_data;
pub(crate) mod data_for_infoset;
pub(crate) mod data_for_known_infosets;
pub(crate) mod data_for_move;
mod move_data;
pub(crate) mod timestamp;

pub(crate) struct StrategyGenerationProgress<'h, INFO: VisibleInfo> {
    data_for_known_infosets: DataForKnownInfosets<'h, INFO>,
    thread_local_workstack: ThreadLocal<RefCell<ThreadLocalWorkStack<'h, INFO>>>,
    updated_infosets: SegQueue<&'h DataForInfoSet<INFO>>,
}

impl<'h, INFO: VisibleInfo> StrategyGenerationProgress<'h, INFO> {
    pub(crate) fn new() -> Self {
        Self {
            data_for_known_infosets: DataForKnownInfosets::default(),
            thread_local_workstack: ThreadLocal::new(),
            updated_infosets: Default::default(),
        }
    }

    pub(crate) fn thread_local_workstack(&self) -> RefMut<ThreadLocalWorkStack<'h, INFO>> {
        self.thread_local_workstack.get_or_default().borrow_mut()
    }

    pub(crate) fn mark_as_completed(
        &self,
        data_for_info_set: &'h DataForInfoSet<INFO>,
        timestamp: Timestamp,
    ) {
        if data_for_info_set.check_for_writeback(timestamp) {
            self.updated_infosets.push(data_for_info_set);
        }
    }

    pub(crate) fn get_data_for_infoset(
        &self,
        data: INFO,
        member: &Member<'h>,
    ) -> &'h DataForInfoSet<INFO> {
        self.data_for_known_infosets.data_for_infoset(data, member)
    }

    #[inline]
    pub(crate) fn consume_updated_infosets(
        &self,
        f: impl Fn(&'h DataForInfoSet<INFO>) + Send + Sync,
    ) {
        rayon::scope(|x| {
            while let Some(i) = self.updated_infosets.pop() {
                x.spawn(|_| f(i));
            }
        });
    }

    pub(crate) fn into_infoset_data(self) -> DataForKnownInfosets<'h, INFO> {
        self.data_for_known_infosets
    }
}

#[derive(Debug)]
pub(crate) struct ThreadLocalWorkStack<'h, INFO: VisibleInfo> {
    stack: Vec<(&'h DataForInfoSet<INFO>, INFO::Gamestate)>,
}

impl<'h, INFO: VisibleInfo> ThreadLocalWorkStack<'h, INFO> {
    pub(crate) fn push(
        &mut self,
        data_for_info_set: &'h DataForInfoSet<INFO>,
        gamestate: INFO::Gamestate,
    ) {
        self.stack.push((data_for_info_set, gamestate));
    }

    pub(crate) fn pop(&mut self) -> Option<(&'h DataForInfoSet<INFO>, INFO::Gamestate)> {
        self.stack.pop()
    }

    pub(crate) fn len(&self) -> usize {
        self.stack.len()
    }

    pub(crate) fn print_debug(&self) {
        for (_, g) in self.stack.iter() {
            println!("\t{:?}", g)
        }
    }
}

impl<'h, INFO: VisibleInfo> Default for ThreadLocalWorkStack<'h, INFO> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}
