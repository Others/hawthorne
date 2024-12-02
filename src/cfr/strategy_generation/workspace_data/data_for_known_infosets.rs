use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use bumpalo_herd::Member;
use dashmap::{DashMap, Entry};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

pub(crate) struct DataForKnownInfosets<'h, INFO: VisibleInfo> {
    infoset_data: DashMap<INFO, &'h DataForInfoSet<INFO>, BuildHasherDefault<FxHasher>>,
}

impl<'h, INFO: VisibleInfo> DataForKnownInfosets<'h, INFO> {
    pub(crate) fn data_for_infoset(
        &self,
        info: INFO,
        member: &Member<'h>,
    ) -> &'h DataForInfoSet<INFO> {
        match self.infoset_data.entry(info) {
            Entry::Occupied(a) => a.get(),
            Entry::Vacant(v) => {
                let h = &*member.alloc_with(|| DataForInfoSet::new(v.key()));

                v.insert(h);
                h
            }
        }

        // let key_hash = self.infoset_data.hash_usize(info);
        // let key_shard = self.infoset_data.determine_shard(key_hash);
        // let shard = &self.infoset_data.shards()[key_shard];
        //
        // let mut shard_guard = shard.write();
        // let f = |(k, v)| {
        //     k == &info
        // };
        // let s = shard_guard.find_or_find_insert_slot(
        //     key_hash as u64,
        //     &f,
        //
        // );
        //
        // let shards = self.infoset_data.shards();
        // self.infoset_data.determine_shard()
        //
        // self.infoset_data.shards().get(&info).cloned().unwrap_or_default()
    }
}

impl<'h, INFO: VisibleInfo> Default for DataForKnownInfosets<'h, INFO> {
    fn default() -> Self {
        Self {
            infoset_data: Default::default(),
        }
    }
}
