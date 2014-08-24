var ws = new WebSocket('ws://localhost:8001/');

var t = new Date;
ws.onmessage = function(event){
  var now = new Date;
  //console.log(now.getTime() - t.getTime());
  t = new Date;

  var objects = JSON.parse(event.data);
  _(objects).each(function(obj){
    obj = JSON.parse(obj); // XXX: needs a fix on server end
    var id = obj[0];
    var transform = obj[1];
    var rot = transform.rotation.submat;
    var pos = transform.translation;
    var gameObject = GameObjects[id];
    if (!gameObject){
      gameObject = new THREE.Mesh(geometry, material);
      gameObject.matrixAutoUpdate = false; // we set the matrix ourselves
      scene.add(gameObject);
      GameObjects[id] = gameObject;
    }
    gameObject.matrix.set(
      rot.m11, rot.m12, rot.m13, pos.x,
      rot.m21, rot.m22, rot.m23, pos.y,
      rot.m31, rot.m32, rot.m33, pos.z,
      0,       0,       0,       1
    );
    gameObject.matrixWorldNeedsUpdate = true;
    //console.log(id, rot, pos);
  });
};

var GameObjects = {};






var scene = new THREE.Scene();
var camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);

var axisHelper = new THREE.AxisHelper( 25 );
scene.add( axisHelper );


var controls = new THREE.TrackballControls( camera );
controls.rotateSpeed = 1.0;
controls.zoomSpeed = 1.2;
controls.panSpeed = 0.8;
controls.noZoom = false;
controls.noPan = false;
controls.staticMoving = true;
controls.dynamicDampingFactor = 0.3;

var renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

var geometry = new THREE.BoxGeometry(2,2,2);
var material = new THREE.MeshNormalMaterial({color: 0x00ff00});
//var cube = new THREE.Mesh(geometry, material);

//cube.matrixAutoUpdate = false; // we set the matrix ourselves

//scene.add(cube);

//camera.lookAt(cube.position);
camera.position.x = 5
camera.position.y = 5
camera.position.z = 100;

var render = function () {
  requestAnimationFrame(render);

  //cube.rotation.x += 0.1;
  //cube.rotation.y += 0.1;

  //camera.lookAt(cube.position);
  controls.update();
  renderer.render(scene, camera);
};

render();
