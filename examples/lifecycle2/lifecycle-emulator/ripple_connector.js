
export class FireboltGatewayConnector {
   constructor(path, callback) {
    this.path = path;
    callback.log("connecting to " + path);
    this.socket = new WebSocket(path);
    this.callback = callback;
    this.socket.onopen = (event) => {
        callback.log("Connection Established");
        callback.onConnect(this)
    };
    this.socket.onerror = (err) => {
      callback.log("Connection Error" + JSON.stringify(err))
    }
    this.socket.onmessage = (ev) => {
      let parsedObject = JSON.parse(ev.data);
      if (parsedObject && parsedObject.id && this.requestMap[parsedObject.id]) {
         callback.onMessage(parsedObject, this.requestMap[parsedObject.id])
      } else {
         callback.onMessage(parsedObject)
      }
    }
    this.socket.onclose = (close) => {
      callback.log("Successfully Closed")
    }
    this.id = 1;
   }

   getId() {
      let currentId = this.id;
      this.id++;
      return currentId
   }

   addToRequestMap(id, entry) {
      if (this.requestMap == undefined){
         this.requestMap = {}
      }
      this.requestMap[id] = entry
   }

   send(method, params) {
      let id = this.getId();
      let request = {
         "jsonrpc": "2.0",
         "method": method,
         "params": params,
         "id" : id
      };
      let message = JSON.stringify(request);
      this.addToRequestMap(id, request);
      this.socket.send(message)
      this.callback.log("Successfully sent >>> ", request); 
   }

   respond(id, result) {
      let response = {
         "jsonrpc": "2.0",
         "result": result,
         "id" : id
      };
      let message = JSON.stringify(response);
      this.socket.send(message)
      this.callback.log("Successfully Responded >>> ", response);
   }
}

