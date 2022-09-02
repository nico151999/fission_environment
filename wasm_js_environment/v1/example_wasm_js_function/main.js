// TODO: support whole node projects, not only single files (potentially using esbuild for bundling and babel for polyfilling)

function handle(param) {
    console.log("Hallo CaaS-Dev-Team");
    console.log(param.method);
    console.log(param.uri);
    console.log(atos(param.body));
    return {
        statusCode: 200,
        headers: [
            {
                key: 'Cool',
                value: 'Foo'
            }
        ],
        body: stoa("This goes back")
    };
}

function atos(arr) {
    for (var i=0, l=arr.length, s='', c; c = arr[i++];)
        s += String.fromCharCode(
            c > 0xdf && c < 0xf0 && i < l-1
                ? (c & 0xf) << 12 | (arr[i++] & 0x3f) << 6 | arr[i++] & 0x3f
                : c > 0x7f && i < l
                    ? (c & 0x1f) << 6 | arr[i++] & 0x3f
                    : c
        );

    return s;
}

function stoa(string) {
    var res = [];
    for (var i = 0; i < string.length; i++) {
        res.push(string.charCodeAt(i));
    }
    return res;
}