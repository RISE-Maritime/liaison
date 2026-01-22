"""Pytest fixtures for Liaison end-to-end tests."""

import shutil
import subprocess
import sys
import time
from pathlib import Path

import pytest

# Paths relative to project root
PROJECT_ROOT = Path(__file__).parent.parent
BUILD_DIR = PROJECT_ROOT / "build"
LIAISON_BIN = BUILD_DIR / "liaison"
EXAMPLES_DIR = PROJECT_ROOT / "examples"


@pytest.fixture(scope="session")
def liaison_binary():
    """Ensure liaison binary exists."""
    if not LIAISON_BIN.exists():
        pytest.skip(f"Liaison binary not found at {LIAISON_BIN}. Run 'cmake --build build' first.")
    return LIAISON_BIN


@pytest.fixture(scope="session")
def binaries_dir():
    """Ensure binaries directory with shared libraries exists."""
    binaries = BUILD_DIR / "binaries"
    if not binaries.exists():
        pytest.skip(f"Binaries directory not found at {binaries}. Run 'cmake --build build' first.")
    return binaries


@pytest.fixture
def working_dir(tmp_path, binaries_dir):
    """
    Create a working directory with required binaries.

    liaison --make-fmu expects ./binaries/ to contain the shared libraries,
    so we copy them to a temp directory for isolation.
    """
    # Copy binaries folder (liaison --make-fmu looks for ./binaries/)
    binaries_dst = tmp_path / "binaries"
    shutil.copytree(binaries_dir, binaries_dst)

    return tmp_path


@pytest.fixture
def bouncing_ball_fmu():
    """Path to the original BouncingBall.fmu."""
    fmu_path = EXAMPLES_DIR / "BouncingBall.fmu"
    if not fmu_path.exists():
        pytest.skip(f"BouncingBall.fmu not found at {fmu_path}")
    return fmu_path


