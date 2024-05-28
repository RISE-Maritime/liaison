# Liaision (in development)

Liaison provides remote access to an FMU through a client FMU and a server application.

By doing so, Liasion enables co-simulations across organisations without sharing intellectual property, as the FMUs never leave the organisation. 

## Usage

### Step 1: Create a "Liaision FMU"

``liaison --make-fmu ./tests/BouncingBall.fmu fmus/bouncingball``

### Step 2: Run the "Liaison Server"

``liaison --server ./tests/BouncingBall.fmu fmus/bouncingball``


## Development

This repository contains the necessary files for developing Liaision smoothly in VSCode. To do so, you must have the VSCode extension "DevContainers". 

1. Clone the repository. 
2. Open it in VSCode.
3. Reopen the repository in a DevContainer. 
4. Build the targets.

