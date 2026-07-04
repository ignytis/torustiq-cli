#include "pipeline.hpp"

#include <spdlog/spdlog.h>

#include <ranges>
#include <stdexcept>
#include <string>
#include <unordered_set>

#include "stages/mixins/receiver_stage.hpp"
#include "stages/mixins/sender_stage.hpp"
#include "stages/processor_stage.hpp"
#include "stages/sink_stage.hpp"
#include "stages/source_stage.hpp"

using namespace std;
using namespace std::ranges;

using namespace TorustiqCli::Pipeline;
using namespace TorustiqCli::Typedefs::Pipeline;

using TorustiqCli::Pipeline::Stages::Mixins::ReceiverStage;
using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;

Pipeline::Pipeline(const PipelineDefinition& def) {
    size_t count = def.stages.size();
    if (count < 2) {
        spdlog::error("Pipeline must have at least a source and a sink stage");
        exit(1);
    }
    stages.reserve(count);

    for (size_t i = 0; i < count; ++i) {
        const PipelineStageDefinition& stageDef = def.stages[i];
        Stages::AbstractStage* stage =
            (i == 0)           ? static_cast<Stages::AbstractStage*>(
                                     new Stages::SourceStage(stageDef))
            : (i == count - 1) ? static_cast<Stages::AbstractStage*>(
                                     new Stages::SinkStage(stageDef))
                               : static_cast<Stages::AbstractStage*>(
                                     new Stages::ProcessorStage(stageDef));
        stages.push_back(stage);
    }
}

unordered_set<string> Pipeline::getHandlersInUse() {
    return stages | views::transform([](const Stages::AbstractStage* stage) {
               return stage->GetHandlerId();
           }) |
           ranges::to<unordered_set<string>>();
}

void Pipeline::setPlugins(vector<TorustiqCli::Plugins::StagePlugin>& plugins) {
    for (Stages::AbstractStage* stage : stages) {
        for (TorustiqCli::Plugins::StagePlugin& plugin : plugins) {
            if (stage->GetHandlerId() != plugin.GetId()) {
                continue;
            }
            stage->SetPlugin(&plugin);
            spdlog::debug("Assigned plugin [{}] to stage '{}'.", plugin.GetId(),
                          stage->GetName());
            break;
        }
    }
}

void Pipeline::initStages() {
    SenderStage* prevSenderStage = nullptr;
    TorustiqPluginStageHandle stageHandle = 0;
    for (Stages::AbstractStage* stage : stages) {
        stage->Init(stageHandle);
        spdlog::debug("Stage '{}' initialized.", stage->GetName());

        // Connect the output of the previous sender stage to the input of the
        // current receiver stage
        if (prevSenderStage != nullptr) {
            ReceiverStage* stageRcvr = dynamic_cast<ReceiverStage*>(stage);
            prevSenderStage->SetOutputQueuePtr(stageRcvr->GetInputQueuePtr());
            spdlog::debug(
                "Connected output of stage '{}' to input of stage '{}'.",
                prevSenderStage->GetName(), stageRcvr->GetName());
        }

        prevSenderStage = dynamic_cast<SenderStage*>(stage);
        stageHandle++;
    }
}

void Pipeline::start() {
    spdlog::debug("Starting the pipeline...");
    spdlog::debug("Starting stages in reverse order...");
    vector<thread> stageThreads;
    stageThreads.reserve(stages.size());
    for (int i = stages.size() - 1; i >= 0; i--) {
        Stages::AbstractStage* stage = stages.at(i);
        stageThreads.emplace_back([stage]() { stage->Start(); });
        spdlog::debug("Stage #{} '{}' started.", i, stage->GetName());
    }
    spdlog::debug("Pipeline started.");

    for (thread& stageThread : stageThreads) {
        if (stageThread.joinable()) {
            stageThread.join();
        }
    }
    spdlog::debug("All stages have completed execution. Pipeline terminated.");
}

Stages::AbstractStage* Pipeline::GetStagePtrByHandle(
    TorustiqPluginStageHandle handle) {
    return stages.at(handle);
}
