#ifndef _TORUSTIQ_CLI_SYSTEM_DLL_H_
#define _TORUSTIQ_CLI_SYSTEM_DLL_H_

#include <string>

#ifdef _WIN32
#include <windows.h>
using LibHandle = HMODULE;
#define DLLEXPORT __declspec(dllexport)
#else
#include <dlfcn.h>
using LibHandle = void*;
#define DLLEXPORT
#endif

namespace TorustiqCli {
namespace System {

#ifdef _WIN32
constexpr const char* kLibFileExtension = ".dll";
#else
constexpr const char* kLibFileExtension = ".so";
#endif

/**
 * A cross-platform wrapper for library loading
 *
 * Example usage:
 *
 * DynamicLibrary lib("myplugin");  // or "myplugin.dll" on Windows if you want
 * explicit
 *
 * using FuncType = void(*)(int);
 * if (lib.IsValid()) {
 *     auto my_func = lib.get<FuncType>("my_function");
 *     if (my_func) my_func(42);
 * }
 */
class DynamicLibrary {
   public:
    explicit DynamicLibrary(const string& name) : library_name_(name) {
#ifdef _WIN32
        handle_ = LoadLibraryA(name.c_str());
#else
        // On Unix-like, try with and without "lib" + ".so" for convenience
        string full_name = name;
        if (full_name.find('/') == string::npos &&
            full_name.find('.') == string::npos) {
            full_name = "lib" + full_name + ".so";
        }
        handle_ = dlopen(full_name.c_str(), RTLD_NOW | RTLD_LOCAL);
#endif
        if (!handle_) {
#ifdef _WIN32
            last_error_ = "Failed to load library: " + name;
#else
            const char* err = dlerror();
            last_error_ = string("Failed to load library: ") + name +
                         (err ? string(" (") + err + ")" : "");
#endif
        }
    }

    ~DynamicLibrary() {
        if (handle_) {
#ifdef _WIN32
            FreeLibrary(handle_);
#else
            dlclose(handle_);
#endif
        }
    }

    bool IsValid() const { return handle_ != nullptr; }

    string GetLastError() const { return last_error_; }

    // Get function pointer (use extern "C" in the library for C linkage)
    // Returns nullptr if symbol not found instead of crashing
    template <typename Func>
    Func get(const string& symbol) const {
        if (!handle_) {
            return nullptr;
        }
#ifdef _WIN32
        auto proc = GetProcAddress(handle_, symbol.c_str());
#else
        auto proc = dlsym(handle_, symbol.c_str());
#endif
        if (!proc) {
#ifdef _WIN32
            const_cast<DynamicLibrary*>(this)->last_error_ =
                "Symbol not found: " + symbol;
#else
            const char* err = dlerror();
            const_cast<DynamicLibrary*>(this)->last_error_ =
                string("Symbol not found: ") + symbol +
                (err ? string(" (") + err + ")" : "");
#endif
            return nullptr;
        }
        return reinterpret_cast<Func>(proc);
    }

   private:
    LibHandle handle_ = nullptr;
    string library_name_;
    string last_error_;

    // Disable copy/move for simplicity (or implement if needed)
    DynamicLibrary(const DynamicLibrary&) = delete;
    DynamicLibrary& operator=(const DynamicLibrary&) = delete;
};

}  // namespace System
}  // namespace TorustiqCli

#endif
