#include "source_stage.hpp"

#include "../../typedefs/pipeline/stage.hpp"
#include "abstract_stage.hpp"
#include "mixins/sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::SourceStage;
using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

SourceStage::SourceStage(const PipelineStageDefinition& def)
    : SenderStage(def), AbstractStage(def) {}

TorustiqPluginStageKind SourceStage::GetStageKind() {
    return TORUSTIQ_PLUGIN_STAGE_KIND_SOURCE;
}
