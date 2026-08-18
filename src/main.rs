mod parse_config;

use std::sync::atomic::{AtomicIsize, Ordering,};
use minifb::{Key, MouseMode, Window, WindowOptions};
use rayon::prelude::*;

use std::time::Instant;
use crate::parse_config::generate_config;

const WIDTH: usize = 1000;
const WIDTH2I : isize = (WIDTH as isize) /2;
const HEIGHT: usize = 500;
const HEIGHT2I : isize = (HEIGHT as isize) /2;
const VIEW_DISTANCE : isize = 1000000;

#[derive(Debug)]
struct Camera{
    focal_dist : isize,
    focal_distf : f64,
    pos : (isize,isize,isize),
    posf : (f64,f64,f64),
    rot : (f64,f64,f64),
    normal: Normal,
    sun : (f64,f64,f64),
    lights : Vec<(isize,isize,isize)>,
}

impl Camera{
    pub const fn new(focal_point_distance : isize,sun : (f64,f64,f64), lights : Vec<(isize,isize,isize)>) -> Camera{
        Camera{
            //focal_point: (0, 0, -focal_point_distance),
            focal_dist: focal_point_distance,
            focal_distf: focal_point_distance as f64,
            pos : (0, 0, 0),
            posf: (0.0, 0.0, 0.0),
            rot : (0f64, 0f64, 0f64),
            normal: Normal::new_from_equation((0,0,1), (0,0,focal_point_distance)),
            sun,
            lights,
        }
    }

    pub fn move_camera(&mut self, direction : (isize,isize,isize)){
        let x = ((direction.0 as f64)*(-self.rot.1).cos() - (direction.2 as f64)*(-self.rot.1).sin()) as isize;
        let z = ((direction.0 as f64)*(-self.rot.1).sin() + (direction.2 as f64)*(-self.rot.1).cos()) as isize;
        self.pos.0 +=  x;
        self.pos.1 += direction.1;
        self.pos.2 += z;
        self.posf.0 = self.pos.0 as f64;
        self.posf.1 = self.pos.1 as f64;
        self.posf.2 = self.pos.2 as f64;
    }
}

#[derive(Clone,Debug)]
struct Normal{
    nx : isize,
    ny : isize,
    nz : isize,
    nxf : f64,
    nyf : f64,
    nzf : f64,
    constant : isize,
    constantf : f64
}

impl Normal{
    pub const fn new(p1 : (f64,f64,f64), p2 : (f64,f64,f64), p3 : (f64,f64,f64)) -> Normal{
        let a = (p2.0 - p1.0, p2.1 - p1.1, p2.2 - p1.2);
        let b = (p3.0 - p1.0, p3.1 - p1.1, p3.2 - p1.2);
        let nx = a.1* b.2 - a.2* b.1;
        let ny = a.2* b.0 - a.0* b.2;
        let nz = a.0* b.1 - a.1* b.0;
        let constant = -1f64 * (nx*p1.0 + ny*p1.1 + nz*p1.2);
        Normal {
            nxf : nx,
            nx  : nx as isize ,
            nyf : ny,
            ny  : ny as isize,
            nzf : nz,
            nz  : nz as isize,
            constantf: constant,
            constant: constant as isize,
        }
    }

    pub const fn new_from_equation_f64(eq : (f64,f64,f64), pt : (f64,f64,f64)) -> Normal{
        Normal{
            nx: eq.0 as isize,
            ny: eq.1 as isize,
            nz: eq.2 as isize,
            nxf: eq.0,
            nyf: eq.1,
            nzf: eq.2,
            constant: -1 * (eq.0*pt.0 + eq.1*pt.1 + eq.2*pt.2) as isize,
            constantf: -1f64 * (eq.0*pt.0 + eq.1*pt.1 + eq.2*pt.2),
        }
    }

    pub const fn new_from_equation(eq : (isize,isize,isize), pt : (isize,isize,isize)) -> Normal{
        Normal{
            nx: eq.0,
            ny: eq.1,
            nz: eq.2,
            nxf: eq.0 as f64,
            nyf: eq.1 as f64,
            nzf: eq.2 as f64,
            constant: -1 * (eq.0*pt.0 + eq.1*pt.1 + eq.2*pt.2),
            constantf: (-1 * (eq.0*pt.0 + eq.1*pt.1 + eq.2*pt.2)) as f64,
        }
    }

    #[inline(always)]
    pub const fn dot_product(&self, vec : (isize,isize,isize)) -> isize {
        self.nx*vec.0 + self.ny*vec.1 + self.nz*vec.2
    }

    pub const fn dot_product_f64(&self, vec:(f64,f64,f64)) -> f64{
        vec.0 * self.nxf +vec.1 * self.nyf +vec.2 * self.nzf
    }

