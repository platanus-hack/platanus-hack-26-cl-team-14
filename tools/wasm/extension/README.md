# Open the Santiaguito side panel

1. Open `chrome://extensions` in desktop Chrome 142 or later and turn on Developer mode.
2. Select **Load unpacked** and choose this directory.
3. Open Santiaguito at `/web/wasm/`. Reload the page if it was open when you installed the extension.
4. Open **Santiaguito** from Chrome's Extensions menu. Chrome opens the side panel and the simulator shows its controls.
5. Select the Mapocho ferry or return to the usual day. Both controls call the simulator in the active tab.

The side panel uses Chrome's built-in Gemini Nano when the Prompt API supports tool calls. To try it, enable `chrome://flags/#prompt-api-tool-use`, restart Chrome, and enter one request such as "Cierra la estación Tobalaba". Chrome can download the model on first use. The replay and reset controls also work without the model.

The extension connects to the Santiaguito page on `santiaguito.cl`, `localhost`, or `127.0.0.1`. The side panel opens a port to the active tab. It closes the port when you close the panel or switch to another extension panel. The tab uses that connection to show the simulator controls.

- [Chrome side panel API](https://developer.chrome.com/docs/extensions/reference/api/sidePanel)
- [Chrome extension message passing](https://developer.chrome.com/docs/extensions/develop/concepts/messaging)
- [Chrome Prompt API](https://developer.chrome.com/docs/ai/prompt-api)
- [Prompt API tool calls](https://webmachinelearning.github.io/prompt-api/)
