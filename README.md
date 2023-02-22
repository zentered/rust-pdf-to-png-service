# PDF to PNG Service

![midjourney feature image](.github/assets/midjourney_a_friendly_bot_with_blue_eyes_converting_pdf_doc_ccbb5c1d-6129-4dc3-a7b3-88e0f8fb89db.png)

This is the result of a long, painful and embarrassing journey to resolve a very
simple task: get pngs and webp files from a PDF. Here are a few alternatives we
tried:

- [pdf-to-png-converter](https://www.npmjs.com/package/pdf-to-png-converter) -
  great, simple solution, but limited pdf render functions
- [gm](https://www.npmjs.com/package/gm) - tried many different variations but
  we couldn't get this to work
- [sharp](https://sharp.pixelplumbing.com) - built on `libvips` it's the fastest
  image conversion library, but doesn't come with PDF support out of the box.
  `pdfium` is required and it's a nightmare to try to build a somewhat
  up-to-date Dockerfile with `libvips` and `libpdfium`.

Cloud Functions / Lambda have ImageMagick pre-installed which should be
capbable, but:

- on Google Cloud Functions, ImageMagick is only installed on the Node 12 and 14
  runtime. Not on Node 16 :facepalm:
- In order to read PDF, ImageMagick requires a change in a `policy.xml` which we
  don't have access to in the Functions environment

## Setup

The `Dockerfile` has everything you need. You can build it locally or in the
cloud and use the Cloud Run environment to start the container. There are two
environment variables needed:

```
    SOURCE_BUCKET=source-bucket
    DEST_BUCKET=destination-bucket
```

This service is currently designed for Google Cloud
[EventArc](https://cloud.google.com/eventarc/docs/) triggers. You should create
a trigger on the source bucket when new files are uploaded/finalized. The
trigger will then call the Cloud Run instance and execute this service. There'll
be various versions:

- original png: full size, 90% quality
- preview png: 1920px width, 75% quality, interlace
- thumbnail png: 1024px width, 70% quality, interlace
- lossless webp: full size, 100% quality
- preview webp: 1920px width, 75% quality

All versions will be stored on the destination bucket with the same filename as
the source (including folders), and a prefix for the version.

### Local Development

To build and run locally, you need to create a service account and place the
keyfile in the same folder. You'll need to add:

```
GOOGLE_APPLICATION_CREDENTIALS=keyfile.json
```

As an environment variable. `vips` is linked statically:

```
RUSTFLAGS="-C target-feature=-crt-static $(pkg-config vips --libs)" cargo build && ./target/debug/pdf-service
```

Feel free to fork to adjust this to your own needs.

**Happy converting.**

## Acknowledgements & Thanks

- [Federico Rampazzo](https://github.com/framp) for the help with Rust and
  putting the libraries together.
- [Lachezar Lechev](https://github.com/elpiel) for the final touches and getting
  the http wrapper to work.
- [Naohiro Yoshida](https://github.com/yoshidan) for the Google Cloud libraries
  in Rust

## License

Licensed under [MIT](./LICENSE).

Built with :love_letter: in Cyprus :cyprus:
