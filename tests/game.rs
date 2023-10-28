use tictactoe_client::game::{ point_in_polygon, are_intersecting };

#[test]
fn point_in_polygon_test() {
    // Test that a point inside the polygon returns true.
    let point = (0.5, 0.5);
    let polygon = [
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    ];

    assert!(point_in_polygon(point.0, point.1, polygon));

    // Test that a point outside the polygon returns false.
    let point = (1.5, 0.5);
    let polygon = [
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    ];

    assert!(!point_in_polygon(point.0, point.1, polygon));

    // Test that a point on the edge of the polygon returns false.
    let point = (0.0, 0.5);
    let polygon = [
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    ];

    assert!(!point_in_polygon(point.0, point.1, polygon));

    let point = (0.25, 0.5);
    let polygon = [
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    ];

    assert!(point_in_polygon(point.0, point.1, polygon));

    for i in 1..1000 {
        for j in 1..1000 {
            let point = ((i as f32) / 1000.0, (j as f32) / 1000.0);
            let polygon = [
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
            ];

            assert!(point_in_polygon(point.0, point.1, polygon));
        }
    }

    let polygon = [
        (-0.50732255, 0.60502225),
        (-0.60631233, 0.60502225),
        (-0.60631233, 0.5062431),
        (-0.50732255, 0.5062431),
    ];
    let point = (-0.58208954, 0.5765958);

    assert!(point_in_polygon(point.0, point.1, polygon));
}

#[test]
fn are_intersecting_test() {
    assert!(are_intersecting(0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0));
    assert!(!are_intersecting(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0));
}
