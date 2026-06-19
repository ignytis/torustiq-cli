#include "pipeline.hpp"

#include <spdlog/spdlog.h>

#include <ranges>
#include <stdexcept>
#include <string>
#include <unordered_set>

#include "stages/processor_stage.hpp"
#include "stages/sink_stage.hpp"
#include "stages/source_stage.hpp"

using namespace std;
using namespace std::ranges;

using namespace TorustiqCli::Pipeline;
using namespace TorustiqCli::Typedefs::Pipeline;

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
    for (Stages::AbstractStage* stage : stages) {
        stage->init();
        spdlog::debug("Stage '{}' initialized.", stage->GetName());
    }
}

void Pipeline::start() {
    spdlog::debug("Starting the pipeline...");
    spdlog::debug("Starting stages in reverse order...");
    for (int i = stages.size() - 1; i >= 0; i--) {
        Stages::AbstractStage* stage = stages.at(i);
        stage->Start();
        spdlog::debug("Stage #{} '{}' started.", i, stage->GetName());
    }

    spdlog::debug("Pipeline started.");
}
