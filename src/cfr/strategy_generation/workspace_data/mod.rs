use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForKnownInfosets;

mod data_for_infoset;

pub(crate) struct StrategyGenerationProgress<INFO: VisibleInfo> {
    data_for_known_infosets: DataForKnownInfosets<INFO>
}