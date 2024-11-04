import van from "vanjs-core"
const {a, div, li, p, ul, h1, h2, button, pre,input} = van.tags
import { FireboltGatewayConnector } from "./ripple_connector"
import {FloatingWindow, Tabs, topMostZIndex} from "vanjs-ui"


export const SessionRequest = (type) => {
    let request = {
        "session":
        {"app":
          {"id":"firebolt2App","title":"Firebolt App 2.0"},
          "launch":{
            "inactive":false,
            "intent":{
              "action":"launch",
              "context":{
                "source":"device"
              }
            }
          },"runtime":{
            "id":"firebolt.runtime.system",
            "transport":"websocket"
          }
      }
    };

    if (type == 1) {
        request.session.app.id = "firebolt1App";
        request.session.app.title = "Firebolt App 1.0"
    }
    return request
}

export class FireboltApp {
    constructor(request) {
        this.type = 1;
        this.title = request.params.session.app.title;
        if (request && request.params.session.app.id.startsWith("firebolt2App")) {
            this.type = 2;
        }
        if (this.type == 2) {
            this.headerColor = "lightgreen"
            this.logColor = "honeydew"
            this.inputDom = input({type: "text"})
        } else {
            this.headerColor = "salmon"
            this.logColor = "mistyrose"
        }
        this.panel = div(h2("Panel"))
        this.app_logs = ul({style: "width: 450px;padding:0;list-style-type: none; margin: 0"},li(h2("App Logs")))
    }

    launch(sessionId) {
        this.sessionId = sessionId
        van.add(document.body, FloatingWindow(
            {title: this.title, x: 150 * this.type, y: 150 * this.type, width: 500, height: 560, headerColor: this.headerColor},
            div({style: "height: 500px; overflow:scroll; justify-content: left;"},
              this.panel,
              this.app_logs
            ),
        ))
        this.connector = new FireboltGatewayConnector("ws://127.0.0.1:3473?session="+sessionId, this)
    }

    log(data, result){
        if(!result) {
          van.add(this.app_logs,li(data))
        } else {
          van.add(this.app_logs,li(data))
          van.add(this.app_logs, li({"style": "background-color: "+ this.logColor},pre(JSON.stringify(result, null, 2)) ))
        }
    }
    
    onConnect(appConnector){
        van.add(this.app_logs,li("Successfully connected to Ripple using sessionId"))
        if(this.type==2) {
            appConnector.send("rpc.initialize", { "context": ["eventBased"]})
            van.add(this.panel, button({onclick : () => {
              appConnector.send("keyboard.provideStandard", {})
            }},"Register Keyboard Provider"))
        } else {
          van.add(this.panel, button({onclick : () => {
            appConnector.send("keyboard.standard", {
              "message": "Some message on keyboard"
            })
          }},"Get Keyboard"))
        }

        
        setTimeout(() => {
            appConnector.send("localization.onCountryCodeChanged", {
                "listen":true
              })
        }, 500)   
    }
    
    onMessage(message, request){
        this.log("Recieved <<< ", message)
        if (message.method == "keyboard.standard") {
          van.add(this.app_logs, li(message.params.message))
          van.add(this.app_logs,li(this.inputDom), button(
            { onclick: () => {
              this.connector.respond(message.id, this.inputDom.value)
            }
            }, "Send Keyboard Response")
          )
        }
    }
}
