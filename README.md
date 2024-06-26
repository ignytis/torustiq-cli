# torustiq-cli

## Trivia

Torustiq (Estonian 🇪🇪: _torustik_, a pipeline) is a modular and serverless tool to build the data processing pipelines. All what you need is to put all the required modules into some directory, define the pipeline using YAML and run the app

## Status

WIP; NOT production-ready. Currently I'm working on implementation of core features and few general-purpose modules like HTTP, standatd i/o, filesystem and Python plugins.

## Vision

- Fast and memory-efficient
  - Crucial for intense workloads. That's why it's written in Rust
  - No interruptions / "sleeps" inside data processing loops to poll the data. The data must be pushed to the next module and handled immediately 
- Modular
  - External services can be added via plugins
  - Language-agnostic. The [Foreign Function Interface](https://en.wikipedia.org/wiki/Foreign_function_interface) is used to communicate with modules written in Rust, C or any other language which compiles libraries into DLL, SO or any other system library format
- Simple
  - One process only
    - NO servers (web UI, metrics, analytics, whatever), unless user adds them as modules
  - Ideally the working system is:
    - The application binary
    - `2..n` library files: input, output and `n-2` transformations. No transformations is also possible e.g. for data dumps
    - A pipeline definition YAML file
      - Need another pipeline to process? Just add a new YAML file
    - Installed system dependencies, if any
  - One pipeline per process. It is NOT a pipeline orchestration system

## Roadmap

### Version 1
- Modular system
- Records contain the content and metadata
- A simple pipeline with single input, output and any number of transformation modules in between. NO branching (i.e. splits / joins of the data flow) at this point
- Basic modules for IO and transforms
  - Filesystem
  - HTTP
  - Standard I/O
  - Kafka (???)

### Version 2 (just few ideas)
- Simple brancing (splits only; joins are more complex to implement)

## Known issues

As the project is on the early stage, there are some problems in the code that I would like to fix later:

- Memory leaks. Memory is dynamically allocated to store input, output and intermediate results. The static garbage collector of Rust cannot cover those cases, because _unsafe_ code is used for cross-library communnication. Need to review the memory allocation logic and to add the manual de-allocation wherever it is needed
- Error handling. The application "panics" if any error occurs, but in some cases errors should be ignored or just reported to error log
- Some functions like `torustiq_module_step_set_param` are copypasted across modules. Could this be replaced with imported single module?

## TODO

_Some of these points might repeat the "known issues" section_

- Pass parameters from YAML pipeline definition into modules
- Handle errors. Instead of _expect_ function call use matching + print more detailed error messages and do NOT terminate the app if error is not critical
- Deallocate the memory for dynamically created objects which are shared between the app and modules
- Use modules as both source and destination (if applicable). Splitting modules into sources and destination causes having too much components