#version 450
#define FLT_MAX 3.402823466e+38
#define MAT_DIFFUSE 0
#define MAT_METAL   1


layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

struct Ray {
	vec3 origin;
	vec3 direction;
};


struct Material {
    int instance_of;
    vec3 albedo;
    float roughness;
};

struct Sphere {
    vec3 center;
    float radius;
    Material material;
};

layout (binding = 1) buffer SphereList {
    Sphere spheres[ ];
};

struct HitRecord {
    float t;
    vec3 point;
    vec3 normal;
    Material material;
};

highp float rand(vec2 co)
{
    highp float a = 12.9898;
    highp float b = 78.233;
    highp float c = 43758.5453;
    highp float dt= dot(co.xy ,vec2(a,b));
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

highp float trand(vec2 co, float seed) {
    return rand(co * seed);
}

vec3 random_in_unit_sphere(vec2 uv, float seed) {
    vec3 p;
    do {
        p = 2.0*vec3(trand(uv, seed), trand(uv, seed*2.334), trand(uv, seed*-1.334)) - vec3(1,1,1);
        seed *= 1.312;
    } while (dot(p, p) >= 1.0);
    return p;
}

vec3 point_at_parameter(Ray ray, float t) {
    return ray.origin + t*ray.direction;
}

bool hit_sphere(Sphere self, Ray ray, float t_min, float t_max, inout HitRecord record) {
    vec3 oc = ray.origin - self.center;
    float a = dot(ray.direction, ray.direction);
    float b = dot(oc, ray.direction);
    float c = dot(oc, oc) - self.radius * self.radius;
    float discriminant = b * b - a * c;

    if (discriminant > 0.0) {
        // check - root
        float temp = (-b - sqrt(discriminant)) / a;
        if (temp < t_max && temp > t_min) {
            record.t = temp;
            record.point = point_at_parameter(ray, temp);
            record.normal = (record.point - self.center)/ self.radius;
            record.material = self.material;

            return true;
        }

        // check + root
        temp = (-b + sqrt(discriminant)) / a;
        if (temp < t_max && temp > t_min) {
            record.t = temp;
            record.point = point_at_parameter(ray, record.t);
            record.normal = (record.point - self.center)  / self.radius;
            record.material = self.material;

            return true;
        }
    }
    return false;
}

bool hit_world(Ray ray, float t_min, float t_max, inout HitRecord record) {
    HitRecord temp_rec;
    bool hit_anything = false;
    float closest_so_far = t_max;
    for (int i = 0; i < spheres.length(); i++) {
        if (hit_sphere(spheres[i], ray, t_min, closest_so_far, temp_rec)) {
            hit_anything = true;
            closest_so_far = temp_rec.t;
            record = temp_rec;
        }
    }
    return hit_anything;
}


bool diffuse_scatter(inout Ray ray, vec2 uv, HitRecord record, out vec3 attenuation) {
    vec3 target = record.point + record.normal + random_in_unit_sphere(uv, 1.0);
    ray = Ray(record.point, target - record.point);
    attenuation = record.material.albedo;
    return true;
}

bool scatter(inout Ray ray, vec2 uv, HitRecord record, out vec3 attenuation) {
    // based on the material ID of the current hit, extract the right// material parameters and call the specific scatter 
    if (record.material.instance_of == MAT_DIFFUSE) {
        return diffuse_scatter(ray, uv, record, attenuation);
    } else if (record.material.instance_of == MAT_METAL) {
        //return MetalScatter(r, uv, hit, attenuation);
        return false;
    } else {
        return false;
    }
}


vec3 missed_color(vec3 direction) {
    vec3 unit_direction = normalize(direction);
    float t = 0.5 * (unit_direction.y + 1.0);
    return mix(vec3(1.0, 1.0, 1.0), vec3(.5, .7, 1.0), t);
}


vec3 color(in Ray ray, vec2 uv) {
    int max_bounces = 5;
    vec3 final_color = vec3(1.0);
    HitRecord record;
    float seed = 1.0;

    for (int i = 0; i < max_bounces; ++i) {
        if (hit_world(ray, 0.0, FLT_MAX, record)) {
            vec3 attenuation;
            if (scatter(ray, uv * seed * 0.897, record, attenuation)) {
                final_color *= attenuation;
            }
            else {
                final_color *= vec3(0.0);
                break;
            }
            seed *= 1.456;
        }
        else {
            final_color *= missed_color(ray.direction);
        }
    }

    return final_color;
}

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
    int num_samples = 100;

    vec2 size = vec2(imageSize(img));
    uint idx = gl_GlobalInvocationID.x;
    uint idy = uint(size.y) - gl_GlobalInvocationID.y;

    vec3 lower_left_corner = vec3(-2.0, -1.0, -1.0);
    vec3 horizontal = vec3(4.0, 0.0, 0.0);
    vec3 vertical = vec3(0.0, 2.0, 0.0);
    vec3 origin = vec3(0.0, 0.0, 0.0);

    vec3 col = vec3(0, 0, 0);
    for (int i = 0; i < num_samples; ++i) {
        float rand = rand(vec2(i));
        float u = float(idx + rand) / float(size.x);
        float v = float(idy + rand) / float(size.y);

        Ray ray = Ray(origin,  lower_left_corner + u*horizontal + v*vertical);
        col += color(ray, vec2(u, v));
    }
    col /= float(num_samples);

    //col = vec3(sqrt(col.x), sqrt(col.y), sqrt(col.z));
    col = sqrt(col);

    vec4 to_write = vec4(col, 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}

