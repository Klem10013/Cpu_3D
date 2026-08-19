use std::ffi::c_double;
use crate::{Color, Normal};
use std::fs::File;
use std::io::{BufReader, Read};
use serde::Deserialize;
use crate::{Triangle3D,Camera};


#[derive(Deserialize)]
struct Config{
    sun : (f64,f64,f64),
    lights  : Vec<(isize,isize,isize)>,
    rectangles : Vec<Rectangle>,
    triangles : Vec<Triangle>,
    stl_files : Vec<String>,
    obj_files : Vec<String>,
}

#[derive(Deserialize)]
struct Rectangle{
    x : (f64,f64,f64),
    size : (f64,f64,f64),
    color : u32,
    only_face : bool,
    inverse_normal : bool
}
#[derive(Deserialize)]
struct Triangle{
    x : (f64,f64,f64),
    y : (f64,f64,f64),
    z : (f64,f64,f64),
    color : u32,
    only_face : bool,
    inverse_normal : bool
}

pub fn make_plane(pos : (f64,f64,f64), size : (f64,f64,f64), color : Color, only_face : bool, inverse_normal : bool) -> (Triangle3D,Triangle3D){
    let mut sz1 = (size.0,0f64,0f64);
    let mut sz2 = (0f64,size.1,0f64);
    if size.0 == 0f64{
        sz1 = (0f64,0f64,size.2);
    }else if size.1 == 0f64{
        sz2 = (0f64,0f64,size.2);
    }
    let face_1 = Triangle3D::new(pos,(pos.0+sz1.0,pos.1+sz1.1,pos.2+sz1.2),(pos.0+size.0,pos.1+size.1,pos.2 + size.2),color.clone(),only_face,!inverse_normal);
    let face_2 = Triangle3D::new(pos,(pos.0+sz2.0,pos.1+sz2.1,pos.2+sz2.2),(pos.0+size.0,pos.1+size.1,pos.2 + size.2),color,only_face,inverse_normal);
    (face_1,face_2)
}

fn stl_line_to_vec(line : &Vec<&str>) -> (f64,f64,f64){
    let x = str::parse::<f64>(line[2]).unwrap() * 100f64;
    let y = -str::parse::<f64>(line[3]).unwrap() * 100f64;
    let z = -str::parse::<f64>(line[1]).unwrap() * 100f64;
    (x,y,z)
}

fn obj_parser(config: &Config) -> Vec<Triangle3D>{
    let mut all_triangles = vec![];
    for obj_files in config.obj_files.iter(){
        let mut file = File::open(obj_files).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let lines = content.split("\n").collect::<Vec<&str>>();
        let mut index = 3;
        while index < lines.len(){
            
        }
    }
    all_triangles
}
fn stl_parser(config: &Config) -> Vec<Triangle3D>{
    let mut all_triangles = vec![];
    for stl_file in config.stl_files.iter(){
        let mut file = File::open(stl_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let lines = content.split("\n").collect::<Vec<&str>>();
        let mut index = 1;
        while index + 7 < lines.len(){
            let mut line_normal = lines[index].split_whitespace().collect::<Vec<&str>>();
            line_normal.remove(0);
            let nrm = stl_line_to_vec(&line_normal);
            let p1 = stl_line_to_vec(&lines[index+2].split_whitespace().collect::<Vec<&str>>());
            let p2 = stl_line_to_vec(&lines[index+3].split_whitespace().collect::<Vec<&str>>());
            let p3 = stl_line_to_vec(&lines[index+4].split_whitespace().collect::<Vec<&str>>());
            let normal = Normal::new_from_equation_f64(nrm,p1);
            all_triangles.push(Triangle3D::new_stl(p1,p2,p3,normal));

            index += 7;
        }
    }
    all_triangles
}

pub fn generate_config() -> (Camera,Vec<Triangle3D>){
    let mut file = File::open("config.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config : Config = serde_json::from_str(&content).unwrap();
    let color = Color::new_from_u32(0xFFFFFF);
    let mut triangles = vec![];
    for triangle in &config.triangles {
        let triangle_3d = Triangle3D::new(triangle.x,triangle.y,triangle.z,color.clone(),triangle.only_face,triangle.inverse_normal);
        println!("nx : {} ny : {} nz : {}",triangle_3d.normal.nx,triangle_3d.normal.ny,triangle_3d.normal.nz);
        triangles.push(triangle_3d);
    }
    for rec in &config.rectangles {
        //top face
        let (t1,t2) = make_plane(rec.x,(rec.size.0,0f64,rec.size.2),color.clone(),rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //front face
        let (t1,t2) = make_plane(rec.x,(rec.size.0,rec.size.1,0f64),color.clone(),rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //left face
        let (t1,t2) = make_plane(rec.x,(0f64,rec.size.1,rec.size.2),color.clone(),rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //right face
        let (t1,t2) = make_plane((rec.x.0 + rec.size.0,rec.x.1,rec.x.2),(0f64,rec.size.1,rec.size.2),color.clone(),rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //back face
        let (t1,t2) = make_plane((rec.x.0,rec.x.1,rec.x.2 + rec.size.2),(rec.size.0,rec.size.1,0f64),color.clone(),rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //bottom face
        let (t1,t2) = make_plane((rec.x.0,rec.x.1 + rec.size.1,rec.x.2),(rec.size.0,0f64,rec.size.2),color.clone(),rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);
    }
    triangles.append(&mut stl_parser(&config));
    (Camera::new(350,config.sun, config.lights), triangles)
}

pub fn read_png_to_col() -> Color{
    let path = "./textures/random.png";
    let img = image::open(path);
    let rgba = img.unwrap().to_rgba8();
    Color::new_from_img(rgba)
}