    #[inline(always)]
    pub const fn inverse(&mut self){
        self.nx *= -1;
        self.ny *= -1;
        self.nz *= -1;
        self.nxf *= -1.0;
        self.nyf *= -1.0;
        self.nzf *= -1.0;
        self.constant *= -1;
        self.constantf *= -1f64;
    }

    #[inline(always)]
    pub const fn to_vec_f64(&self) -> (f64,f64,f64){
        (self.nxf,self.nyf,self.nzf)
    }
}

struct Triangle3D{
    p1 : (f64,f64,f64),
    p2 : (f64,f64,f64),
    p3 : (f64,f64,f64),
    color : u32,
    inverse_normal : bool,
    normal : Normal,
    only_face : bool,
}

impl Triangle3D{
    pub fn new(p1 : (f64,f64,f64), p2 : (f64,f64,f64), p3 : (f64,f64,f64), color : u32, only_face : bool,inverse_normal : bool) -> Triangle3D{
        let mut normal = Normal::new(p1,p2,p3);
        if inverse_normal{
            normal.inverse();
        }
        Triangle3D{p1 ,p2 ,p3 ,color , inverse_normal, normal, only_face }
    }

    pub fn new_stl(p1:(f64,f64,f64), p2: (f64,f64,f64), p3 : (f64,f64,f64),normal: Normal)-> Triangle3D{
        Triangle3D{
            p1,
            p2,
            p3,
            color: 0xFFFFFF,
            inverse_normal: false,
            normal,
            only_face: false,
        }
    }

    pub fn to_2d(&self, camera: &Camera) ->  Vec<Triangle2D>{
        let p1_3d = point_transformation(self.p1, camera);
        let p2_3d = point_transformation(self.p2, camera);
        let p3_3d = point_transformation(self.p3, camera);

        if p1_3d.2 < camera.focal_distf && p2_3d.2 < camera.focal_distf && p3_3d.2 < camera.focal_distf {
            vec![]
        }else {
            let mut triangles = vec![];
            let mut normal = Normal::new(p1_3d,p2_3d,p3_3d);
            if self.inverse_normal{
                normal.inverse()
            }
            let p1 : (f64,f64);
            let p2 : (f64,f64);
            let p3 : (f64,f64);
            if p1_3d.2 < camera.focal_distf {
                if p2_3d.2 < camera.focal_distf {
                    p1 = plane_clipping(p1_3d,p3_3d,&camera.normal);
                    p2 = plane_clipping(p2_3d,p3_3d,&camera.normal);
                    p3 = projection(p3_3d,&camera.normal);
                }else if p3_3d.2 < camera.focal_distf {
                    p1 = plane_clipping(p1_3d,p2_3d,&camera.normal);
                    p2 = projection(p2_3d,&camera.normal);
                    p3 = plane_clipping(p3_3d,p2_3d,&camera.normal);
                }else{
                    let pr2 = plane_clipping(p2_3d,p1_3d,&camera.normal);
                    let pr3 = plane_clipping(p3_3d,p1_3d,&camera.normal);
                    p2 = projection(p2_3d,&camera.normal);
                    p3 = projection(p3_3d,&camera.normal);
                    p1 = pr2;
                    triangles.push(Triangle2D::new(pr2,pr3,p3,normal.clone(),self.only_face));
                }
            }else if p2_3d.2 < camera.focal_distf {
                if p3_3d.2 < camera.focal_distf{
                    p1 = projection(p1_3d,&camera.normal);
                    p2 = plane_clipping(p2_3d,p1_3d,&camera.normal);
                    p3 = plane_clipping(p3_3d,p1_3d,&camera.normal);
                }else{
                    let pr1 = plane_clipping(p1_3d,p2_3d,&camera.normal);
                    let pr3 = plane_clipping(p3_3d,p2_3d,&camera.normal);
                    p1 = projection(p1_3d,&camera.normal);
                    p3 = projection(p3_3d,&camera.normal);
                    p2 = pr1;
                    triangles.push(Triangle2D::new(pr1,pr3,p3,normal.clone(),self.only_face));
                }
            }else if p3_3d.2 < camera.focal_distf {
                let pr1 = plane_clipping(p1_3d,p3_3d,&camera.normal);
                let pr2 = plane_clipping(p2_3d,p3_3d,&camera.normal);
                p1 = projection(p1_3d,&camera.normal);
                p2 = projection(p2_3d,&camera.normal);
                p3 = pr1;
                triangles.push(Triangle2D::new(pr1,pr2,p2,normal.clone(),self.only_face));
            }else{
                p1 = projection(p1_3d,&camera.normal);
                p2 = projection(p2_3d,&camera.normal);
                p3 = projection(p3_3d,&camera.normal);
            }
            triangles.push(Triangle2D::new(p1, p2, p3, normal,self.only_face));
            triangles
        }
    }
}
#[inline(always)]
fn plane_clipping(p1 : (f64,f64,f64), p2 : (f64,f64,f64), normal: &Normal) -> (f64,f64){
    let vec = (p2.0 - p1.0, p2.1 - p1.1, p2.2 - p1.2);
    intersection_xy(normal, vec, p1)
}

