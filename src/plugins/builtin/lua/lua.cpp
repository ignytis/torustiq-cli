#include "lua.hpp"

#include <cstring>
#include <fstream>
#include <map>
#include <memory>
#include <string>

extern "C" {
#include <lauxlib.h>
#include <lua.h>
#include <lualib.h>
}

#include <spdlog/spdlog.h>

#include <torustiq_sdk/message.h>
#include <torustiq_sdk/typedefs.h>

#include "../../../defs.hpp"

using namespace std;

namespace {

// Forward declarations
class StageInstance;
extern map<TorustiqPluginStageHandle, StageInstance> stageInstances;
extern TorustiqHostGlobals hostGlobals;

// Wrapper function for sendMessage to be called from Lua
int luaSendMessage(lua_State* L) {
    // Get the stage handle from the Lua registry
    lua_getfield(L, LUA_REGISTRYINDEX, "stage_handle");
    TorustiqPluginStageHandle stageHandle = lua_tointeger(L, -1);
    lua_pop(L, 1);

    // Get the message payload from Lua argument
    size_t payloadSize = 0;
    const char* payloadStr = luaL_checklstring(L, 1, &payloadSize);

    // Create message
    TorustiqMessage msg{};
    msg.type = TORUSTIQ_MESSAGE_TYPE_DATA;
    msg.payload_size = payloadSize;
    msg.payload = reinterpret_cast<uint8_t*>(malloc(payloadSize));
    if (msg.payload != nullptr) {
        memcpy(msg.payload, payloadStr, payloadSize);
    }
    msg.headers_count = 0;
    msg.headers = nullptr;

    // Send the message
    hostGlobals.sendMessageFnPtr(stageHandle, &msg);
    free(msg.payload);

    return 0;
}

// Wrapper function for receiveMessage to be called from Lua
int luaReceiveMessage(lua_State* L) {
    // Get the stage handle from the Lua registry
    lua_getfield(L, LUA_REGISTRYINDEX, "stage_handle");
    TorustiqPluginStageHandle stageHandle = lua_tointeger(L, -1);
    lua_pop(L, 1);

    // Receive the message
    const TorustiqMessage* msg = hostGlobals.receiveMessageFnPtr(stageHandle);

    if (msg == nullptr) {
        lua_pushnil(L);
        return 1;
    }

    // Create a Lua table to represent the message
    lua_newtable(L);

    // Add message type
    lua_pushinteger(L, msg->type);
    lua_setfield(L, -2, "type");

    // Add payload if it exists and is data
    if (msg->type == TORUSTIQ_MESSAGE_TYPE_DATA && msg->payload != nullptr) {
        lua_pushlstring(L, reinterpret_cast<const char*>(msg->payload),
                        msg->payload_size);
        lua_setfield(L, -2, "payload");
    }

    // Free the message
    torustiq_message_free(const_cast<TorustiqMessage*>(msg));

    return 1;
}

/**
 * Represents an instance of a Lua stage.
 */
class StageInstance {
   public:
    StageInstance();
    StageInstance(TorustiqPluginStageHandle handle,
                  TorustiqPluginStageKind kind);
    ~StageInstance();

    StageInstance(const StageInstance& other) = delete;
    StageInstance& operator=(const StageInstance& other) = delete;

    StageInstance(StageInstance&& other) noexcept;
    StageInstance& operator=(StageInstance&& other) noexcept;

    TorustiqPluginStageHandle stageHandle;
    TorustiqPluginStageKind stageKind;
    string scriptPath;
    string scriptContent;
    lua_State* luaState;

