mod parse_config;

use std::sync::atomic::{AtomicIsize, Ordering,};
use minifb::{Key, MouseMode, Window, WindowOptions};
use rayon::prelude::*;

use std::time::Instant;
use serde::{Deserialize, Serialize};
use parse_config::generate_triangle;

const WIDTH: usize = 960;
const WIDTH2I : isize = (WIDTH as isize) /2;
const HEIGHT: usize = 540;
const HEIGHT2I : isize = (HEIGHT as isize) /2;
const VIEW_DISTANCE : isize = 1000000;
const FPS : usize = 60;

#[derive(Debug)]
struct Camera{
    focal_dist : isize,
    focal_point: (isize, isize, isize),
    pos : (isize,isize,isize),
    rot : (f64,f64,f64),
    normal: Normal,
}

impl Camera{
    pub const fn new(focal_point_distance : isize) -> Camera{
        Camera{
            //focal_point: (0, 0, -focal_point_distance),
            focal_dist: focal_point_distance,
            focal_point: (0, 0, 0),
            pos : (0, 0, 0),
            rot : (0f64, 0f64, 0f64),
            normal: Normal::new_from_equation((0,0,1), (0,0,focal_point_distance)),
        }
    }

    pub fn get_focal_point(&self) -> (isize,isize,isize){
        self.focal_point
    }

    pub fn move_camera(&mut self, direction : (isize,isize,isize)){
        let x = ((direction.0 as f64)*(-self.rot.1).cos() - (direction.2 as f64)*(-self.rot.1).sin()) as isize;
        let z = ((direction.0 as f64)*(-self.rot.1).sin() + (direction.2 as f64)*(-self.rot.1).cos()) as isize;
        self.pos.0 +=  x;
        self.pos.1 += direction.1;
        self.pos.2 += z;
    }
}

#[derive(Clone,Debug)]
struct Normal{
    nx : isize,
    ny : isize,
    nz : isize,
    constant : isize,
}

impl Normal{
    pub const fn new(p1 : (isize,isize,isize), p2 : (isize,isize,isize), p3 : (isize,isize,isize)) -> Normal{
        let a: (isize, isize, isize) = (p2.0 - p1.0, p2.1 - p1.1, p2.2 - p1.2);
        let b: (isize, isize, isize) = (p3.0 - p1.0, p3.1 - p1.1, p3.2 - p1.2);
        let nx = a.1* b.2 - a.2* b.1;
        let ny = a.2* b.0 - a.0* b.2;
        let nz = a.0* b.1 - a.1* b.0;
        Normal {
            nx ,
            ny ,
            nz ,
            constant : -1 * (nx*p1.0 + ny*p1.1 + nz*p1.2),
        }
    }

    pub const fn new_from_equation(eq : (isize,isize,isize), pt : (isize,isize,isize)) -> Normal{
        Normal{
            nx: eq.0,
            ny: eq.1,
            nz: eq.2,
            constant: -1 * (eq.0*pt.0 + eq.1*pt.1 + eq.2*pt.2),
        }
    }

    pub fn to_str(&self) -> String {
        format!("NX : {} | NY : {} | NZ : {}", self.nx, self.ny, self.nz).to_string()
    }

    pub const fn dot_product(&self, vec : (isize,isize,isize)) -> isize {
        self.nx*vec.0 + self.ny*vec.1 + self.nz*vec.2
    }
}

struct Triangle3D{
    p1 : (isize,isize,isize),
    p2 : (isize,isize,isize),
    p3 : (isize,isize,isize),
    color : u32,
    normal : Normal,
}

impl Triangle3D{
    pub fn new(p1 : (isize,isize,isize), p2 : (isize,isize,isize), p3 : (isize,isize,isize), color : u32) -> Triangle3D{
        Triangle3D{p1 ,p2 ,p3 ,color , normal : Normal::new(p1,p2,p3)}
    }

    pub fn to_2d(&self, camera: &Camera) ->  Triangle2D{
        let (p1,p1_3d) = point_to_screen(self.p1, camera);
        let (p2,p2_3d) = point_to_screen(self.p2, camera);
        let (p3,p3_3d) = point_to_screen(self.p3, camera);

        Triangle2D::new(p1,p2,p3,Normal::new(p1_3d,p2_3d,p3_3d),self.color)
    }
}

