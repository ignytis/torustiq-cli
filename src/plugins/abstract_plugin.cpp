#include "abstract_plugin.hpp"

using namespace TorustiqCli::Plugins;

AbstractPlugin::AbstractPlugin(AbstractPluginConstructorArgs args)
    : InitPtr(args.init_fn_ptr), GetInfoPtr(args.get_info_fn_ptr) {
    if (!InitPtr) {
        InitPtr = []() {};
    }
}

void AbstractPlugin::init() { InitPtr(); }

string AbstractPlugin::GetId() const { return GetInfoPtr().id; }

string AbstractPlugin::GetName() const { return GetInfoPtr().name; }