@pytest.fixture
def liaison_fmu(working_dir, liaison_binary, bouncing_ball_fmu):
    """
    Create a LiaisonFMU using liaison --make-fmu.

    Returns the path to the created LiaisonFMU.
    """
    responder_id = "test/bouncingball"

    # Copy original FMU to working directory
    fmu_copy = working_dir / "BouncingBall.fmu"
    shutil.copy(bouncing_ball_fmu, fmu_copy)

    # Create LiaisonFMU
    result = subprocess.run(
        [str(liaison_binary), "--make-fmu", str(fmu_copy), responder_id],
        cwd=working_dir,
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode != 0:
        pytest.fail(f"liaison --make-fmu failed:\nstdout: {result.stdout}\nstderr: {result.stderr}")

    liaison_fmu_path = working_dir / "BouncingBallLiaison.fmu"
    if not liaison_fmu_path.exists():
        pytest.fail(f"LiaisonFMU was not created. Output:\n{result.stdout}\n{result.stderr}")

    return liaison_fmu_path


@pytest.fixture
def liaison_server(working_dir, liaison_binary, bouncing_ball_fmu):
    """
    Start liaison server as a background process.

    The server is automatically stopped after the test.
    """
    responder_id = "test/bouncingball"

    # Copy original FMU to working directory
    fmu_copy = working_dir / "BouncingBall.fmu"
    if not fmu_copy.exists():
        shutil.copy(bouncing_ball_fmu, fmu_copy)

    # Start server process
    server = subprocess.Popen(
        [str(liaison_binary), "--serve", str(fmu_copy), responder_id],
        cwd=working_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Wait for server to be ready (Zenoh peer discovery takes time)
    time.sleep(2.0)

    # Check server is still running
    if server.poll() is not None:
        stdout, stderr = server.communicate()
        pytest.fail(f"Server exited prematurely:\nstdout: {stdout.decode()}\nstderr: {stderr.decode()}")

    yield server

    # Cleanup: send 'q' to gracefully stop, then terminate if needed
    try:
        server.stdin.write(b'q\n')
        server.stdin.flush()
        server.wait(timeout=5)
    except Exception:
        server.terminate()
        try:
            server.wait(timeout=5)
        except Exception:
            server.kill()


# =============================================================================
# Python FMU fixtures
# =============================================================================


@pytest.fixture(scope="session")
def python_fmu_venv(tmp_path_factory):
    """
    Create a Python virtual environment with dependencies for Python FMUs.

    The BouncingBallPython.fmu requires numpy at runtime. The pythonfmu3
    runtime is bundled inside the FMU itself.
    """
    venv_dir = tmp_path_factory.mktemp("pythonfmu_venv")

    # Create venv
    result = subprocess.run(
        [sys.executable, "-m", "venv", str(venv_dir)],
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        pytest.skip(f"Failed to create venv: {result.stderr}")

    # Get pip path
    if sys.platform == "win32":
        pip = venv_dir / "Scripts" / "pip.exe"
    else:
        pip = venv_dir / "bin" / "pip"

    if not pip.exists():
        pytest.skip(f"pip not found in venv at {pip}")

    # Install required packages (numpy is needed by bouncingball.py)
    result = subprocess.run(
        [str(pip), "install", "numpy"],
        capture_output=True,
        text=True,
        timeout=120
    )
    if result.returncode != 0:
        pytest.skip(f"Failed to install numpy in venv: {result.stderr}")

    return venv_dir


@pytest.fixture
def bouncing_ball_python_fmu():
    """Path to the original BouncingBallPython.fmu (Python-based FMU)."""
    fmu_path = EXAMPLES_DIR / "BouncingBallPython.fmu"
    if not fmu_path.exists():
        pytest.skip(f"BouncingBallPython.fmu not found at {fmu_path}")
    return fmu_path


@pytest.fixture
def python_liaison_fmu(working_dir, liaison_binary, bouncing_ball_python_fmu):
    """
    Create a LiaisonFMU from the Python-based BouncingBallPython.fmu.

    Returns the path to the created LiaisonFMU.
    """
    responder_id = "test/bouncingballpython"

    # Copy original FMU to working directory
    fmu_copy = working_dir / "BouncingBallPython.fmu"
    shutil.copy(bouncing_ball_python_fmu, fmu_copy)

    # Create LiaisonFMU
    result = subprocess.run(
        [str(liaison_binary), "--make-fmu", str(fmu_copy), responder_id],
        cwd=working_dir,
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode != 0:
        pytest.fail(
            f"liaison --make-fmu failed for Python FMU:\n"
            f"stdout: {result.stdout}\nstderr: {result.stderr}"
        )

    liaison_fmu_path = working_dir / "BouncingBallPythonLiaison.fmu"
    if not liaison_fmu_path.exists():
        pytest.fail(
            f"Python LiaisonFMU was not created.\n"
            f"Output: {result.stdout}\n{result.stderr}"
        )

    return liaison_fmu_path


@pytest.fixture
def python_liaison_server(
    working_dir, liaison_binary, bouncing_ball_python_fmu, python_fmu_venv
):
    """
    Start liaison server for a Python-based FMU with --python-env.

    The server is automatically stopped after the test.
    """
    responder_id = "test/bouncingballpython"

    # Copy original FMU to working directory
    fmu_copy = working_dir / "BouncingBallPython.fmu"
    if not fmu_copy.exists():
        shutil.copy(bouncing_ball_python_fmu, fmu_copy)

    # Start server process with --python-env
    server = subprocess.Popen(
        [
            str(liaison_binary), "--serve", str(fmu_copy), responder_id,
            "--python-env", str(python_fmu_venv)
        ],
        cwd=working_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Wait for server to be ready (Python loading + Zenoh discovery)
    time.sleep(3.0)

    # Check server is still running
    if server.poll() is not None:
        stdout, stderr = server.communicate()
        pytest.fail(
            f"Python FMU server exited prematurely:\n"
            f"stdout: {stdout.decode()}\nstderr: {stderr.decode()}"
        )

    yield server

    # Cleanup: send 'q' to gracefully stop, then terminate if needed
    try:
        server.stdin.write(b'q\n')
        server.stdin.flush()
        server.wait(timeout=5)
    except Exception:
        server.terminate()
        try:
            server.wait(timeout=5)
        except Exception:
            server.kill()
