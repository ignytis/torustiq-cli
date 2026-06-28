#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_MIXINS_RECEIVER_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_MIXINS_RECEIVER_STAGE_H_

#include "../abstract_stage.hpp"

using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {
namespace Mixins {

/**
 * @brief A receiver stage: accepts an incoming payload from the previous stage
 * and processes it.
 */
class ReceiverStage : virtual public AbstractStage {
   public:
    explicit ReceiverStage(
        const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def);

    TSQueue<TorustiqMessage>* GetInputQueuePtr();

   protected:
    /** The input queue for receiving messages */
    TSQueue<TorustiqMessage> inputQueue;
};

}  // namespace Mixins
}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_MIXINS_RECEIVER_STAGE_H_
