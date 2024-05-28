# PortalFMI (in development)

PortalFMI enables co-simulations across organisations without sharing intellectual property by levaraging the [FMI standard](https://www.fmi-standard.org) and the communication protocol [Zenoh](https://zenoh.io). 


## Installation

Download ...

## Usage

### Step 1: Create a `PortalFMU`

`python3 make_portal_fmu.py "BouncingBall" "./build/libliaison.so" "./modelDescription.xml" "./tests"`


### Test

In a terminal, `cd` to the `build` folder and start the liaison server:

``./portal-server ../tests/BouncingBall.fmu``

In a terminal, `cd` to the `tests` folder and create the Liaison FMU with the command:

``python3 ../make_portal_fmu.py "BouncingBall" "../build/libliaison.so" "./modelDescription.xml" "./" ``

Then, in the same terminal, run the Liaison FMU withthe command:
``fmpy simulate BouncingBallPortal.fmu --show-plot``

### Step 2: Run the `PortalServer` fo.
1. Creating a PortalFMU.
2. Running the PortalServer. 

## Development

This repository contains the necessary files for developing Liaision smoothly in VSCode. To do so, you must have the VSCode extension "DevContainers". 

## Old 

1. Build the dynamic libraries with CMake.
2. Make the Liaision FMU by running: `python make_liaison_fmu.py BouncingBall .\build\Debug\liaison.dll .\modelDescription.xml .\`
3. Generate the necessary files for the python server by running `python -m grpc_tools.protoc --proto_path=./ --python_out=. --pyi_out=. --grpc_python_out=. ./fmi3.proto`

## Using
1. Start the FMI server by running `python fmi3_server.py`
2. Run a simulation with FMPy `fmpy simulate BouncingBall.fmu`


