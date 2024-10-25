use crate::cfr::game_model::{OracleGamestate, VisibleInfo};
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::data_for_known_infosets::DataForKnownInfosets;
use crate::cfr::strategy_generation::workspace_data::timestamp::Timestamp;
use crossbeam::queue::SegQueue;
use std::cell::{RefCell, RefMut};
use std::sync::Arc;
use thread_local::ThreadLocal;

pub(crate) mod batch_item_data;
pub(crate) mod data_for_infoset;
pub(crate) mod data_for_known_infosets;
pub(crate) mod data_for_move;
mod move_data;
pub(crate) mod timestamp;

pub(crate) struct StrategyGenerationProgress<INFO: VisibleInfo> {
    data_for_known_infosets: DataForKnownInfosets<INFO>,
    thread_local_workstack: ThreadLocal<RefCell<ThreadLocalWorkStack<INFO>>>,
    updated_infosets: SegQueue<Arc<DataForInfoSet<INFO>>>,
    iter_count: usize,
}

impl<INFO: VisibleInfo> StrategyGenerationProgress<INFO> {
    pub(crate) fn new(iter_count: usize) -> Self {
        Self {
            data_for_known_infosets: DataForKnownInfosets::default(),
            thread_local_workstack: ThreadLocal::new(),
            updated_infosets: Default::default(),
            iter_count,
        }
    }

    pub(crate) fn thread_local_workstack(&self) -> RefMut<ThreadLocalWorkStack<INFO>> {
        self.thread_local_workstack.get_or_default().borrow_mut()
    }

    pub(crate) fn mark_as_completed(
        &self,
        data_for_info_set: Arc<DataForInfoSet<INFO>>,
        timestamp: Timestamp,
    ) {
        if data_for_info_set.check_for_writeback(timestamp) {
            self.updated_infosets.push(data_for_info_set);
        }
    }

    pub(crate) fn get_data_for_infoset(&self, data: INFO) -> Arc<DataForInfoSet<INFO>> {
        self.data_for_known_infosets.data_for_infoset(data)
    }

    #[inline]
    pub(crate) fn consume_updated_infosets(
        &self,
        f: impl Fn(Arc<DataForInfoSet<INFO>>) + Send + Sync,
    ) {
        rayon::scope(|x| {
            while let Some(i) = self.updated_infosets.pop() {
                x.spawn(|_| f(i));
            }
        });
    }

    pub(crate) fn into_infoset_data(self) -> DataForKnownInfosets<INFO> {
        self.data_for_known_infosets
    }
}

#[derive(Debug)]
pub(crate) struct ThreadLocalWorkStack<INFO: VisibleInfo> {
    stack: Vec<(Arc<DataForInfoSet<INFO>>, INFO::Gamestate)>,
}

impl<INFO: VisibleInfo> ThreadLocalWorkStack<INFO> {
    pub(crate) fn push(
        &mut self,
        data_for_info_set: Arc<DataForInfoSet<INFO>>,
        gamestate: INFO::Gamestate,
    ) {
        self.stack.push((data_for_info_set, gamestate));
    }

    pub(crate) fn push_default(&mut self, gamestate: INFO::Gamestate) {
        // let mut uninit = Arc::new_uninit();
        //
        // let arc = unsafe {
        //     Arc::get_mut(&mut uninit).unwrap().write(
        //         DataForInfoSet::new(&gamestate.info_for_turn_player())
        //     );
        //
        //     uninit.assume_init()
        // };

        let arc = Arc::new(DataForInfoSet::new(&gamestate.info_for_turn_player()));

        self.stack.push((arc, gamestate));
    }

    pub(crate) fn pop(&mut self) -> Option<(Arc<DataForInfoSet<INFO>>, INFO::Gamestate)> {
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

impl<INFO: VisibleInfo> Default for ThreadLocalWorkStack<INFO> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}
