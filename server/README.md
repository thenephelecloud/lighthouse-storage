Collection of different backends for an example network
Includes both an Actix (https://github.com/actix/actix-web, upload-only) and a simple Go (loosely based off https://github.com/zupzup/golang-http-file-upload-download, upload and DL) implementations.
The aim of the lighthouse-storage crate is to be as backend agnostic as possible, and the final aim is to provide as many protocols (HTTPS, FTP flavours, and so on) and clients as possible.
Currently, files are uploaded and served from the sector-content folder in the storage directory.