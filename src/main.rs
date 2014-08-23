extern crate ws;
extern crate http;
extern crate time;
extern crate nphysics = "nphysics3df32";
extern crate ncollide = "ncollide3df32";
extern crate nalgebra;
extern crate serialize;

use serialize::json;

use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::na::{Vec3, Translation};
use ncollide::geom::{Cuboid, Ball, Plane};
use nphysics::world::World;
use nphysics::object::RigidBody;

use std::io::timer;
use std::time::Duration;


use http::server::{Config, Server, Request, ResponseWriter};
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use http::headers::content_type::MediaType;

use ws::message::{Message, TextOp, Text, BinaryOp, Binary};
use ws::server::WebSocketServer;


#[deriving(Clone)]
struct GameServer {
    netman: Arc<Mutex<NetMan>>
}

#[deriving(Clone)]
struct NetMan {
    players: Vec<Sender<Box<Message>>>,
}

impl NetMan {
    pub fn broadcast(&self, message: Box<Message>) {
        for player in self.players.iter() {
            player.send(message.clone());
        }
    }
}

impl Server for GameServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, r: Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: String::from_str("text"),
            subtype: String::from_str("html"),
            parameters: vec!((String::from_str("charset"), String::from_str("UTF-8"))),
        });
        w.headers.server = Some(String::from_str("GameServer"));

        w.write(b"<base href='http://localhost:8000/'/>").unwrap(); // using python -m SimpleHTTPServer as asset server for now

        w.write(b"<link rel='stylesheet' href='css/main.css'></script>").unwrap();
        w.write(b"<h1>Game Server</h1>").unwrap();

        w.write(b"<script src='js/three.min.js'></script>").unwrap();
        w.write(b"<script src='js/main.js'></script>").unwrap();
    }
}

impl WebSocketServer for GameServer {
    fn handle_ws_connect(&self, receiver: Receiver<Box<Message>>, sender: Sender<Box<Message>>) {
        let mut netman = self.netman.lock();
        (*netman).players.push(sender);

        let netman = self.netman.clone();
        spawn(proc() {

            // first message received is nick
            let nick = match receiver.recv().payload {
                Text(p) => p,
                _       => fail!(),
            };

            loop {
                let message = receiver.recv();
                let (payload, opcode) = match message.payload {
                    Text(p) => (Text(box nick.clone().append(": ").append((*p).as_slice())), TextOp),
                    _       => fail!(),
                };
                let broadcast_message = box Message {
                    payload: payload,
                    opcode: opcode,
                };
                let mut netman = netman.lock();
                netman.broadcast(broadcast_message);
            }
        });
    }
}

fn main(){
    //let x = Message { payload: Empty, opcode: PingOp };
    //println!("{}", x);

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
    let mut rb = RigidBody::new_dynamic(Cuboid::new(Vec3::new(1.0, 1.0, 1.0)), 1.0f32, 0.3, 0.6);
    rb.append_translation(&Vec3::new(15.0, 30.0, -15.0));
    let body = Rc::new(RefCell::new(rb.clone()));

    world.add_body(body.clone());
    //graphics.add(window, body);

    let netman = Arc::new(Mutex::new(NetMan { players: vec![] }));
    let cloned_netman = netman.clone();
    spawn(proc() {
        GameServer { netman: cloned_netman }.ws_serve_forever();
    });

    let fps = 30.0f32;

    // XXX nphysics uses Rc and RefCell, which I can't share between tasks :(
    // // broadcast loop
    // let broadcast_body = Arc::new(rb.clone());
    // spawn(proc() {
    //     loop {
    //         println!("here");
    //         let step_duration = Duration::milliseconds((1000.0 / fps) as i32);
    //         timer::sleep(step_duration); // FIXME take into account time spent calcuating step

    //         let broadcast_message = box Message {
    //             payload: Text(box json::encode(broadcast_body.center_of_mass())),
    //             opcode: TextOp,
    //         };
    //         let mut netman = netman.lock();
    //         netman.broadcast(broadcast_message);
    //     }
    // });

    loop {
        world.step(1.0 / fps);
        let step_duration = Duration::milliseconds((1000.0 / fps) as i32);
        timer::sleep(step_duration); // FIXME take into account time spent calcuating step

        //println!("{}", (*body).borrow().center_of_mass());

        let broadcast_message = box Message {
            payload: Text(box json::encode((*body).borrow().transform_ref())),
            opcode: TextOp,
        };
        let mut netman = netman.lock();
        netman.broadcast(broadcast_message);
    }
}
