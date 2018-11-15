var GetURL = function() {};
GetURL.prototype = {
run: function(arguments) {
    arguments.completionFunction({"url": document.URL,
                                 "title": document.title,
                                 "document": JSON.stringify(document)
                                 });
}
};
var ExtensionPreprocessingJS = new GetURL;
