# Definitions

__Library__ - a dynamic library file (Linux: `*.so`, Windows: `*.dll`) which is pluggable to Torustiq app.

__Module__ - a component inside pipeline which is based on library and uses functions from there.
One library might be used to initialize zero or more modules.

__Pipeline module__ - a module which takes part in data transforms. There are 3 kinds of pipeline modules: Source, Transformation and Destination.

__Source pipeline module__ - a module which reads the data from external source and sends it to the next step in pipeline. Source is always the first pipeline module.

__Transformation pipeline module__ - a module which reads input from the previous module (source or transformation) and sends the processed data to the next module (transformation or destination). Input and output are not _1:1_: one input might result in multiple outputs and vice versa.

__Destination pipeline module__ - a module which writes the data to external destination. Destination is always the last pipeline module.

__Listener module__ - a module which doesn't process any data, but handles application events instead.