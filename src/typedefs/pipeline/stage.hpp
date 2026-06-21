#ifndef _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_STAGE_H_
#define _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_STAGE_H_

#include <yaml-cpp/yaml.h>

#include <map>
#include <string>

using namespace std;

typedef map<string, string> ConfigKV;

namespace TorustiqCli {
namespace Typedefs {
namespace Pipeline {

/** Represents a single stage in a pipeline definition */
class PipelineStageDefinition {
   public:
    string name;
    string handler;
    ConfigKV config;
};

}  // namespace Pipeline
}  // namespace Typedefs
}  // namespace TorustiqCli

namespace YAML {
using namespace TorustiqCli::Typedefs::Pipeline;

template <>
struct convert<PipelineStageDefinition> {
    static Node encode(const PipelineStageDefinition& stage) {
        Node node;
        node["name"] = stage.name;
        node["handler"] = stage.handler;
        for (const pair<const string, string>& kv : stage.config) {
            node["config"][kv.first] = kv.second;
        }
        return node;
    }

    static bool decode(const Node& node, PipelineStageDefinition& stage) {
        if (!node.IsMap()) {
            return false;
        }
        stage.name = node["name"].as<string>();
        stage.handler = node["handler"].as<string>();
        if (node["config"]) {
            for (YAML::const_iterator it = node["config"].begin();
                 it != node["config"].end(); ++it) {
                stage.config[it->first.as<string>()] = it->second.as<string>();
            }
        }
        return true;
    }
};
}  // namespace YAML

#endif  // _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_STAGE_H_
