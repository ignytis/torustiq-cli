#include "receiver_stage.hpp"

using TorustiqCli::Pipeline::Stages::Mixins::ReceiverStage;

ReceiverStage::ReceiverStage(const PipelineStageDefinition& def)
    : AbstractStage(def) {}

TSQueue<const TorustiqMessage*>* ReceiverStage::GetInputQueuePtr() {
    return &inputQueue;
}

void ReceiverStage::PushMessage(const TorustiqMessage* message) {
    inputQueue.push(message);
}

const TorustiqMessage* ReceiverStage::PopMessage() { return inputQueue.pop(); }
