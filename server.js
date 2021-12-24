#!/usr/bin/env node

var http = require('http');
var connect = require('connect');
var serveStatic = require('serve-static');
var morgan = require('morgan');
var path = require('path');

var app = connect();

app.use(morgan('dev'));
app.use('/', serveStatic(path.join(__dirname, 'app'), {
    setHeaders: (res, path) => {

        res.setHeader('Cross-Origin-Resource-Policy', 'same-origin');
        res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
        res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
        res.setHeader('Access-Control-Allow-Origin', '*');
    }
}));

const port = 8001;
app.listen(port);

console.log('Server listening on port', port);
