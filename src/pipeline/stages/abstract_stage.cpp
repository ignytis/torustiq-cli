#include "abstract_stage.hpp"

#include <spdlog/spdlog.h>

using TorustiqCli::Pipeline::Stages::AbstractStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

AbstractStage::AbstractStage(const PipelineStageDefinition& def)
    : name(def.name), handlerId(def.handler), config(def.config) {}

void AbstractStage::Init() { this->plugin->createNewStage(); }

string AbstractStage::GetName() const { return name; }

string AbstractStage::GetHandlerId() const { return handlerId; }

void AbstractStage::SetPlugin(StagePlugin* plugin) { this->plugin = plugin; }

void AbstractStage::Start() {}
