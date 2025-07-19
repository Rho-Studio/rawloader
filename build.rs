#[path = "data/cameras/join.rs"]
mod join;

use crate::join::join_toml_data;
// use crate::schema::Cameras;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let response = reqwest::blocking::get("https://raw.githubusercontent.com/darktable-org/rawspeed/refs/heads/develop/data/cameras.xml")?;
    // let cameras: Cameras = serde_xml_rs::from_str(response.text()?.as_str())?;
    //
    // for camera in cameras.cameras {
    //     println!("Make: {}, Model: {}", camera.make, camera.model);
    // }

    join_toml_data();

    Ok(())
}