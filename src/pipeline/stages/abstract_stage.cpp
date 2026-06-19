#include "abstract_stage.hpp"

using TorustiqCli::Pipeline::Stages::AbstractStage;

AbstractStage::AbstractStage(
    const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def)
    : name(def.name), handlerId(def.handler), config(def.config) {}

string AbstractStage::GetName() const { return name; }

string AbstractStage::GetHandlerId() const { return handlerId; }

void AbstractStage::SetPlugin(StagePlugin* plugin) { this->plugin = plugin; }
