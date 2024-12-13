use macroquad::prelude::*;
use sgp4::Elements;
use sgp4::Constants;
use std::error::Error;
use sgp4::chrono::{DateTime, Utc}; // Use sgp4's re-exported chrono
use std::time::SystemTime;

fn draw_equator(center: Vec3, radius: f32, segments: usize, color: Color) {
    let mut prev_point = center + vec3(radius, 0.0, 0.0); // Starting point on the circle

    for i in 1..=segments {
        let theta = i as f32 * std::f32::consts::TAU / segments as f32; // Angle in radians TAU=2pi
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        let current_point = center + vec3(x, z, 0.0); // Y is fixed at 0 for the equator

        // Draw a line segment
        draw_line_3d(prev_point, current_point, color);

        // Update previous point
        prev_point = current_point;
    }
}

#[macroquad::main("Satellite orbit simulation")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Constants
    let sphere_center = vec3(0.0, 0.0, 0.0); // Center of the sphere
    let earth_radius_km = 6378.0; // Earth's radius in km
    let satellite_altitude_km = 500.0; // Satellite's altitude in km
    let scale_factor = 1000.0; // Scale factor: 1 unit = 1000 km
    let earth_radius = earth_radius_km / scale_factor; // Scaled radius
    let satellite_orbit_radius = (earth_radius_km + satellite_altitude_km) / scale_factor;
    let equator_segments = 100; // Number of segments for the equator circle

    // Space Station Information
    let constants = sgp4::Constants::from_elements_afspc_compatibility_mode(
        &sgp4::Elements::from_tle(
            Some("ISS (ZARYA)".to_owned()),
            "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992".as_bytes(),
            "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008".as_bytes(),
        )?,
    )?;
    
    let camera_distance = 20.0; // Fixed distance from the sphere
    let mut azimuth_angle = 0.0; // Horizontal angle (radians)
    let mut elevation_angle : f32 = 0.0; // Vertical angle (radians)
    let rotation_speed = 0.01; // Rotation speed (radians per second)
    
    // Capitals ECI coordinates in km with coordinates scaled properly
    let lux_xyz_km = vec3(4092.183/scale_factor,439.465/scale_factor,4872.922/scale_factor);
    let ger_xyz_km = vec3(3775.280/scale_factor,899.747/scale_factor,5061.495/scale_factor);
    let pol_xyz_km = vec3(3646.877/scale_factor,1400.796/scale_factor,5041.822/scale_factor);
    let fra_xyz_km = vec3(4192.953/scale_factor,172.233/scale_factor,4803.176/scale_factor);
    let bra_xyz_km = vec3(4116.715/scale_factor,-4553.310/scale_factor,-1736.287/scale_factor);

    // Camera setup
    // let camera_distance = earth_radius * 3.0; // Camera distance for better view
    // interesting angles: Camera Position: x = 0.03, y = -18.79, z = 6.86  Initial
    //                     Camera Position: x = 19.13, y = -3.27, z = 19.13 North on the Top
    //                     Camera Position: x = 19.64, y = 0.21, z = 6.86
    let mut camera_position = vec3(15.94,0.0,14.0);
    
    let mut iterations = 0.0;
    loop {
        clear_background(BLACK);

        // Move the camera with arrow keys
        if is_key_down(KeyCode::Up) {
            camera_position.y += 0.1; // Move up
        }
        if is_key_down(KeyCode::Down) {
            camera_position.y -= 0.1; // Move down
        }
        if is_key_down(KeyCode::Left) {
            camera_position.x -= 0.1; // Move left
        }
        if is_key_down(KeyCode::Right) {
            camera_position.x += 0.1; // Move right
        }
        if is_key_down(KeyCode::Z) {
            camera_position.z -= 0.1; // Move z
        }
        if is_key_down(KeyCode::X) {
            camera_position.z += 0.1; // Move z
        }

        // Rotate camera with key presses
        if is_key_down(KeyCode::A) {
            camera_position = rotate_camera_horizontally(camera_position, sphere_center, rotation_speed);
        }
        if is_key_down(KeyCode::D) {
            camera_position = rotate_camera_horizontally(camera_position, sphere_center, -rotation_speed);
        }
        if is_key_down(KeyCode::W) {
            camera_position = rotate_camera_vertically(camera_position, sphere_center, rotation_speed);
        }
        if is_key_down(KeyCode::S) {
            camera_position = rotate_camera_vertically(camera_position, sphere_center, -rotation_speed);
        }       

        // Print camera position when Enter is pressed
        if is_key_pressed(KeyCode::Enter) {
            println!(
                "Camera Position: x = {:.2}, y = {:.2}, z = {:.2}",
                camera_position.x, camera_position.y, camera_position.z
            );
        }

        // Set 3D camera
        set_camera(&Camera3D {
            position: camera_position,
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            ..Default::default()
        });

        draw_axes(vec3(0.,0.,0.), earth_radius * 2.0); // Axes with length 2x the sphere radius
                                                       //
        // Draw Earth (Sphere)
        draw_sphere(vec3(0.0, 0.0, 0.0), earth_radius, None, BLUE);

        // Draw Equator
        draw_equator(vec3(0.,0.,0.), earth_radius, equator_segments, YELLOW);

        // Time for orbit calculation
        let time = get_time();

       // Draw Capitals (Cube)
        draw_cube(lux_xyz_km, vec3(0.2, 0.2, 0.2), None, SKYBLUE);
        draw_cube(ger_xyz_km, vec3(0.2, 0.2, 0.2), None, YELLOW);
        draw_cube(pol_xyz_km, vec3(0.2, 0.2, 0.2), None, WHITE);
        draw_cube(fra_xyz_km, vec3(0.2, 0.2, 0.2), None, RED);
        draw_cube(bra_xyz_km, vec3(0.2, 0.2, 0.2), None, GREEN);

        // Draw North and South Poles
        draw_cube(vec3(0.,0.,earth_radius), vec3(0.2, 0.2, 0.2), None, PURPLE);
        draw_cube(vec3(0.,0.,-earth_radius), vec3(0.2, 0.2, 0.2), None, BROWN);

        // Space Station Position Calculation per frame
        // Propagate to current simulation time
        // 60 fps in 1 sec 
        // every 60 fps 1 minute has pass on Orbit Propagator
        let prediction = constants.propagate(sgp4::MinutesSinceEpoch(iterations)).unwrap();
        iterations += 0.1;
        let position = prediction.position;
        let _velocity = prediction.velocity;

        // Scale ECI position for the game world
        let satellite_position = vec3(
            position[0] as f32 / 1000.0, // Convert km to game units
            position[2] as f32 / 1000.0, // Swap Z and Y for game orientation
            position[1] as f32 / 1000.0,
        );

        // Draw the Space Station (Red Cube)
        draw_cube(satellite_position, vec3(0.2, 0.2, 0.2), None, RED);
       
        // Satellite (Square) position (orbiting)
        // let satellite_x = satellite_orbit_radius * (time.cos() as f32);
        // let satellite_z = 0.0; // Keep it on the equatorial plane
        // let satellite_y = satellite_orbit_radius * (time.sin() as f32); 
        // Draw satellite (Cube)
        // draw_cube(vec3(satellite_x, satellite_y, satellite_z), vec3(0.2, 0.2, 0.2), None, RED);

        // Reset camera to 2D for HUD
        set_default_camera();
        draw_text("Space Station orbit simulation", 20.0, 20.0, 30.0, WHITE);
        draw_text(format!("FPS: {}", get_fps()).as_str(), 20., 60., 30., WHITE);

        // Render next frame
        next_frame().await;
    }
}

