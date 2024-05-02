# Liaison

## Build

1. Build the dynamic libraries with CMake.
2. Make the Liaision FMU by running: `python make_liaison_fmu.py BouncingBall .\build\Debug\liaison.dll .\modelDescription.xml .\`
3. Generate the necessary files for the python server by running `python -m grpc_tools.protoc --proto_path=./ --python_out=. --pyi_out=. --grpc_python_out=. ./fmi3.proto`

## Using
1. Start the FMI server by running `python fmi3_server.py`
2. Run a simulation with FMPy `fmpy simulate BouncingBall.fmu`