fn point_to_screen(point : (isize,isize,isize), camera: &Camera) -> ((isize,isize),(isize,isize,isize)){
    let pt = camera.get_focal_point();
    let mut point_c = (point.0-camera.pos.0,point.1-camera.pos.1,point.2-camera.pos.2);
    let tmp = ((point_c.0 as f64)*camera.rot.1.cos() - (point_c.2 as f64)*camera.rot.1.sin()) as isize;
    point_c.2     = ((point_c.0 as f64)*camera.rot.1.sin() + (point_c.2 as f64)*camera.rot.1.cos()) as isize;
    point_c.0     = tmp;
    let vec = (point_c.0 - pt.0, point_c.1 - pt.1,point_c.2 - pt.2);
    if is_secant(&camera.normal,vec){
        let (mut x,y) = intersection_xy(&camera.normal,vec,point_c);
        if point_c.2 < 0{
            x = -1 * x;
        }
        ((x,y),point_c)

    }else{
        ((0,0),(0,0,0))
    }
}



#[derive(Debug)]
struct Triangle2D{
    p1i : (isize,isize), // position
    p2i : (isize,isize), // position
    p3i : (isize,isize), // position
    square_x : isize,
    square_y : isize,
    square_width : isize,
    square_height : isize,
    normal: Normal,
    color : u32
}

impl Triangle2D{
    pub const fn new(p1 : (isize,isize), p2 : (isize,isize), p3 : (isize,isize), normal: Normal, color : u32) -> Triangle2D{
        Triangle2D{
            p1i: p1,
            p2i: p2,
            p3i: p3,
            square_x: min3(p1.0, p2.0, p3.0),
            square_y: min3(p1.1, p2.1, p3.1),
            square_width: max3(p1.0, p2.0, p3.0),
            square_height: max3(p1.1, p2.1, p3.1),
            normal,
            color,
        }
    }
}

#[inline(always)]
fn sign(p1 : (isize,isize), p2 : (isize,isize), p3 : (isize,isize)) -> isize{
    (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
}

#[inline(always)]
const fn min3(v1 : isize, v2 : isize, v3 : isize) -> isize{
    if v1 < v2{
        if v1 < v3 {v1} else {v3}
    }else{
        if v3 < v2 {v3} else {v2}
    }
}

#[inline(always)]
const fn max3(v1 : isize, v2 : isize, v3 : isize) -> isize{
    if v1 > v2 {
        if v1 > v3 {v1} else {v3}
    }else{
        if v3 > v2 {v3} else {v2}
    }
}

#[inline(always)]
const fn is_secant(normal: &Normal, droite : (isize,isize,isize)) -> bool{
    normal.nx*droite.0 + normal.ny*droite.1 + normal.nz*droite.2 != 0
}

#[inline(always)]
const fn intersection_t(normal : &Normal, droite: (isize,isize,isize), point : (isize,isize,isize)) -> f64{
    ((-1*(normal.nx*point.0 + normal.ny*point.1 + normal.nz*point.2  + normal.constant)) as f64)/
        ((normal.nx*droite.0 + normal.ny*droite.1 + normal.nz*droite.2) as f64)
}
#[inline(always)]
const fn intersection_z(normal: &Normal, droite : (isize,isize,isize), point : (isize,isize,isize), ) -> isize{
    let z = (point.2 as f64) + (droite.2 as f64)*intersection_t(normal, droite, point);
    z as isize
}

#[inline(always)]
const fn intersection_xy(normal : &Normal, droite : (isize,isize,isize), point : (isize,isize,isize)) -> (isize,isize){
    let t = intersection_t(normal, droite, point);
    let x = (point.0 as f64) + (droite.0 as f64)*t;
    let y = (point.1 as f64) + (droite.1 as f64)*t;
    (x as isize, y as isize)
}

#[inline(always)]
const fn encode_isize(z : isize, color : u32) -> isize{
    (color as isize) | z << 32
}

#[inline(always)]
const fn decode(code : isize) -> (isize,u32) {
    let z = code >> 32;
    let color : u32 = code as u32;
    (z as isize,color)
}

fn compute_triangle(triangles: &[Triangle3D], camera: &Camera, atomic_buffer: &mut Vec<AtomicIsize>) {
    atomic_buffer.par_iter().for_each(|p| {p.store(encode_isize(VIEW_DISTANCE,0), Ordering::Relaxed)});

    triangles.par_iter().for_each(|tri_3d| {
        let tri = tri_3d.to_2d(camera);
        let focal_point = camera.get_focal_point();
        let min_x = if tri.square_x > -WIDTH2I {tri.square_x} else {-WIDTH2I};
        let min_y = if tri.square_y > -HEIGHT2I {tri.square_y} else {-HEIGHT2I};
        let max_x = if tri.square_width < WIDTH2I {tri.square_width} else {WIDTH2I};
        let max_y = if tri.square_height < HEIGHT2I {tri.square_height } else {HEIGHT2I};

        for y  in min_y..max_y {
            for x in min_x..max_x {
                let d1 = sign((x, y), tri.p1i, tri.p2i) as usize;
                let d2 = sign((x, y), tri.p2i, tri.p3i) as usize;
                let d3 = sign((x, y), tri.p3i, tri.p1i) as usize;
                if (d1 ^ d2) & 0x8000000000000000 == 0 && (d1 ^ d3) & 0x8000000000000000 == 0 {
                    let idx = (y+HEIGHT2I) as usize * WIDTH + (x+WIDTH2I) as usize;
                    let vec = (x,y,camera.focal_dist);
                    if is_secant(&tri.normal, vec){
                        let z = intersection_z(&tri.normal,vec,(x,y,camera.focal_dist));
                        if z > camera.focal_point.2 {
                            atomic_buffer[idx].fetch_min(encode_isize(z, tri.color), Ordering::Relaxed);
                        }
                    }
                }
            }
        }
    });
}

#[allow(dead_code)]
fn simple_rng(min : usize,max : usize) -> isize{
    static mut SEED: usize = 100;
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (SEED % (max-min) + min)  as isize
    }
}


