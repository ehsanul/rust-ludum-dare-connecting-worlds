extern crate ws;
extern crate nphysics = "nphysics3df32";
extern crate ncollide = "ncollide3df32";
extern crate nalgebra;

use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::na::{Vec3, Translation};
use ncollide::geom::{Ball, Plane};
use nphysics::world::World;
use nphysics::object::RigidBody;

use std::io::timer;
use std::time::Duration;
use ws::message::{Message, Empty, PingOp};

fn main(){
    let x = Message { payload: Empty, opcode: PingOp };
    println!("{}", x);

    let mut world = World::new();
    world.set_gravity(Vec3::new(0.0f32, -9.81, 0.0));

    // planes
    let normals = [
        Vec3::new(-1.0f32, 1.0, -1.0 ),
        Vec3::new(1.0f32, 1.0, -1.0 ),
        Vec3::new(-1.0f32, 1.0, 1.0 ),
        Vec3::new(1.0f32, 1.0, 1.0 )
    ];
    for n in normals.iter() {
        let rb   = RigidBody::new_static(Plane::new(*n), 0.3, 0.6);
        let body = Rc::new(RefCell::new(rb));

        world.add_body(body.clone());
        //graphics.add(body); FIXME instead, our ws stuff needs a reference to it right, to send coordinates to the browser?
    }

    // ball
    let rad = 0.5;
    let mut rb = RigidBody::new_dynamic(Ball::new(rad), 1.0f32, 0.3, 0.6);
    rb.append_translation(&Vec3::new(15.0, 30.0, -15.0));
    let body = Rc::new(RefCell::new(rb));

    world.add_body(body.clone());
    //graphics.add(window, body);

    let fps = 30.0f32;
    loop {
        println!("{}", (*body).borrow().center_of_mass());
        world.step(1.0 / fps);
        let step_duration = Duration::milliseconds((1000.0 / fps) as i32);
        timer::sleep(step_duration); // FIXME take into account time spent calcuating step
    }
}
