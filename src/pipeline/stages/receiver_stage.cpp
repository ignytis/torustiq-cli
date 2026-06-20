#include "receiver_stage.hpp"

using TorustiqCli::Pipeline::Stages::ReceiverStage;

ReceiverStage::ReceiverStage(const PipelineStageDefinition& def)
    : AbstractStage(def) {}

TSQueue<TorustiqMessage>* ReceiverStage::GetInputQueuePtr() {
    return &inputQueue;
}
