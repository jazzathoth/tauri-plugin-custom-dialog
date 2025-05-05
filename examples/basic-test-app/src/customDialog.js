const { invoke } = window.__TAURI__.core;
const { emit } = window.__TAURI__.event;

document.addEventListener("DOMContentLoaded", async () => {
  var params = new URLSearchParams(window.location.search)
  console.log("Loaded custom dialog tester");
  console.log("current window: ", window.__TAURI__.webviewWindow.getCurrentWebviewWindow());
  console.log("__TAURI__", window.__TAURI__);
  const windowLabel = window.__TAURI__.webviewWindow.getCurrentWebviewWindow().label;
  const resultEventName = params.get('event');
  console.log("resultEventName: ", resultEventName);

  document.getElementById("confirmBtn").addEventListener('click', async () => {
    console.log("confirmBtn clicked");
    const inputData = {test1: "test1", test2: {a: 'a', b: 'b'}}
    const payload = {
      status: 'Confirm',
      data: { value: inputData } // Example: sending data back as JSON object
    };
    console.log(`Invoking finish_dialog with Confirm`);
    try {
      await invoke("greet");
      await emit(resultEventName, payload);
      //await invoke("plugin:custom-dialog:finish_dialog", payload);
    } catch (e) {
      console.log("Failed to invoke finish_dialog: ", e);
    }
  });


  document.getElementById("cancelBtn").addEventListener('click', async () => {
    console.log("cancelBtn clicked");
    const payload = {
      status: "Cancel"
    }
    try {
      // await invoke("plugin:custom-dialog:finish_dialog", payload);
      await emit(resultEventName, payload);
    } catch (e) {
      console.log("Failed to invoke finish_dialog: ", e)
    }
  });
});
