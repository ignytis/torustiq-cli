#include "processor_stage.hpp"

#include "../../typedefs/pipeline/stage.hpp"
#include "abstract_stage.hpp"
#include "mixins/receiver_stage.hpp"
#include "mixins/sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::AbstractStage;
using TorustiqCli::Pipeline::Stages::ProcessorStage;
using TorustiqCli::Pipeline::Stages::Mixins::ReceiverStage;
using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;

ProcessorStage::ProcessorStage(
    const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def)
    : ReceiverStage(def), SenderStage(def), AbstractStage(def) {}

TorustiqPluginStageKind ProcessorStage::GetStageKind() {
    return TORUSTIQ_PLUGIN_STAGE_KIND_PROCESSOR;
}
