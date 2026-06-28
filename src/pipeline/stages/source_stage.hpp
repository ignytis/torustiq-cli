#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_SOURCE_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_SOURCE_STAGE_H_

#include "mixins/sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/** A source stage: accepts an incoming payload and forwards it to the next
 * stage */
class SourceStage : public virtual SenderStage {
   public:
    explicit SourceStage(const PipelineStageDefinition& def);
    TorustiqPluginStageKind GetStageKind();
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_SOURCE_STAGE_H_