const TRIANGLE_NB : usize = 150;

const FPS_BUFFER_SIZE: usize = 500;
const FPS_BUFFER_SIZE_128: u128= FPS_BUFFER_SIZE as u128;
fn main() {
    let mut camera : Camera = Camera::new(350);
    let mut buffer : Vec<u32> = vec![0; TRIANGLE_NB];
    let mut atomic_buffer : Vec<AtomicIsize> = (0..(WIDTH*HEIGHT)).map(|_| AtomicIsize::new(encode_isize(VIEW_DISTANCE,0))).collect();
    let mut fps_100 = vec![0; FPS_BUFFER_SIZE];
    let mut index = 0;
    let triangles = generate_triangle();

    //Windo management
    let mut window = Window::new("Mon moteur 3D", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap();
    window.set_target_fps(FPS);

    let mut fps = 0;
    let mut old_mouse_pos = (0f32,0f32);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(pos) = window.get_mouse_pos(MouseMode::Discard) {
            if old_mouse_pos == (0f32,0f32) {
                old_mouse_pos = pos;
            }else {
                let diff_x = (pos.0 - old_mouse_pos.0) / 5000f32;
                camera.rot.1 += diff_x.to_degrees() as f64;
                old_mouse_pos = (pos.0, pos.1);
            }
        }else{
            old_mouse_pos = (0f32,0f32)
        }
        if window.is_key_down(Key::W){
            camera.rot.1 += 1f64.to_radians();
        }
        if window.is_key_down(Key::X){
            camera.rot.1 -= 1f64.to_radians();
        }
        if window.is_key_down(Key::D){
            camera.move_camera((10,0,0));
        }
        if window.is_key_down(Key::Q){
            camera.move_camera((-10,0,0));
        }
        if window.is_key_down(Key::Z){
            camera.move_camera((0,0,10));
        }
        if window.is_key_down(Key::S){
            camera.move_camera((0,0,-10));
        }
        if window.is_key_down(Key::A){
            camera.move_camera((0,10,0));
        }
        if window.is_key_down(Key::E){
            camera.move_camera((0,-10,0));
        }
        let start = Instant::now();
        compute_triangle(&triangles,&camera,&mut atomic_buffer);
        buffer = atomic_buffer.iter().map(|p| decode(p.load(Ordering::Relaxed)).1).collect::<Vec<u32>>();
        let d = start.elapsed().as_nanos();
        let d2 = 1_000_000_000/d;
        fps += d2 - fps_100[index];
        fps_100[index] = d2;
        index = (index + 1) %FPS_BUFFER_SIZE;
        //println!("\r FPS : mean {} instant {} index {} ",fps/FPS_BUFFER_SIZE_128,d2,index);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}