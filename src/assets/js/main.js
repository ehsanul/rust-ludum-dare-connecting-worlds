var ws = new WebSocket('ws://localhost:8001/');
ws.onmessage = function(event){
  var transform = JSON.parse(event.data);
  var rot = transform.rotation.submat;
  var pos = transform.translation;
  //debugger
  cube.matrix.set(
    rot.m11, rot.m12, rot.m13, pos.x,
    rot.m21, rot.m22, rot.m23, pos.y,
    rot.m31, rot.m32, rot.m33, pos.z,
    0,       0,       0,       1
  );
  cube.matrixWorldNeedsUpdate = true;
  console.log(rot, pos)
  //console.log(cube.matrix);
};







var scene = new THREE.Scene();
var camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);

var renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

var geometry = new THREE.BoxGeometry(1,1,1);
var material = new THREE.MeshNormalMaterial({color: 0x00ff00});
var cube = new THREE.Mesh(geometry, material);

cube.matrixAutoUpdate = false; // we set the matrix ourselves

scene.add(cube);

camera.lookAt(cube.position);
camera.position.x = 5
camera.position.y = 5
camera.position.z = 10;

var render = function () {
  requestAnimationFrame(render);

  //cube.rotation.x += 0.1;
  //cube.rotation.y += 0.1;

  //camera.lookAt(cube.position);
  renderer.render(scene, camera);
};

render();
