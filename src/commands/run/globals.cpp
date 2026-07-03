#include "globals.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

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
    // TODO: deleteme
    spdlog::debug("Message globals :: Sent a message from handle {}", stageHandle);

    if (_PIPELINE == nullptr) {
        spdlog::error("Message globals :: messageSender :: pipeline is not initialized");
        return;
    }

    // As stageHandle is sender's handle, receiver is stage is the next one
    ReceiverStage* receiverStage = dynamic_cast<ReceiverStage*>(
        _PIPELINE->GetStagePtrByHandle(stageHandle + 1));
    if (receiverStage == nullptr) {
        spdlog::error("Message globals :: messageSender :: stage with handle {} is not a sender stage", stageHandle);
        return;
    }

    receiverStage->PushMessage(message);
}

const TorustiqMessage* messageReceiver(TorustiqPluginStageHandle stageHandle) {
    // TODO: deleteme
    spdlog::debug("Message globals :: Received a message in handle {}", stageHandle);

    if (_PIPELINE == nullptr) {
        spdlog::error("Message globals :: messageReceiver :: pipeline is not initialized");
        return nullptr;
    }

    // As stageHandle is sender's handle, receiver is stage is the next one
    ReceiverStage* receiverStage = dynamic_cast<ReceiverStage*>(
        _PIPELINE->GetStagePtrByHandle(stageHandle));
    if (receiverStage == nullptr) {
        spdlog::error("Message globals :: messageReceiver :: stage with handle {} is not a receiver stage", stageHandle);
        return nullptr;
    }
    spdlog::debug("Message globals :: messageReceiver :: sending a message from handerl {}", stageHandle);

    return receiverStage->PopMessage();
}