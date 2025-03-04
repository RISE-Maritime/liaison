# Liaison (in development)

Liaison provides remote access to an FMU through a client FMU and a server application.

By doing so, Liasion enables co-simulations across organisations without sharing intellectual property, as the FMUs never leave the organisation.

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
