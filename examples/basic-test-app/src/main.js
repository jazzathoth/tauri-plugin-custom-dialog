const { invoke } = window.__TAURI__.core;


export class DialogOptions {
  constructor() {
    this.title = null;
    this.width = null;
    this.height = null;
    this.resizable = null;
    this.alwaysOnTop = null;
    this.isModal = null;
  }
}

export class DialogArgs {
  constructor(name = '', url = '') { // Allow basic constructor args maybe
    this.name = name;
    this.url = url;
    this.options = new DialogOptions(); // Nest the options
  }
}

async function testDialog(useModal) {
  let args = new DialogArgs();
  args.options.title = "Test Dialog";
  args.options.width = 600;
  args.options.height = 400;
  args.options.resizable = true;
  args.options.alwaysOnTop = false;
  args.options.isModal = useModal;
  args.name = "testDialog";
  args.url = "./customDialog.html";
  console.log(`Starting dialog. Modal = ${useModal}`);
  

  const dlgResult = await invoke("plugin:custom-dialog|start_custom_dialog", args);
  console.log("finished dialog, result: ", dlgResult);
  
  
}

window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  document.getElementById("testDialogBtn").addEventListener("click", (e) => {
    let useModal = document.getElementById("useModal").checked
    testDialog(useModal)
  });
});
