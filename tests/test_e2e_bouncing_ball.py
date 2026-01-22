"""End-to-end test for Liaison using BouncingBall FMU."""

import numpy as np
import pytest
from fmpy import simulate_fmu


class TestBouncingBallE2E:
    """End-to-end tests for BouncingBall FMU via Liaison."""

    def test_liaison_fmu_creation(self, liaison_fmu):
        """Test that LiaisonFMU is created successfully."""
        assert liaison_fmu.exists()
        assert liaison_fmu.suffix == ".fmu"
        assert "Liaison" in liaison_fmu.name

    def test_liaison_fmu_structure(self, liaison_fmu):
        """Test that LiaisonFMU has the expected structure."""
        import zipfile

        with zipfile.ZipFile(liaison_fmu, 'r') as fmu:
            names = fmu.namelist()

            # Must have modelDescription.xml
            assert "modelDescription.xml" in names

            # Must have config.json in binaries
            assert "binaries/config.json" in names

            # Must have at least one platform binary
            has_linux = any("x86_64-linux" in n and n.endswith(".so") for n in names)
            has_windows = any("x86_64-windows" in n and n.endswith(".dll") for n in names)
            assert has_linux or has_windows, "No platform binaries found in LiaisonFMU"

    def test_simulation_via_liaison(self, liaison_fmu, liaison_server, bouncing_ball_fmu):
        """
        Test that simulation through LiaisonFMU produces correct results.

        This test:
        1. Simulates the original FMU directly (baseline)
        2. Simulates the LiaisonFMU through the server
        3. Compares results to ensure they match
        """
        # Mark server fixture as used (needed for side effect)
        _ = liaison_server

        stop_time = 3.0
        step_size = 0.01
        output_vars = ['h', 'v']

        # Simulate original FMU directly (baseline)
        baseline_result = simulate_fmu(
            str(bouncing_ball_fmu),
            stop_time=stop_time,
            output_interval=step_size,
            output=output_vars
        )

        # Simulate LiaisonFMU (through server)
        liaison_result = simulate_fmu(
            str(liaison_fmu),
            stop_time=stop_time,
            output_interval=step_size,
            output=output_vars
        )

        # Verify we got results
        assert len(liaison_result) > 0, "No simulation results from LiaisonFMU"
        assert len(baseline_result) > 0, "No simulation results from baseline FMU"

        # Verify result structure
        assert 'time' in liaison_result.dtype.names
        assert 'h' in liaison_result.dtype.names
        assert 'v' in liaison_result.dtype.names

        # Compare results between baseline and liaison
        np.testing.assert_allclose(
            liaison_result['time'],
            baseline_result['time'],
            rtol=1e-5,
            err_msg="Time arrays differ between baseline and LiaisonFMU"
        )

        np.testing.assert_allclose(
            liaison_result['h'],
            baseline_result['h'],
            rtol=1e-5,
            err_msg="Height (h) results differ between baseline and LiaisonFMU"
        )

        np.testing.assert_allclose(
            liaison_result['v'],
            baseline_result['v'],
            rtol=1e-5,
            err_msg="Velocity (v) results differ between baseline and LiaisonFMU"
        )

    def test_bouncing_ball_physics(self, liaison_fmu, liaison_server):
        """
        Test that the BouncingBall physics are correct via LiaisonFMU.

        The ball:
        - Starts at h=1.0m, v=0
        - Falls under gravity
        - Bounces with coefficient of restitution e=0.7
        - Eventually settles near h=0
        """
        # Mark server fixture as used (needed for side effect)
        _ = liaison_server

        result = simulate_fmu(
            str(liaison_fmu),
            stop_time=3.0,
            output_interval=0.01,
            output=['h', 'v']
        )

        # Initial conditions
        assert result['h'][0] == pytest.approx(1.0, abs=0.01), "Initial height should be 1.0m"
        assert result['v'][0] == pytest.approx(0.0, abs=0.01), "Initial velocity should be 0"

        # Ball should fall (height decreases initially)
        assert result['h'][10] < result['h'][0], "Ball should be falling"

        # Ball should have bounced and settled by end of simulation
        final_h = result['h'][-1]
        assert final_h < 0.1, f"Ball should have settled near ground, but h={final_h}"

        # Height should never go significantly negative (ground constraint)
        assert np.min(result['h']) >= -0.01, "Ball went below ground"


class TestServerLifecycle:
    """Tests for server lifecycle and error handling."""

    def test_server_starts_and_stops(self, liaison_server):
        """Test that the server starts and can be stopped gracefully."""
        # Server should be running
        assert liaison_server.poll() is None, "Server should be running"

    def test_multiple_simulations(self, liaison_fmu, liaison_server):
        """Test that multiple simulations can run against the same server."""
        # Mark server fixture as used (needed for side effect)
        _ = liaison_server

        for i in range(3):
            result = simulate_fmu(
                str(liaison_fmu),
                stop_time=1.0,
                output_interval=0.1,
                output=['h']
            )
            assert len(result) > 0, f"Simulation {i+1} failed"
            assert result['h'][0] == pytest.approx(1.0, abs=0.01)