#[inline(always)]
fn point_transformation(point : (f64,f64,f64), camera: &Camera,) -> (f64,f64,f64) {
    let point_c = (point.0 - camera.posf.0, point.1 - camera.posf.1, point.2 - camera.posf.2);
    let nx     = point_c.0 * camera.rot.1.cos() - point_c.2 * camera.rot.1.sin();
    let nz_tmp = point_c.0 * camera.rot.1.sin() + point_c.2 * camera.rot.1.cos();
    let nz     = point_c.1 * camera.rot.2.sin() + nz_tmp * camera.rot.2.cos();
    let ny     = point_c.1 * camera.rot.2.cos() - nz_tmp * camera.rot.2.sin();
    (nx, ny, nz)
}

#[inline(always)]
const fn projection(point : (f64,f64,f64), normal: &Normal) -> (f64,f64){
    if is_secant(normal,(point.0 as isize,point.1 as isize, point.2 as isize)) {
            intersection_xy(normal,point,point)
    }else{
        (0f64,0f64)
    }
}

#[derive(Debug)]
struct Triangle2D{
    p1i : (f64,f64), // position
    p2i : (f64,f64), // position
    p3i : (f64,f64), // position
    square_x : isize,
    square_y : isize,
    square_width : isize,
    square_height : isize,
    only_face : bool,
    normal: Normal,
}

impl Triangle2D{
    pub const fn new(p1i : (f64,f64), p2i : (f64,f64), p3i : (f64,f64), normal: Normal, only_face : bool) -> Triangle2D{
        let p1 = (p1i.0 as isize,p1i.1 as isize);
        let p2 = (p2i.0 as isize,p2i.1 as isize);
        let p3 = (p3i.0 as isize,p3i.1 as isize);
        Triangle2D{
            p1i,
            p2i,
            p3i,
            square_x: min3(p1.0, p2.0, p3.0),
            square_y: min3(p1.1, p2.1, p3.1),
            square_width: max3(p1.0, p2.0, p3.0),
            square_height: max3(p1.1, p2.1, p3.1),
            only_face,
            normal,
        }
    }
}

#[inline(always)]
const fn sign(p1 : (isize,isize), p2 : (f64,f64), p3 : (f64,f64)) -> f64{
    (p1.0 as f64 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 as f64 - p3.1)
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
    normal.dot_product(droite) != 0
}

#[inline(always)]
const fn intersection_t(normal : &Normal, droite: (f64,f64,f64), point : (f64,f64,f64)) -> f64{
    -(normal.dot_product_f64(point)  + normal.constantf)/ normal.dot_product_f64(droite)
}
#[inline(always)]
const fn intersection_z(value : isize, prod : isize, normal_const : isize) -> isize{
    value - (value * (prod + normal_const))/prod
}

#[inline(always)]
const fn intersection_xy(normal : &Normal, droite : (f64,f64,f64), point : (f64,f64,f64)) -> (f64,f64){
    let t = intersection_t(normal, droite, point);
    let x = point.0 + droite.0*t;
    let y = point.1 + droite.1*t;
    (x, y)
}

#[inline(always)]
const fn encode_isize(z : isize, color : u32) -> isize{
    (color as isize) | z << 32
}

#[allow(dead_code)]
#[inline(always)]
const fn decode(code : isize) -> (isize,u32) {
    let z = code >> 32;
    let color : u32 = code as u32;
    (z as isize,color)
}

#[inline(always)]
const fn same_signe(x : isize, y : isize) -> bool{
    ((x as usize ^ (y as usize)) & 0x8000000000000000) == 0
}

#[inline(always)]
fn vec_size(vec : (f64,f64,f64)) -> f64{
    ((vec.0 * vec.0) + (vec.1 * vec.1) + (vec.2 * vec.2)).sqrt()
}

