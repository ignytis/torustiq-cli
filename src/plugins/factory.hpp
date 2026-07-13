#ifndef _TORUSTIQ_CLI_PLUGINS_FACTORY_H_
#define _TORUSTIQ_CLI_PLUGINS_FACTORY_H_

#include <string>
#include <vector>

#include "stage_plugin.hpp"

using TorustiqCli::Plugins::StagePlugin;

namespace TorustiqCli {
namespace Plugins {

vector<StagePlugin> LoadPlugins(string moduleDir);

}  // namespace Plugins
}  // namespace TorustiqCli

#endif
