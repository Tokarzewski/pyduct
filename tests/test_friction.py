def test_friction_coefficient():
    from pyduct.physics.friction import friction_coefficient, friction_coefficient2

    Re = 4000
    E = 0.5
    assert 0.99 < friction_coefficient(Re, E) / friction_coefficient2(Re, E) < 1.01
