#include "globals.hpp"

#include <torustiq_sdk/message.h>
#include <torustiq_sdk/typedefs.h>

#include "../../pipeline/pipeline.hpp"
#include "../../pipeline/stages/abstract_stage.hpp"
#include "../../pipeline/stages/mixins/receiver_stage.hpp"

using TorustiqCli::Pipeline::Stages::AbstractStage;
using TorustiqCli::Pipeline::Stages::Mixins::ReceiverStage;

Pipeline* _PIPELINE = nullptr;

#include <spdlog/spdlog.h>

void setPipeline(Pipeline* pipeline) { _PIPELINE = pipeline; }

void messageSender(TorustiqPluginStageHandle stageHandle,
                   const TorustiqMessage* message) {
    if (_PIPELINE == nullptr) {
        spdlog::error(
            "Message globals :: messageSender :: pipeline is not initialized");
        return;
    }

    // As stageHandle is sender's handle, receiver is stage is the next one
    ReceiverStage* receiverStage = dynamic_cast<ReceiverStage*>(
        _PIPELINE->GetStagePtrByHandle(stageHandle + 1));
    if (receiverStage == nullptr) {
        spdlog::error(
            "Message globals :: messageSender :: stage with handle {} is not a "
            "sender stage",
            stageHandle);
        return;
    }

    // Create a copy of the message where all the data is owned by the receiver
    // stage. The sender stage will free its own message after sending it.
    TorustiqMessage* messageOwned = torustiq_message_clone(message);
    receiverStage->PushMessage(messageOwned);
}

const TorustiqMessage* messageReceiver(TorustiqPluginStageHandle stageHandle) {
    if (_PIPELINE == nullptr) {
        spdlog::error(
            "Message globals :: messageReceiver :: pipeline is not "
            "initialized");
        return nullptr;
    }

    // As stageHandle is sender's handle, receiver is stage is the next one
    ReceiverStage* receiverStage = dynamic_cast<ReceiverStage*>(
        _PIPELINE->GetStagePtrByHandle(stageHandle));
    if (receiverStage == nullptr) {
        spdlog::error(
            "Message globals :: messageReceiver :: stage with handle {} is not "
            "a receiver stage",
            stageHandle);
        return nullptr;
    }

    return receiverStage->PopMessage();
}
