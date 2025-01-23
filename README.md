# Liaison (in development)

Liaison provides remote access to an FMU through a client FMU and a server application.

By doing so, Liasion enables co-simulations across organisations without sharing intellectual property, as the FMUs never leave the organisation. 

## Usage

Dowload the latest [release](https://github.com/RISE-Maritime/liaison/releases).

### Step 1: Create a "Liaison FMU"

``./liaison --make-fmu ../tests/BouncingBall.fmu fmus/bouncingball``

### Step 2: Serve the FMU 

``./liaison --serve ../tests/BouncingBall.fmu fmus/bouncingball``

### Step 3: Simulate the "Liaison FMU"

``fmpy simulate BouncingBallLiaison.fmu --show-plot``

## Debug

The flag `--debug` can be used in the creation of the Liasion FMU or the serving of the FMU so that the output of either is extra verbose.

``./liaison --make-fmu ../tests/BouncingBall.fmu fmus/bouncingball --debug``
``./liaison --serve ../tests/BouncingBall.fmu fmus/bouncingball --debug``

``

## Development

This repository contains the necessary files for developing Liaison smoothly in VSCode. To do so, you must have the VSCode extension "DevContainers". 

1. Clone the repository. 
2. Open it in VSCode.
3. Reopen the repository in a DevContainer. 
4. Build the targets.