fn spherical_to_cartesian(radius: f32, azimuth: f32, elevation: f32) -> Vec3 {
    Vec3::new(
        radius * elevation.cos() * azimuth.cos(),
        radius * elevation.sin(),
        radius * elevation.cos() * azimuth.sin(),
    )
}

fn rotate_camera_horizontally(camera: Vec3, center: Vec3, angle: f32) -> Vec3 {
    let radius = (camera - center).length(); // Distance from the camera to the sphere's center
    let current_angle = (camera.z - center.z).atan2(camera.x - center.x); // Current angle in the XZ-plane
    let new_angle = current_angle + angle; // Apply the rotation angle

    Vec3::new(
        center.x + radius * new_angle.cos(),
        camera.y, // Keep the same height (Y-coordinate)
        center.z + radius * new_angle.sin(),
    )
}

fn rotate_camera_vertically(camera: Vec3, center: Vec3, angle: f32) -> Vec3 {
    let radius = (camera - center).length(); // Distance from the camera to the sphere's center
    let elevation_angle = ((camera.y - center.y) / radius).asin(); // Current elevation angle
    let new_elevation_angle = (elevation_angle + angle).clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1); // Clamp to avoid flipping

    Vec3::new(
        center.x + radius * new_elevation_angle.cos() * (camera.x - center.x).atan2(camera.z - center.z).cos(),
        center.y + radius * new_elevation_angle.sin(),
        center.z + radius * new_elevation_angle.cos() * (camera.x - center.x).atan2(camera.z - center.z).sin(),
    )
}

 // Function to draw axes in 3D
fn draw_axes(origin: Vec3, length: f32) {
    // X-Axis (Red)
    draw_line_3d(origin, origin + vec3(length, 0.0, 0.0), RED);
    // Y-Axis (Green)
    draw_line_3d(origin, origin + vec3(0.0, length, 0.0), GREEN);
    // Z-Axis (Blue)
    draw_line_3d(origin, origin + vec3(0.0, 0.0, length), BLUE);
} 
