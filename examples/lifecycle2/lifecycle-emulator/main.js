import van from "vanjs-core"
const { div, li, p, ul, h1, h2, button, pre} = van.tags
let appManagerConnector;
import { FireboltGatewayConnector } from "./ripple_connector"
import { FireboltApp, SessionRequest } from "./app";


let appManagerConnectorHandler = {
  log: (data, result) => {
    if(!result) {
      van.add(app_manager_logs,li(data))
    } else {
      van.add(app_manager_logs,li(data))
      van.add(app_manager_logs, li({"style": "background-color: ivory"},pre(JSON.stringify(result, null, 2)) ))
    }
  },
  onConnect: (appManagerConnector) => {
    van.add(appManagerConnectedPanel, h2("Control Panel"))
    van.add(appManagerConnectedPanel, p("Use the below controls for emulation."))
    van.add(appManagerConnectedPanel, button({onclick : () => {
      appManagerConnector.send("lifecyclemanagement.session", SessionRequest(1))
    }},"Launch Firebolt 1.0 App"))
    van.add(appManagerConnectedPanel, button({onclick : () => {
      appManagerConnector.send("lifecyclemanagement.session", SessionRequest(2))
    }},"Launch Firebolt 2.0 App"))
  },
  onMessage: (message, request) => {
    appManagerConnectorHandler.log("Recieved Response <<< ", message)
    if(request.method.includes("lifecyclemanagement.session")) {
      let sessionId = message.result.sessionId;
      appManagerConnectorHandler.log("Launching app with " + sessionId);
      let app = new FireboltApp(request);
      app.launch(sessionId)
    }
  }

}

const setup = () => div(
  h1("👋Meet Firebolt Emulator"),
  div(
    p("Lets start with the connection to Firebolt API Gateway"),
    button({onclick : () => {
       appManagerConnector  = new FireboltGatewayConnector("ws://127.0.0.1:3474?appId=AppManager", appManagerConnectorHandler)
    }},"Connect")
  ),
  appManagerConnectedPanel,
  app_manager_logs
) 

const app_manager_logs = ul({style: "padding:5;list-style-type: none; margin: 0"},li(h2("Emulator Logs")))


const appManagerConnectedPanel = div()
van.add(document.body, setup())
