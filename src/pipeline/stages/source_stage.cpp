#include "source_stage.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

#include "../../typedefs/pipeline/stage.hpp"
#include "abstract_stage.hpp"
#include "sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::SenderStage;
using TorustiqCli::Pipeline::Stages::SourceStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

SourceStage::SourceStage(const PipelineStageDefinition& def)
    : SenderStage(def), AbstractStage(def) {}

TorustiqPluginStageKind SourceStage::GetStageKind() {
    return TORUSTIQ_PLUGIN_STAGE_KIND_SOURCE;
}
