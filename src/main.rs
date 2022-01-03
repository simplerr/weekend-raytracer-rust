#[allow(unused_imports, dead_code)]
mod utopian_math;

#[allow(dead_code)]
use utopian_math::*;

use std::rc::Rc;
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

trait Material {
    fn scatter(&self, input_ray: &Ray, hit_record: &HitRecord, attenuation: &mut Vec3, scattered_ray: &mut Ray) -> bool;
}

struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    fn new(albedo: Vec3) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _input_ray: &Ray, hit_record: &HitRecord, attenuation: &mut Vec3, scattered_ray: &mut Ray) -> bool {
        // Note: randomPointInUnitSphere() can be replaced by other distributions,
        // see chapter 8.5 in the tutorial.
        let mut scatter_direction = hit_record.normal + random_point_in_unit_sphere();

        if scatter_direction.length() < f32::EPSILON {
            scatter_direction = hit_record.normal;
        }

        *scattered_ray = Ray { origin: hit_record.pos, dir: scatter_direction };
        *attenuation = self.albedo;

        true
    }
}

struct Metal {
    albedo: Vec3,
    fuzz: f32,
}

impl Metal {
    fn new(albedo: Vec3, fuzz: f32) -> Metal {
        Metal { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, input_ray: &Ray, hit_record: &HitRecord, attenuation: &mut Vec3, scattered_ray: &mut Ray) -> bool {
        let reflected = input_ray.dir.normalize().reflect(hit_record.normal);
        *scattered_ray = Ray {
            origin: hit_record.pos,
            dir: reflected + self.fuzz * random_point_in_unit_sphere(),
        };
        *attenuation = self.albedo;

        scattered_ray.dir.dot(hit_record.normal) > 0.0
    }
}

struct Dielectric {
    ir: f32,
}

impl Dielectric {
    fn new(ir: f32) -> Dielectric {
        Dielectric { ir }
    }

    fn calc_reflectance(&self, cosine: f32, ref_idx: f32) -> f32 {
        // Schlick's approximation
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1.0 - r0) * ((1.0 - cosine).powf(5.0))
    }
}

impl Material for Dielectric {
    fn scatter(&self, input_ray: &Ray, hit_record: &HitRecord, attenuation: &mut Vec3, scattered_ray: &mut Ray) -> bool {
        *attenuation = vec3(1.0, 1.0, 1.0);

        let refraction_ratio = match hit_record.front_face {
            true => 1.0 / self.ir,
            false => self.ir,
        };

        let normalized_direction = input_ray.dir.normalize();
        let cos_theta = (-1.0 * normalized_direction).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let mut direction = Vec3::default();

        if cannot_refract || self.calc_reflectance(cos_theta, refraction_ratio) > random_float(0.0, 1.0) {
            direction = normalized_direction.reflect(hit_record.normal);
        }
        else {
            direction = normalized_direction.refract(hit_record.normal, refraction_ratio);
        }

        *scattered_ray = Ray {
            origin: hit_record.pos,
            dir: direction,
        };

        true
    }
}

struct Ray {
    origin: Vec3,
    dir: Vec3,
}

struct HitRecord {
    pos: Vec3,
    normal: Vec3,
    t: f32,
    front_face: bool,
    material: Rc<dyn Material>,
}

struct Sphere {
    center: Vec3,
    radius: f32,
    material: Rc<dyn Material>,
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
    lens_radius: f32,
}

impl Camera {
    fn new(look_from: Vec3, look_at: Vec3, vertical_fov: f32, aspect_ratio: f32, aperture: f32, focus_dist: f32) -> Camera {
        let theta = vertical_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

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
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.dir.length_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let mut hit_record = HitRecord {
            t: root,
            pos: ray.at(root),
            normal: vec3(0.0, 0.0, 0.0),
            front_face: false,
            material: Rc::clone(&self.material),
        };

        let outward_normal = (hit_record.pos - self.center) / self.radius;
        hit_record.set_face_normal(ray, outward_normal);

        return Some(hit_record);
    }
}

impl World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut temp_record = None;
        let mut closest_hit = t_max;

        for object in &self.objects {
            if let Some(rec) = object.hit(ray, t_min, closest_hit) {
                closest_hit = rec.t;
                temp_record = Some(rec);
            }
        }

        temp_record
    }
}

