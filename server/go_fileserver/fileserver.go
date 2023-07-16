// Example static file server.
//
// Serves static files from the given directory.
// Exports various stats at /stats .
package main

import (
	"expvar"
	"flag"
	"log"

	"html/template"
	"fmt"
	"path/filepath"
	"os"
	"crypto/rand"

	"github.com/valyala/fasthttp"
	"github.com/valyala/fasthttp/expvarhandler"
)

const maxUploadSize = 2000 * 1024 * 1024 // 2 gb

var (
	addr               = flag.String("addr", "localhost:8080", "TCP address to listen to")
	addrTLS            = flag.String("addrTLS", "", "TCP address to listen to TLS (aka SSL or HTTPS) requests. Leave empty for disabling TLS")
	byteRange          = flag.Bool("byteRange", false, "Enables byte range requests if set to true")
	certFile           = flag.String("certFile", "./ssl-cert.pem", "Path to TLS certificate file")
	compress           = flag.Bool("compress", false, "Enables transparent response compression if set to true")
	dir                = flag.String("dir", "/usr/share/nginx/html", "Directory to serve static files from")
	generateIndexPages = flag.Bool("generateIndexPages", true, "Whether to generate directory index pages")
	keyFile            = flag.String("keyFile", "./ssl-cert.key", "Path to TLS key file")
	vhost              = flag.Bool("vhost", false, "Enables virtual hosting by prepending the requested path with the requested hostname")
)

func main() {
	// Parse command-line flags.
	flag.Parse()

	// Setup FS handler
	fs := &fasthttp.FS{
		Root:               *dir,
		IndexNames:         []string{"index.html"},
		GenerateIndexPages: *generateIndexPages,
		Compress:           *compress,
		AcceptByteRange:    *byteRange,
	}
	if *vhost {
		fs.PathRewrite = fasthttp.NewVHostPathRewriter(0)
	}
	fsHandler := fs.NewRequestHandler()

	// Create RequestHandler serving server stats on /stats and files
	// on other requested paths.
	// /stats output may be filtered using regexps. For example:
	//
	//   * /stats?r=fs will show only stats (expvars) containing 'fs'
	//     in their names.
	requestHandler := func(ctx *fasthttp.RequestCtx) {
		switch string(ctx.Path()) {
		case "/stats":
			expvarhandler.ExpvarHandler(ctx)
		case "/upload":
			uploadHandler(ctx)
		default:
			fsHandler(ctx)
			updateFSCounters(ctx)
		}
	}

	srv := &fasthttp.Server{
		Handler:                      requestHandler,
		MaxConnsPerIP:                1,
		MaxRequestBodySize:           260 * 1024 * 1024,
		DisablePreParseMultipartForm: true,
		StreamRequestBody:            true,
	}

	// Start HTTP server.
	if len(*addr) > 0 {
		log.Printf("Starting HTTP server on %q", *addr)
		go func() {
			if err := srv.ListenAndServe(*addr); err != nil {
				log.Fatalf("error in ListenAndServe: %v", err)
			}
		}()
	}

	// Start HTTPS server.
	if len(*addrTLS) > 0 {
		log.Printf("Starting HTTPS server on %q", *addrTLS)
		go func() {
			if err := srv.ListenAndServeTLS(*addrTLS, *certFile, *keyFile); err != nil {
				log.Fatalf("error in ListenAndServeTLS: %v", err)
			}
		}()
	}

	log.Printf("Serving files from directory %q", *dir)
	log.Printf("See stats at http://%s/stats", *addr)

	// Wait forever.
	select {}
}

func uploadHandler(ctx *fasthttp.RequestCtx) {
	if string(ctx.Method()) == fasthttp.MethodGet {
		ctx.SetContentType("text/html; charset=utf-8")
		t, _ := template.ParseFiles("upload.gtpl")
		t.Execute(ctx, nil)
		return
	}
	/*if string(ctx.Method()) != fasthttp.MethodPost {
	if err := ctx.ParseMultipartForm(maxUploadSize); err != nil {
		fmt.Printf("Could not parse multipart form: %v\n", err)
		renderError(ctx, "CANT_PARSE_FORM", fasthttp.StatusInternalServerError)
		return
	}
*/
	// parse and validate file and post parameters
	fileHeader, err := ctx.FormFile("uploadFile")
	if err != nil {
		renderError(ctx, "INVALID_FILE", fasthttp.StatusBadRequest)
		return
	}
	// Get and print out file size
	fileSize := fileHeader.Size
	fmt.Printf("File size (bytes): %v\n", fileSize)
	// validate file size
	if fileSize > maxUploadSize {
		renderError(ctx, "FILE_TOO_BIG", fasthttp.StatusBadRequest)
		return
	}
	/*
	fileBytes, err := ioutil.ReadAll(fileHeader)
	if err != nil {
		renderError(ctx, "INVALID_FILE", fasthttp.StatusBadRequest)
		return
	}

	// check file type, detectcontenttype only needs the first 512 bytes
	detectedFileType := http.DetectContentType(fileBytes)
	fileName := randToken(12)
	fileEndings, err := mime.ExtensionsByType(detectedFileType)
	if err != nil {
		renderError(ctx, "CANT_READ_FILE_TYPE", fasthttp.StatusInternalServerError)
		return
	}
	*/

	newFileName := fileHeader.Filename
	newPath := filepath.Join(*dir, newFileName)
	fmt.Printf("File: %s\n", newPath)

	// write file
	newFile, err := os.Create(newPath)
	if err != nil {
		renderError(ctx, "CANT_WRITE_FILE", fasthttp.StatusInternalServerError)
		return
	}
	defer newFile.Close() // idempotent, okay to call twice
	/*
	if _, err := newFile.Write(fileBytes); err != nil || newFile.Close() != nil {
		renderError(ctx, "CANT_WRITE_FILE", fasthttp.StatusInternalServerError)
		return
	}
	*/
	fasthttp.SaveMultipartFile(fileHeader, newPath)
	ctx.Write([]byte(fmt.Sprintf("SUCCESS - go back to the home to view all the uploaded files")))
}

func updateFSCounters(ctx *fasthttp.RequestCtx) {
	// Increment the number of fsHandler calls.
	fsCalls.Add(1)

	// Update other stats counters
	resp := &ctx.Response
	switch resp.StatusCode() {
	case fasthttp.StatusOK:
		fsOKResponses.Add(1)
		fsResponseBodyBytes.Add(int64(resp.Header.ContentLength()))
	case fasthttp.StatusNotModified:
		fsNotModifiedResponses.Add(1)
	case fasthttp.StatusNotFound:
		fsNotFoundResponses.Add(1)
	default:
		fsOtherResponses.Add(1)
	}
}

// Various counters - see https://pkg.go.dev/expvar for details.
var (
	// Counter for total number of fs calls
	fsCalls = expvar.NewInt("fsCalls")

	// Counters for various response status codes
	fsOKResponses          = expvar.NewInt("fsOKResponses")
	fsNotModifiedResponses = expvar.NewInt("fsNotModifiedResponses")
	fsNotFoundResponses    = expvar.NewInt("fsNotFoundResponses")
	fsOtherResponses       = expvar.NewInt("fsOtherResponses")

	// Total size in bytes for OK response bodies served.
	fsResponseBodyBytes = expvar.NewInt("fsResponseBodyBytes")
)

func renderError(ctx *fasthttp.RequestCtx, message string, statusCode int) {
	ctx.SetStatusCode(statusCode)
	ctx.Write([]byte(message))
}

func randToken(len int) string {
	b := make([]byte, len)
	rand.Read(b)
	return fmt.Sprintf("%x", b)
}