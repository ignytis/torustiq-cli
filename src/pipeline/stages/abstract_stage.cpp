#include "abstract_stage.hpp"

#include <spdlog/spdlog.h>

using TorustiqCli::Pipeline::Stages::AbstractStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

AbstractStage::AbstractStage(const PipelineStageDefinition& def)
    : name(def.name), handlerId(def.handler), config(def.config) {}

void AbstractStage::Init() {
    this->stageHandle = this->plugin->createNewStage(this->GetStageKind());
    for (const auto& [key, value] : config) {
        this->plugin->setConfigValue(this->stageHandle, key.c_str(),
                                     value.c_str());
    }
}

string AbstractStage::GetName() const { return name; }

string AbstractStage::GetHandlerId() const { return handlerId; }

void AbstractStage::SetPlugin(StagePlugin* plugin) { this->plugin = plugin; }

void AbstractStage::Start() {
    if (!this->plugin) {
        spdlog::error("Plugin not set for stage: {}", this->GetName());
        return;
    }
    this->plugin->Start(this->stageHandle);
}
