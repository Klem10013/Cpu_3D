use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use crate::{Triangle3D,Camera};

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
    color : u32,
    only_face : bool,
    inverse_normal : bool
}
#[derive(Deserialize)]
struct Triangle{
    x : (isize,isize,isize),
    y : (isize,isize,isize),
    z : (isize,isize,isize),
    color : u32,
    only_face : bool,
    inverse_normal : bool
}

pub fn make_plane(pos : (isize,isize,isize), size : (isize,isize,isize), color : u32, only_face : bool, inverse_normal : bool) -> (Triangle3D,Triangle3D){
    let mut sz1 = (size.0,0,0);
    let mut sz2 = (0,size.1,0);
    if size.0 == 0{
        sz1 = (0,0,size.2);
    }else if size.1 == 0{
        sz2 = (0,0,size.2);
    }
    let face_1 = Triangle3D::new(pos,(pos.0+sz1.0,pos.1+sz1.1,pos.2+sz1.2),(pos.0+size.0,pos.1+size.1,pos.2 + size.2),color,only_face,!inverse_normal);
    let face_2 = Triangle3D::new(pos,(pos.0+sz2.0,pos.1+sz2.1,pos.2+sz2.2),(pos.0+size.0,pos.1+size.1,pos.2 + size.2),color,only_face,inverse_normal);
    (face_1,face_2)
}

pub fn generate_config() -> (Camera,Vec<Triangle3D>){
    let mut file = File::open("config.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config : Config = serde_json::from_str(&content).unwrap();

    let mut triangles = vec![];
    for triangle in &config.triangles {
        let triangle_3d = Triangle3D::new(triangle.x,triangle.y,triangle.z,triangle.color,triangle.only_face,triangle.inverse_normal);
        println!("nx : {} ny : {} nz : {}",triangle_3d.normal.nx,triangle_3d.normal.ny,triangle_3d.normal.nz);
        //triangles.push(triangle_3d);
    }
    for rec in &config.rectangles {
        //top face
        let (t1,t2) = make_plane(rec.x,(rec.size.0,0,rec.size.2),rec.color,rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //front face
        let (t1,t2) = make_plane(rec.x,(rec.size.0,rec.size.1,0),rec.color,rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //left face
        let (t1,t2) = make_plane(rec.x,(0,rec.size.1,rec.size.2),rec.color,rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //right face
        let (t1,t2) = make_plane((rec.x.0 + rec.size.0,rec.x.1,rec.x.2),(0,rec.size.1,rec.size.2),rec.color,rec.only_face,rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //back face
        let (t1,t2) = make_plane((rec.x.0,rec.x.1,rec.x.2 + rec.size.2),(rec.size.0,rec.size.1,0),rec.color,rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);

        //bottom face
        let (t1,t2) = make_plane((rec.x.0,rec.x.1 + rec.size.1,rec.x.2),(rec.size.0,0,rec.size.2),rec.color,rec.only_face,!rec.inverse_normal);
        triangles.push(t1);
        triangles.push(t2);
    }
    (Camera::new(350,config.sun), triangles)
}