fn ray_color(ray: &Ray, world: &World, depth: i32) -> Vec3 {

    if depth < 0 {
        return vec3(0.0, 0.0, 0.0);
    }

    let shadow_acne_constant = 0.001;
    if let Some(rec) = world.hit(ray, shadow_acne_constant, 100.0) {
        let mut attenuation = Vec3::default();
        let mut scattered_ray = Ray { origin: vec3(0.0, 0.0, 0.0), dir: vec3(0.0, 0.0, 0.0) };
        if rec.material.scatter(ray, &rec, &mut attenuation, &mut scattered_ray) {
            return attenuation * ray_color(&scattered_ray, &world, depth - 1);
        }

        return vec3(0.0, 0.0, 0.0);
    }

    let t = 0.5 * (ray.dir.normalize().y + 1.0);
    (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
}

fn render(camera: &Camera, world: &World, width: u32, height: u32, samples_per_pixel: u32, max_depth: i32) {
    print!("P3\n{} {}\n255\n", width, height);

    eprintln!("Rendering {}x{} image", width, height);

    for y in (0..height).rev() {
        for x in 0..width {
            let mut color = Vec3::default();
            for _s in 0..samples_per_pixel {
                let u = (x as f32 + random_float(0.0, 1.0)) / ((width - 1) as f32);
                let v = (y as f32 + random_float(0.0, 1.0)) / ((height - 1) as f32);
                let ray = camera.get_ray(u, v);
                color = color + ray_color(&ray, &world, max_depth);
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

fn create_random_scene() -> World {
    let mut world = World::default();

    let ground_material = Rc::new(Lambertian::new(vec3(0.5, 0.5, 0.5)));
    let lambertian_material = Rc::new(Lambertian::new(vec3(0.4, 0.2, 0.1)));
    let metal_material = Rc::new(Metal::new(vec3(0.7, 0.6, 0.5), 0.0));
    let dielectric_material = Rc::new(Dielectric::new(1.5));

    world.objects.push(Sphere { center: vec3(0.0, -1000.0, 0.0), radius: 1000.0, material: ground_material });
    world.objects.push(Sphere { center: vec3(-4.0, 1.0, 0.0), radius: 1.0, material: lambertian_material });
    world.objects.push(Sphere { center: vec3(0.0, 1.0, 0.0), radius: 1.0, material: dielectric_material });
    world.objects.push(Sphere { center: vec3(4.0, 1.0, 0.0), radius: 1.0, material: metal_material });

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_float(0.0, 1.0);
            let center = vec3((a as f32) + 0.9 * random_float(0.0, 1.0), 0.2, (b as f32) + 0.9 * random_float(0.0, 1.0));

            if (center - vec3(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let color1 = vec3(random_float(0.0, 1.0), random_float(0.0, 1.0), random_float(0.0, 1.0));
                    let color2 = vec3(random_float(0.0, 1.0), random_float(0.0, 1.0), random_float(0.0, 1.0));
                    let albedo = color1 * color2;
                    let material = Rc::new(Lambertian::new(albedo));
                    world.objects.push(Sphere { center, radius: 0.2, material });
                }
                else if choose_mat < 0.95 {
                    let albedo = vec3(random_float(0.5, 1.0), random_float(0.5, 1.0), random_float(0.5, 1.0));
                    let fuzz = random_float(0.0, 0.5);
                    let material = Rc::new(Metal::new(albedo, fuzz));
                    world.objects.push(Sphere { center, radius: 0.2, material });
                }
                else {
                    let material = Rc::new(Dielectric::new(1.5));
                    world.objects.push(Sphere { center, radius: 0.2, material });
                };
            }
        }
    }

    world
}

fn main() {
    //vector_tests();
    let aspect_ratio = 3.0 / 2.0;
    let width = 1200;
    let height = (width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 10;
    let max_depth: i32 = 50;
    let _num_threads = 16;

    let camera = Camera::new(vec3(13.0, 2.0, 3.0), vec3(0.0, 0.0, 0.0), 20.0, aspect_ratio, 0.1, 10.0);
    let world = create_random_scene();

    render(&camera, &world, width, height, samples_per_pixel, max_depth);
}

