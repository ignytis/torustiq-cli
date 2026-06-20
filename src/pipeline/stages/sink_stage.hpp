#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_SINK_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_SINK_STAGE_H_

#include "../../typedefs/pipeline/stage.hpp"
#include "receiver_stage.hpp"

using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/** A sink stage: accepts input from the previous stage and outputs data without
 * passing it further */
class SinkStage : public virtual ReceiverStage {
   public:
    explicit SinkStage(const PipelineStageDefinition& def);
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_SINK_STAGE_H_
