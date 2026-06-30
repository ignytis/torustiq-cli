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

void onMessageReceived(TorustiqPluginStageHandle stageHandle,
                       const TorustiqMessage* message) {
    // TODO: deleteme
    spdlog::debug("Received a message");

    if (_PIPELINE == nullptr) {
        // TODO: log error
        return;
    }

    // As stageHandle is sender's handle, receiver is stage is the next one
    ReceiverStage* receiverStage = dynamic_cast<ReceiverStage*>(
        _PIPELINE->GetStagePtrByHandle(stageHandle + 1));
    if (receiverStage == nullptr) {
        // TODO: log error
        return;
    }

    receiverStage->OnMessageReceived(message);
}
