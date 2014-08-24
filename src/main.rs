extern crate ws;
extern crate http;
extern crate time;
extern crate nphysics = "nphysics3df32";
extern crate ncollide = "ncollide3df32";
extern crate nalgebra;
extern crate serialize;

use serialize::json;
use serialize::json::ToJson;

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

        w.write(b"
            <body>
                <script src='js/three.min.js'></script>
                <script src='js/TrackballControls.js'></script>
                <script src='js/underscore.min.js'></script>
                <script src='js/main.js'></script>
            </body>
        ");
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

struct Body { b: Rc<RefCell<RigidBody>>, id: i32 }
impl ToJson for Body {
    fn to_json(&self) -> json::Json {
        // UGH, use json::Object instead! after figuring out how to implement tojson for nphysics transforms
        json::String(format!("[{}, {}]", self.id, json::encode((*self.b).borrow().transform_ref())))
    }
}

fn main(){
    let mut world = World::new();
    world.set_gravity(Vec3::new(0.0f32, -9.81, 0.0));
    let mut bodies = vec![];
    let mut id = 0i32;

    let geom = Plane::new(Vec3::new(0.0f32, 1.0, 0.0));
    let rb   = RigidBody::new_static(geom, 0.3, 0.6);
    let body = Rc::new(RefCell::new(rb));

    world.add_body(body.clone());

    let num     = 3;
    let rad     = 1.0;
    let shift   = rad * 2.0;
    let centerx = shift * (num as f32) / 2.0;
    let centery = 40.0 + shift / 2.0;
    let centerz = shift * (num as f32) / 2.0;

    for i in range(0u, num) {
        for j in range(0u, num) {
            for k in range(0u, num) {
                let x = i as f32 * shift - centerx;
                let y = j as f32 * shift + centery;
                let z = k as f32 * shift - centerz;

                let mut rb = RigidBody::new_dynamic(Cuboid::new(Vec3::new(1.0, 1.0, 1.0)), 1.0f32, 0.3, 0.6);
                rb.append_translation(&Vec3::new(x, y, z));
                let body = Rc::new(RefCell::new(rb.clone()));

                world.add_body(body.clone());
                bodies.push(Body { b: body.clone(), id: id});
                id += 1;
            }
        }
    }

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

        let broadcast_message = box Message {
            payload: Text(box bodies.to_json().to_string()),
            opcode: TextOp,
        };
        let mut netman = netman.lock();
        netman.broadcast(broadcast_message);
    }
}
