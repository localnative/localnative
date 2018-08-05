// Listen for messages from the popup
chrome.runtime.onMessage.addListener(function (msg, sender, response) {
  if (msg == "get_content") {
    var content = document.body.outerHTML;
    response(content);
  }
});