    bool loadLuaScript();
};

StageInstance::StageInstance()
    : stageHandle(0),
      stageKind(TORUSTIQ_PLUGIN_STAGE_KIND_SOURCE),
      luaState(nullptr) {}

StageInstance::StageInstance(TorustiqPluginStageHandle handle,
                             TorustiqPluginStageKind kind)
    : stageHandle(handle), stageKind(kind), luaState(nullptr) {
    luaState = luaL_newstate();
    if (luaState != nullptr) {
        luaL_openlibs(luaState);

        // Register sendMessage function
        lua_register(luaState, "sendMessage", luaSendMessage);

        // Register receiveMessage function
        lua_register(luaState, "receiveMessage", luaReceiveMessage);

        // Store the stage handle in registry for access by Lua functions
        lua_pushinteger(luaState, handle);
        lua_setfield(luaState, LUA_REGISTRYINDEX, "stage_handle");
    }
}

StageInstance::~StageInstance() {
    if (luaState != nullptr) {
        lua_close(luaState);
    }
}

StageInstance::StageInstance(StageInstance&& other) noexcept
    : stageHandle(other.stageHandle),
      stageKind(other.stageKind),
      scriptPath(move(other.scriptPath)),
      scriptContent(move(other.scriptContent)),
      luaState(other.luaState) {
    other.luaState = nullptr;
}

StageInstance& StageInstance::operator=(StageInstance&& other) noexcept {
    if (this != &other) {
        if (luaState != nullptr) {
            lua_close(luaState);
        }
        stageHandle = other.stageHandle;
        stageKind = other.stageKind;
        scriptPath = move(other.scriptPath);
        scriptContent = move(other.scriptContent);
        luaState = other.luaState;
        other.luaState = nullptr;
    }
    return *this;
}

bool StageInstance::loadLuaScript() {
    if (luaState == nullptr) {
        spdlog::error("lua :: Lua state is null");
        return false;
    }

    int result = 0;
    if (!scriptPath.empty()) {
        // Load from file
        result = luaL_dofile(luaState, scriptPath.c_str());
    } else if (!scriptContent.empty()) {
        // Load from inline content
        result = luaL_dostring(luaState, scriptContent.c_str());
    } else {
        spdlog::error("lua :: No script path or content provided");
        return false;
    }

    if (result != LUA_OK) {
        const char* error = lua_tostring(luaState, -1);
        spdlog::error("lua :: Lua error: {}", error ? error : "Unknown error");
        lua_pop(luaState, 1);
        return false;
    }

    return true;
}

map<TorustiqPluginStageHandle, StageInstance> stageInstances;
TorustiqHostGlobals hostGlobals;

void startSource(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    spdlog::debug("lua :: Starting source stage with handle {}", stageHandle);

    if (!instance->loadLuaScript()) {
        return;
    }

    // Call the run() function if it exists
    lua_getglobal(instance->luaState, "run");
    if (lua_isfunction(instance->luaState, -1)) {
        int result = lua_pcall(instance->luaState, 0, 0, 0);
        if (result != LUA_OK) {
            const char* error = lua_tostring(instance->luaState, -1);
            spdlog::error("lua :: Error running source script: {}",
                          error ? error : "Unknown error");
            lua_pop(instance->luaState, 1);
        }
    } else {
        spdlog::warn("lua :: run() function not found in source script");
        lua_pop(instance->luaState, 1);
    }

    // Send EOF message
    TorustiqMessage msg{};
    msg.type = TORUSTIQ_MESSAGE_TYPE_EOF;
    hostGlobals.sendMessageFnPtr(stageHandle, &msg);
}

void startProcessor(TorustiqPluginStageHandle stageHandle,
                    StageInstance* instance) {
    spdlog::debug("lua :: Starting processor stage with handle {}",
                  stageHandle);

    if (!instance->loadLuaScript()) {
        return;
    }

    while (true) {
        const TorustiqMessage* msg =
            hostGlobals.receiveMessageFnPtr(stageHandle);
        if (msg == nullptr) {
            spdlog::error("lua :: Empty pointer received");
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_EOF) {
            spdlog::debug("lua :: Received EOF message in processor");
            torustiq_message_free(const_cast<TorustiqMessage*>(msg));

            TorustiqMessage eofMsg = torustiq_message_create_eof();
            hostGlobals.sendMessageFnPtr(stageHandle, &eofMsg);
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_DATA) {
            // Call the process() function with the message payload
            lua_getglobal(instance->luaState, "process");
            if (lua_isfunction(instance->luaState, -1)) {
                lua_pushlstring(instance->luaState,
                                reinterpret_cast<const char*>(msg->payload),
                                msg->payload_size);
                int result = lua_pcall(instance->luaState, 1, 0, 0);
                if (result != LUA_OK) {
                    const char* error = lua_tostring(instance->luaState, -1);
                    spdlog::error("lua :: Error processing message: {}",
                                  error ? error : "Unknown error");
                    lua_pop(instance->luaState, 1);
                }
            } else {
                spdlog::warn("lua :: process() function not found");
                lua_pop(instance->luaState, 1);
            }
        }

        torustiq_message_free(const_cast<TorustiqMessage*>(msg));
    }
}

void startSink(TorustiqPluginStageHandle stageHandle, StageInstance* instance) {
    spdlog::debug("lua :: Starting sink stage with handle {}", stageHandle);

    if (!instance->loadLuaScript()) {
        return;
    }

    while (true) {
        const TorustiqMessage* msg =
            hostGlobals.receiveMessageFnPtr(stageHandle);
        if (msg == nullptr) {
            spdlog::error("lua :: Empty pointer received");
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_EOF) {
            spdlog::debug("lua :: Received EOF message in sink");
            torustiq_message_free(const_cast<TorustiqMessage*>(msg));
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_DATA) {
            // Call the write() function with the message payload
            lua_getglobal(instance->luaState, "write");
            if (lua_isfunction(instance->luaState, -1)) {
                lua_pushlstring(instance->luaState,
                                reinterpret_cast<const char*>(msg->payload),
                                msg->payload_size);
                int result = lua_pcall(instance->luaState, 1, 0, 0);
                if (result != LUA_OK) {
                    const char* error = lua_tostring(instance->luaState, -1);
                    spdlog::error("lua :: Error writing message: {}",
                                  error ? error : "Unknown error");
                    lua_pop(instance->luaState, 1);
                }
            } else {
                spdlog::warn("lua :: write() function not found");
                lua_pop(instance->luaState, 1);
            }
        }

        torustiq_message_free(const_cast<TorustiqMessage*>(msg));
    }
}

}  // namespace

