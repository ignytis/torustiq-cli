#include "sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;

SenderStage::SenderStage(
    const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def)
    : AbstractStage(def) {}

void SenderStage::SetOutputQueuePtr(TSQueue<TorustiqMessage>* queue) {
    this->outputQueue = queue;
}
