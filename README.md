# Liaison

> :warning: **Currently in development!**

Liaison is an open-source tool designed to simplify the sharing of Functional Mock-up Units (FMUs) both within and between organizations. It addresses two specific challenges:

1. Platform and environment compatibility — when an FMU cannot be easily packaged to run across different operating systems (e.g. Linux, Windows) or when the target environment lacks required dependencies (e.g., MATLAB runtime).

2. Intellectual property protection — when there is resistance to sharing models directly due to concerns about revealing proprietary information.

To address these challenges, Liaison uses a client-server architecture with a two-step workflow:

1. First, Liaison is used to generate a client FMU, called the LiaisonFMU, based on the original FMU. This LiaisonFMU is configured to communicate with a server via a Zenoh network at a predefined endpoint. It represents the original FMU but does not contain any of the original software or files besides the model description file.
2. Second, Liaison is used to serve the original FMU at the predefined endpoint on a [Zenoh network](https://zenoh.io/docs/overview/zenoh-in-action/) using a computer with the appropriate operating system and/or dependencies.

[Zenoh](https://zenoh.io) acts as the communication layer, enabling seamless data exchange within and across networks, depending on its configuration. This approach allows FMUs to be used remotely without needing to share the original files, preserving intellectual property while enabling interoperability.

Liaison is inspired on [FMU-proxy](https://github.com/NTNU-IHB/FMU-proxy). The main difference between both softwares is that Liaison relies on Zenoh as the communication layer. Thanks to Zenoh’s peer-to-peer capabilities, the client FMU does not need to know the exact location of the FMU server — routing is handled automatically by the network. Another difference is that Liaison currently supports FMI 3.0.

## Get Started

To get started with Liaison, follow these steps:

1. **Download and Install**

   - Download the latest [release](https://github.com/RISE-Maritime/liaison/releases).
   - Install [FMPy](https://fmpy.readthedocs.io/en/latest/install/) in your Python environment.  
     _Note: FMPy is not a required dependency for Liaison; it is only used for testing._

2. **Create a "Liaison FMU"**

Use the following command to create a Liaison FMU.

```bash
./liaison --make-fmu ../tests/BouncingBall.fmu fmus/bouncingball
```

The result will be an FMU named `BouncingBallLiaison.fmu`. Unlike the original FMU, this Liaison FMU does not include the original logic but retains the model description. It acts as a client, making all FMI function calls to the FMU being served at fmus/bouncingball.

3. **Step 2: Serve the original FMU**

Start serving the original FMU at `fmus/bouncingball` with the command:

```bash
./liaison --serve ../tests/BouncingBall.fmu fmus/bouncingball
```

4. **Simulate the "Liaison FMU"**

Simulate the Liaison FMU using FMPy and display the results:

```bash
fmpy simulate BouncingBallLiaison.fmu --show-plot
```

With this setup, FMPy will simulate the FMU as if it were the original, but the underlying logic is now served by the original FMU, which can be hosted on another computer within the same local network (LAN) or in another network using a Zenoh router (see "Zenoh configuration" below.)

## Usage

### Zenoh configuration

The flag `--zenoh-config` can be used to provide a [Zenoh configuration JSON file](https://zenoh.io/docs/manual/configuration/#configuration-files) so that the Liasion FMU and/or the Liasion server connects to a Zenoh router.

```bash
./liaison --make-fmu ../tests/BouncingBall.fmu fmus/bouncingball --zenoh-config ../config.json
```

```bash
./liaison --serve ../tests/BouncingBall.fmu fmus/bouncingball --zenoh-config ../tests/config.json
```

The following is an example of Zenoh configuration JSON file using TLS. The `*.pem` must be in the same directory as the JSON configuration file.

```json
{
  "mode": "client",
  "connect": {
    "endpoints": ["tls/zenoh.foobar.com:443"]
  },
  "listen": {
    "endpoints": ["tls/[::]:7447"]
  },
  "metadata": {
    "name": "BouncingBallLiaison.fmu"
  },
  "transport": {
    "link": {
      "tls": {
        "root_ca_certificate": "minica.pem",
        "enable_mtls": true,
        "connect_private_key": "key.pem",
        "connect_certificate": "cert.pem"
      }
    }
  }
}
```

### Debug

The flag `--debug` can be used so that the output is extra verbose to facilitate debbuging.

```bash
./liaison --serve ./tests/BouncingBall.fmu fmus/bouncingball --debug
```

### Python FMUs

If the FMU depends on a Python environment, the Python environment (e.g. Conda or virtualenv) or Python library can be declared by using either the `--python-env` or `--python-lib` flags.

**Python environment**

Conda

```bash
./liaison --serve ./BouncingBall.fmu fmus/bouncingball --python-env /home/user/miniconda3/envs/bouncingball
```

Virtualenv

```bash
./liaison --serve ./BouncingBall.fmu fmus/bouncingball --python-env /home/user/.virualenvs/bouncingball
```

**Python library**

```bash
./liaison --serve ./BouncingBall.fmu fmus/bouncingball --python-lib /usr/lib/x86_64-linux-gnu/libpython3.8.so
```

## Development

This repository contains the necessary files for developing Liaison smoothly in VSCode. To do so, you must have the VSCode extension "DevContainers".

1. Clone the repository.
2. Open it in VSCode.
3. Reopen the repository in a DevContainer.
4. Build the targets.

## Current functionality

FMI 3.0 functions stated below without any remarks are implemented.

**Common Functions**

    fmi3GetVersion

    fmi3SetDebugLogging

    fmi3InstantiateModelExchange

    fmi3InstantiateCoSimulation (partially implemented)

    fmi3InstantiateScheduledExecution (partially implemented)

    fmi3FreeInstance

    fmi3EnterInitializationMode

    fmi3ExitInitializationMode

    fmi3EnterEventMode

    fmi3Terminate

    fmi3Reset

**Get/Set Variable Values**

    fmi3GetFloat32, fmi3SetFloat32

    fmi3GetFloat64, fmi3SetFloat64

    fmi3GetInt8, fmi3SetInt8

    fmi3GetUInt8, fmi3SetUInt8

    fmi3GetInt16, fmi3SetInt16

    fmi3GetUInt16, fmi3SetUInt16

    fmi3GetInt32, fmi3SetInt32

    fmi3GetUInt32, fmi3SetUInt32

    fmi3GetInt64, fmi3SetInt64

    fmi3GetUInt64, fmi3SetUInt64

    fmi3GetBoolean, fmi3SetBoolean

    fmi3GetString

    fmi3SetString

    fmi3GetBinary

    fmi3SetBinary

    fmi3GetClock

    fmi3SetClock

**Variable Dependency and FMU State**

    fmi3GetNumberOfVariableDependencies (NOT implemented)

    fmi3GetVariableDependencies (NOT implemented)

    fmi3GetFMUState (NOT implemented)

    fmi3SetFMUState (NOT implemented)

    fmi3FreeFMUState (NOT implemented)

    fmi3SerializedFMUStateSize (NOT implemented)

    fmi3SerializeFMUState (NOT implemented)

    fmi3DeserializeFMUState (NOT implemented)

**PGetting Partial Derivatives**

    fmi3GetDirectionalDerivative (NOT implemented)

    fmi3GetAdjointDerivative (NOT implemented)

**Configuration Mode**

    fmi3EnterConfigurationMode (NOT implemented)

    fmi3ExitConfigurationMode (NOT implemented)

**Clock Interval / Shift**

    fmi3GetIntervalDecimal (NOT implemented)

    fmi3GetIntervalFraction (NOT implemented)

    fmi3GetShiftDecimal (NOT implemented)

    fmi3GetShiftFraction (NOT implemented)

    fmi3SetIntervalDecimal (NOT implemented)

    fmi3SetIntervalFraction (NOT implemented)

    fmi3SetShiftDecimal (NOT implemented)

    fmi3SetShiftFraction (NOT implemented)

**Discrete States**

    fmi3EvaluateDiscreteStates (NOT implemented)

    fmi3UpdateDiscreteStates (NOT implemented)

**Model Exchange**

    fmi3EnterContinuousTimeMode (NOT implemented)

    fmi3CompletedIntegratorStep (NOT implemented)

    fmi3SetTime (NOT implemented)

    fmi3SetContinuousStates (NOT implemented)

    fmi3GetContinuousStateDerivatives (NOT implemented)

    fmi3GetEventIndicators (NOT implemented)

    fmi3GetContinuousStates (NOT implemented)

    fmi3GetNominalsOfContinuousStates (NOT implemented)

    fmi3GetNumberOfEventIndicators (NOT implemented)

    fmi3GetNumberOfContinuousStates (NOT implemented)

**Co-Simulation**

    fmi3DoStep

    fmi3EnterStepMode (NOT implemented)

    fmi3GetOutputDerivatives (NOT implemented)

**Scheduled Execution**

    fmi3ActivateModelPartition (NOT implemented)