#[inline(always)]
fn compute_triangle(triangles: &[Triangle3D], camera: &Camera, atomic_buffer: &mut Vec<AtomicIsize>) {
    atomic_buffer.par_iter().for_each(|p| {p.store(encode_isize(VIEW_DISTANCE,0x202030), Ordering::Relaxed)});

    triangles.par_iter().for_each(|tri_3d| {
        let dt = -tri_3d.normal.dot_product_f64(camera.sun);
        let mut light = 220;
        if dt != 0f64{
            let sz = vec_size(camera.sun) * vec_size(tri_3d.normal.to_vec_f64());
            let angle = dt/sz;
            if angle > 0f64{
                light = (255f64 - (255f64*angle)) as u32;
            }
        }
        if light > 220{
            light = 220
        }
        let r = ((tri_3d.color >> 16).saturating_sub(light)) << 16;
        let g = (((tri_3d.color >> 8) & 0x00FF).saturating_sub(light)) << 8;
        let b = (tri_3d.color & 0x0000FF).saturating_sub(light);
        let color = r+g+b;

        for tri in tri_3d.to_2d(camera).iter() {
            if tri.only_face && same_signe(tri.normal.nz,camera.normal.nz){
                continue;
            }

            let min_x = if tri.square_x > -WIDTH2I { tri.square_x - 1 } else { -WIDTH2I };
            let min_y = if tri.square_y > -HEIGHT2I { tri.square_y - 1} else { -HEIGHT2I };
            let max_x = if tri.square_width < WIDTH2I { tri.square_width + 1 } else { WIDTH2I };
            let max_y = if tri.square_height < HEIGHT2I { tri.square_height + 1} else { HEIGHT2I };

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let d1 = sign((x, y), tri.p1i, tri.p2i);
                    let d2 = sign((x, y), tri.p2i, tri.p3i);
                    let d3 = sign((x, y), tri.p3i, tri.p1i);
                    if (d1.signum() == d2.signum()) && (d2.signum() == d3.signum()){
                        let idx = (y + HEIGHT2I) as usize * WIDTH + (x + WIDTH2I) as usize;
                        let dot = tri.normal.dot_product((x,y,camera.focal_dist));
                        if dot != 0 {
                            let z = intersection_z(camera.focal_dist,dot,tri.normal.constant);
                            if z > camera.focal_dist {
                                atomic_buffer[idx].fetch_min(encode_isize(z, color), Ordering::Relaxed);
                            }
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

const FPS : usize = 250;
fn main() {
    let mut buffer : Vec<u32> = vec![0; TRIANGLE_NB];
    let mut atomic_buffer : Vec<AtomicIsize> = (0..(WIDTH*HEIGHT)).map(|_| AtomicIsize::new(encode_isize(VIEW_DISTANCE,0))).collect();
    let mut fps_100 = vec![0; FPS_BUFFER_SIZE];
    let mut index = 0;
    let (mut camera,triangles) = generate_config();

    // Windo management
    let mut window = Window::new("Mon moteur 3D", WIDTH, HEIGHT, WindowOptions::default())
       .unwrap();
    window.set_target_fps(FPS);

    let mut fps = 0;
    let mut old_mouse_pos = (0f32,0f32);
    let mut speed = 10;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        speed = 10;
        if let Some(pos) = window.get_mouse_pos(MouseMode::Discard) {
            if old_mouse_pos == (0f32,0f32) {
                old_mouse_pos = pos;
            }else {
                let diff_x = (pos.0 - old_mouse_pos.0) / 5000f32;
                let diff_y = (pos.1 - old_mouse_pos.1) / 5000f32;
                camera.rot.1 += diff_x.to_degrees() as f64;
                camera.rot.2 += diff_y.to_degrees() as f64;
                old_mouse_pos = (pos.0, pos.1);
            }
        }else{
            old_mouse_pos = (0f32,0f32)
        }
        if window.is_key_down(Key::LeftShift){
            speed = 100;
        }
        if window.is_key_down(Key::X){
            camera.rot.1 -= 1f64.to_radians();
        }
        if window.is_key_down(Key::D){
            camera.move_camera((speed,0,0));
        }
        if window.is_key_down(Key::Q){
            camera.move_camera((-speed,0,0));
        }
        if window.is_key_down(Key::Z){
            camera.move_camera((0,0,speed));
        }
        if window.is_key_down(Key::S){
            camera.move_camera((0,0,-speed));
        }
        if window.is_key_down(Key::A){
            camera.move_camera((0,speed,0));
        }
        if window.is_key_down(Key::E){
            camera.move_camera((0,-speed,0));
        }
        let start = Instant::now();
        compute_triangle(&triangles,&camera,&mut atomic_buffer);
        buffer = atomic_buffer.iter().map(|p| p.load(Ordering::Relaxed) as u32).collect::<Vec<u32>>();

        let d = start.elapsed().as_nanos();
        let d2 = 1_000_000_000/d;
        fps += d2 - fps_100[index];
        fps_100[index] = d2;
        index = (index + 1) %FPS_BUFFER_SIZE;
        println!("\r FPS : mean {} instant {} index {} ",fps/FPS_BUFFER_SIZE_128,d2,index);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}