const TorustiqPluginInfo TorustiqCli::Plugins::Builtin::Lua::GetPluginInfo() {
    return TorustiqPluginInfo{
        .host_app = APP_NAME,
        .api_version = API_VERSION,
        .id = "lua",
        .name = "Lua Plugin",
    };
}

void TorustiqCli::Plugins::Builtin::Lua::CreateNewStage(
    CreateNewStageFnArgs args) {
    StageInstance newInstance(args.stageHandle, args.stageKind);
    stageInstances.emplace(args.stageHandle, move(newInstance));
}

void TorustiqCli::Plugins::Builtin::Lua::SetStageConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances[stageHandle];
    if (string(key) == "path") {
        instance.scriptPath = string(value);
    } else if (string(key) == "content") {
        instance.scriptContent = string(value);
    }
}

void TorustiqCli::Plugins::Builtin::Lua::Start(
    TorustiqPluginStageHandle stageHandle) {
    if (!stageInstances.contains(stageHandle)) {
        spdlog::error("lua :: Stage handle not found: {}", stageHandle);
        return;
    }

    StageInstance& instance = stageInstances[stageHandle];

    switch (instance.stageKind) {
        case TORUSTIQ_PLUGIN_STAGE_KIND_SOURCE:
            startSource(stageHandle, &instance);
            break;
        case TORUSTIQ_PLUGIN_STAGE_KIND_PROCESSOR:
            startProcessor(stageHandle, &instance);
            break;
        case TORUSTIQ_PLUGIN_STAGE_KIND_SINK:
            startSink(stageHandle, &instance);
            break;
        default:
            spdlog::error("lua :: Unknown stage kind: {}",
                          static_cast<int>(instance.stageKind));
    }
}

const TorustiqPlugin TorustiqCli::Plugins::Builtin::Lua::InitPlugin(
    TorustiqHostGlobals globals) {
    hostGlobals = globals;
    return TorustiqPlugin{
        .fn_stage_create_new = CreateNewStage,
        .fn_stage_set_config_value = SetStageConfigValue,
        .fn_stage_start = Start,
    };
}
