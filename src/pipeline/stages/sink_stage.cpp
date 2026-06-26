#include "sink_stage.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

#include "../../typedefs/pipeline/stage.hpp"
#include "abstract_stage.hpp"
#include "receiver_stage.hpp"

using TorustiqCli::Pipeline::Stages::AbstractStage;
using TorustiqCli::Pipeline::Stages::ReceiverStage;
using TorustiqCli::Pipeline::Stages::SinkStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

SinkStage::SinkStage(const PipelineStageDefinition& def)
    : ReceiverStage(def), AbstractStage(def) {}

TorustiqPluginStageKind SinkStage::GetStageKind() {
    return TORUSTIQ_PLUGIN_STAGE_KIND_SINK;
}
