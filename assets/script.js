const socket = new WebSocket('ws://localhost:8000/ws?user_id=96b70d7c-ab1a-4ae6-a83e-ccc583ca2bed');

socket.addEventListener('open', function (event) {
    socket.send('Hello Server!');
});

socket.addEventListener('message', function (event) {
    console.log('Message from server ', event.data);
});


setTimeout(() => {
    const obj = { hello: "world" };
    const blob = new Blob([JSON.stringify(obj, null, 2)], {
      type: "application/json",
    });
    console.log("Sending blob over websocket");
    socket.send(blob);
}, 1000);

// setTimeout(() => {
//     socket.send('About done here...');
//     console.log("Sending close over websocket");
//     socket.close(3000, "Crash and Burn!");
// }, 3000);