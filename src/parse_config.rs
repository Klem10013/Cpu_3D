use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use crate::Triangle3D;

#[derive(Deserialize)]
struct Config{
    sun : (isize,isize,isize),
    rectangles : Vec<Rectangle>,
    triangles : Vec<Triangle>,
}

#[derive(Deserialize)]
struct Rectangle{
    x : (isize,isize,isize),
    size : (isize,isize,isize),
    color : u32
}
#[derive(Deserialize)]
struct Triangle{
    x : (isize,isize,isize),
    y : (isize,isize,isize),
    z : (isize,isize,isize),
    color : u32
}
pub fn generate_triangle() -> Vec<Triangle3D>{
    let mut file = File::open("config.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config : Config = serde_json::from_str(&content).unwrap();

    let mut triangles = vec![];
    for triangle in config.triangles {
        let triangle_3d = Triangle3D::new(triangle.x,triangle.y,triangle.z,triangle.color);
        triangles.push(triangle_3d);
    }
    triangles
}

