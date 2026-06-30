#ifndef _TORUSTIQ_CLI_COMMANDS_RUN_GLOBALS_H_
#define _TORUSTIQ_CLI_COMMANDS_RUN_GLOBALS_H_

/**
 * This file contains definitions of global variables in scope of the Run
 * command
 */

#include <torustiq_sdk/plugins/typedefs.h>

#include "../../pipeline/pipeline.hpp"

using TorustiqCli::Pipeline::Pipeline;

void setPipeline(Pipeline* pipeline);

void onMessageReceived(TorustiqPluginStageHandle stageHandle,
                       const TorustiqMessage* message);

#endif
