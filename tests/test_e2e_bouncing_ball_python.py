"""End-to-end test for Liaison using BouncingBallPython.fmu (Python-based FMU)."""

import numpy as np
import pytest
from fmpy import simulate_fmu


class TestBouncingBallPythonE2E:
    """End-to-end tests for Python-based BouncingBallPython FMU via Liaison."""

    def test_python_liaison_fmu_creation(self, python_liaison_fmu):
        """Test that LiaisonFMU is created successfully from Python FMU."""
        assert python_liaison_fmu.exists()
        assert python_liaison_fmu.suffix == ".fmu"
        assert "Liaison" in python_liaison_fmu.name
        assert "Python" in python_liaison_fmu.name

    def test_python_liaison_fmu_structure(self, python_liaison_fmu):
        """Test that Python LiaisonFMU has the expected structure."""
        import zipfile

        with zipfile.ZipFile(python_liaison_fmu, 'r') as fmu:
            names = fmu.namelist()

            # Must have modelDescription.xml
            assert "modelDescription.xml" in names

            # Must have config.json in binaries
            assert "binaries/config.json" in names

            # Must have at least one platform binary
            has_linux = any("x86_64-linux" in n and n.endswith(".so") for n in names)
            has_windows = any("x86_64-windows" in n and n.endswith(".dll") for n in names)
            assert has_linux or has_windows, "No platform binaries found"

    def test_python_fmu_server_starts(self, python_liaison_server):
        """Test that the Python FMU server starts successfully."""
        # Fixture ensures server is running; just verify it's still alive
        assert python_liaison_server.poll() is None, "Server should be running"

    def test_simulation_via_liaison_python(
        self, python_liaison_fmu, python_liaison_server, bouncing_ball_python_fmu
    ):
        """
        Test that simulation through Python LiaisonFMU produces correct results.

        This test:
        1. Simulates the original Python FMU directly (baseline)
        2. Simulates the Python LiaisonFMU through the server
        3. Compares results to ensure they match
        """
        # Mark server fixture as used (needed for side effect)
        _ = python_liaison_server

        stop_time = 3.0
        step_size = 0.01
        output_vars = ['ball.h', 'ball.v']

        # Simulate original Python FMU directly (baseline)
        baseline_result = simulate_fmu(
            str(bouncing_ball_python_fmu),
            stop_time=stop_time,
            output_interval=step_size,
            output=output_vars
        )

        # Simulate Python LiaisonFMU (through server)
        liaison_result = simulate_fmu(
            str(python_liaison_fmu),
            stop_time=stop_time,
            output_interval=step_size,
            output=output_vars
        )

        # Verify we got results
        assert len(liaison_result) > 0, "No simulation results from Python LiaisonFMU"
        assert len(baseline_result) > 0, "No simulation results from baseline Python FMU"

        # Verify result structure
        assert 'time' in liaison_result.dtype.names
        assert 'ball.h' in liaison_result.dtype.names
        assert 'ball.v' in liaison_result.dtype.names

        # Compare results between baseline and liaison
        np.testing.assert_allclose(
            liaison_result['time'],
            baseline_result['time'],
            rtol=1e-5,
            err_msg="Time arrays differ between baseline and Python LiaisonFMU"
        )

        np.testing.assert_allclose(
            liaison_result['ball.h'],
            baseline_result['ball.h'],
            rtol=1e-5,
            err_msg="Height (ball.h) differs between baseline and Python LiaisonFMU"
        )

        np.testing.assert_allclose(
            liaison_result['ball.v'],
            baseline_result['ball.v'],
            rtol=1e-5,
            err_msg="Velocity (ball.v) differs between baseline and Python LiaisonFMU"
        )

    def test_python_bouncing_ball_physics(
        self, python_liaison_fmu, python_liaison_server
    ):
        """
        Test that the BouncingBall physics are correct via Python LiaisonFMU.

        The ball:
        - Starts at h=1.0m, v=0
        - Falls under gravity
        - Bounces with coefficient of restitution e=0.7
        - Eventually settles near h=0
        """
        # Mark server fixture as used (needed for side effect)
        _ = python_liaison_server

        result = simulate_fmu(
            str(python_liaison_fmu),
            stop_time=3.0,
            output_interval=0.01,
            output=['ball.h', 'ball.v']
        )

        # Initial conditions
        assert result['ball.h'][0] == pytest.approx(1.0, abs=0.01), \
            "Initial height should be 1.0m"
        assert result['ball.v'][0] == pytest.approx(0.0, abs=0.01), \
            "Initial velocity should be 0"

        # Ball should fall (height decreases initially)
        assert result['ball.h'][10] < result['ball.h'][0], "Ball should be falling"

        # Ball should have bounced and settled by end of simulation
        final_h = result['ball.h'][-1]
        assert final_h < 0.1, f"Ball should have settled near ground, but h={final_h}"

        # Height should never go significantly negative (ground constraint)
        assert np.min(result['ball.h']) >= -0.01, "Ball went below ground"


class TestPythonServerLifecycle:
    """Tests for Python FMU server lifecycle."""

    def test_multiple_simulations_python(
        self, python_liaison_fmu, python_liaison_server
    ):
        """Test that multiple simulations can run against the same Python server."""
        # Mark server fixture as used (needed for side effect)
        _ = python_liaison_server

        for i in range(3):
            result = simulate_fmu(
                str(python_liaison_fmu),
                stop_time=1.0,
                output_interval=0.1,
                output=['ball.h']
            )
            assert len(result) > 0, f"Simulation {i+1} failed"
            assert result['ball.h'][0] == pytest.approx(1.0, abs=0.01)
