#[allow(unused_imports, dead_code)]
mod utopian_math;

#[allow(dead_code)]
use utopian_math::*;

use rand::Rng;

fn vector_tests() {
    let a = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0
    };

    dbg!(&a);
    println!("Length squared: {}", a.length_squared());
    println!("Length: {}", a.length());

    let b = Vec3::default();
    dbg!(&b);

    let c = vec3(10.0, 20.0, 30.0);
    dbg!(&c);

    let mut d = a + c;
    d = d - a - a + vec3(100.0, 200.0, 300.0);
    dbg!(&d);

    d = d + 2.0;
    d = d - 20.0;
    dbg!(&d);

    d = d * 2.0;
    dbg!(&d);

    let dot_result = c.dot(c);
    dbg!(dot_result);

    let cross_result = c.cross(d);
    dbg!(cross_result);

    if d != a {
        println!("Equal!");
    }
    else {
        println!("Not Equal!");
    }
}

fn random_float(min: f32, max: f32) -> f32 {
    rand::thread_rng().gen_range(min..max)
}

fn random_point_in_unit_sphere() -> Vec3 {
    loop {
        let point = vec3(random_float(-1.0, 1.0), random_float(-1.0, 1.0), random_float(-1.0, 1.0));
        if point.length() < 1.0 {
            return point;
        }
    }
}

fn random_point_in_unit_disc() -> Vec3 {
    loop {
        let point = vec3(random_float(-1.0, 1.0), random_float(-1.0, 1.0), 0.0);
        if point.length_squared() < 1.0 {
            return point;
        }
    }
}

struct Ray {
    origin: Vec3,
    dir: Vec3,
}

#[derive(Default)]
struct HitRecord {
    pos: Vec3,
    normal: Vec3,
    t: f32,
    front_face: bool,
}

#[derive(Debug)]
struct Sphere {
    center: Vec3,
    radius: f32,
}

#[derive(Default)]
struct World {
    objects: Vec<Sphere>,
}

struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: f32,
}

impl Camera {
    fn new(look_from: Vec3, look_at: Vec3, vertical_fov: f32, aspect_ratio: f32, aperture: f32, focus_dist: f32) -> Camera {
        let theta = vertical_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let focal_length = 1.0;
        let up = vec3(0.0, 1.0, 0.0);

        let w = (look_from - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = focus_dist * u * viewport_width;
        let vertical = focus_dist * v * viewport_height;

        Camera {
            u,
            v,
            w,
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - (horizontal / 2.0) - (vertical / 2.0) - (focus_dist * w),
            lens_radius: aperture / 2.0,
        }
    }

    fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_point_in_unit_disc();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin + offset,
            dir: self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        }
    }
}

impl Ray {
    fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }
}


impl HitRecord {
    fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.dir.dot(outward_normal) < 0.0;

        self.normal = match self.front_face {
            true => outward_normal,
            false => outward_normal * vec3(-1.0, -1.0, -1.0),
        };
    }
}

impl Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> bool {
        let oc = ray.origin - self.center;
        let a = ray.dir.length_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }

        hit_record.t = root;
        hit_record.pos = ray.at(hit_record.t);
        let outward_normal = (hit_record.pos - self.center) / self.radius;
        hit_record.set_face_normal(ray, outward_normal);
        return true;
    }
}

impl World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> bool {
        let mut hit_anything = false;
        let mut closest_hit = t_max;

        for object in &self.objects {
            let mut temp_record = HitRecord::default();
            if object.hit(ray, t_min, closest_hit, &mut temp_record) {
                hit_anything = true;
                closest_hit = temp_record.t;
                *hit_record = temp_record;
            }
        }

        hit_anything
    }
}

fn ray_color(ray: &Ray, world: &World) -> Vec3 {
    let mut hit_record = HitRecord::default();

    if world.hit(ray, 0.0, std::f32::INFINITY, &mut hit_record) {
        return 0.5 * (hit_record.normal + 1.0);
    }

    let t = 0.5 * (ray.dir.normalize().y + 1.0);
    (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
}

fn render(camera: &Camera, world: &World, width: u32, height: u32, samples_per_pixel: u32, max_depth: u32) {
    print!("P3\n{} {}\n255\n", width, height);

    eprintln!("Rendering {}x{} image", width, height);

    for y in (0..height).rev() {
        for x in 0..width {
            let mut color = Vec3::default();
            for _s in 0..samples_per_pixel {
                let u = (x as f32 + random_float(0.0, 1.0)) / ((width - 1) as f32);
                let v = (y as f32 + random_float(0.0, 1.0)) / ((height - 1) as f32);
                let ray = camera.get_ray(u, v);
                color = color + ray_color(&ray, &world);
            }

            color = color / (samples_per_pixel as f32);
            color = color.sqrt();
            color = color.clamp(0.0, 0.999);

            let red = (color.x * 256.0) as u32;
            let green = (color.y * 256.0) as u32;
            let blue = (color.z * 256.0) as u32;

            println!("{} {} {}", red, green, blue);
        }

        eprint!(".");
    }

    eprintln!("\nRendering finished!");
}

fn main() {
    //vector_tests();
    let aspect_ratio = 3.0 / 2.0;
    let width = 1200;
    let height = (width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 10;
    let max_depth = 50;
    let _num_threads = 16;

    let camera = Camera::new(vec3(13.0, 2.0, 3.0), vec3(0.0, 0.0, 0.0), 20.0, aspect_ratio, 0.1, 10.0);
    let mut world: World = World::default();

    world.objects.push(Sphere { center: vec3(0.0, -1000.0, 0.0), radius: 1000.0 });
    world.objects.push(Sphere { center: vec3(-4.0, 1.0, 0.0), radius: 1.0 });
    world.objects.push(Sphere { center: vec3(0.0, 1.0, 0.0), radius: 1.0 });
    world.objects.push(Sphere { center: vec3(4.0, 1.0, 0.0), radius: 1.0 });

    render(&camera, &world, width, height, samples_per_pixel, max_depth);
}

